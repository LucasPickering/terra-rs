use crate::{
    util::{range::NumRange, unit::Meter3},
    world::{
        generate::{Generate, TileBuilder, WorldBuilder},
        hex::{HexAxialDirection, HexAxis, HexPoint, HexPointMap},
        World,
    },
    WorldConfig,
};
use std::iter;

/// Generate rainfall on each tile using a simulation of
/// evaporation & precipitation based on wind direction and elevation. This has
/// to run **after elevation and ocean generation**.
#[derive(Debug)]
pub struct RainfallGenerator;

impl Generate for RainfallGenerator {
    fn generate(&self, world: &mut WorldBuilder) {
        // This generator works as a one-pass rainfall simulator. Imagine a line
        // (our clouds) that runs perpendicular to the wind, and is as wide as
        // the world at its fattest point (i.e. it always covers the
        // whole world, but in most places will hang over the edge a
        // bit).
        //
        // We'll start at the upwind side of the world, and move along the world
        // one row at a time. For each row that the clouds are over, we'll pick
        // up moisture (where the amount is based on whether or not it's water),
        // and drop rain (where the amount is based on elevation).

        // This step can be disabled to improve performance
        if world.config.rainfall.enabled {
            let mut cloud_line =
                CloudLine::new(&world.config, world.wind_direction());

            let radius = world.config.radius as i16;
            for _ in -radius..=radius {
                cloud_line.precipitate_and_evaporate(&mut world.tiles);
                cloud_line.advance();
            }
        } else {
            for tile in world.tiles.values_mut() {
                tile.set_rainfall(Meter3(0.0));
            }
        }
    }
}

struct CloudLine<'a> {
    config: &'a WorldConfig,
    wind_direction: HexAxialDirection,
    coax_offset: i16,
    cloud_volumes: Vec<Meter3>,
    rainfall_factor_range: NumRange<f64>,
    spread_coefficients: Vec<f64>,
}

impl<'a> CloudLine<'a> {
    fn new(config: &'a WorldConfig, wind_direction: HexAxialDirection) -> Self {
        let world_width = (config.radius as usize) * 2 + 1;
        let cloud_volumes: Vec<Meter3> =
            iter::repeat(Meter3(0.0)).take(world_width).collect();

        let l = (2 * config.rainfall.evaporation_spread_distance + 1) as usize;
        let center_idx = config.rainfall.evaporation_spread_distance as usize;
        let mut spread_coefficients: Vec<f64> =
            iter::repeat(0.0).take(l).collect();
        let spread_range = NumRange::new(
            0.0,
            config.rainfall.evaporation_spread_distance as f64,
        );
        for d in 0..=config.rainfall.evaporation_spread_distance {
            let v = spread_range
                .normalize(d as f64)
                .powf(config.rainfall.evaporation_spread_exponent);
            spread_coefficients[center_idx - d as usize] = v;
            spread_coefficients[center_idx + d as usize] = v;
        }
        let sum: f64 = spread_coefficients.iter().sum();
        for v in spread_coefficients.iter_mut() {
            *v /= sum;
        }

        Self {
            config,
            wind_direction,
            coax_offset: -(config.radius as i16) * wind_direction.signum(),
            cloud_volumes,
            rainfall_factor_range: NumRange::new(
                0.0,
                config.rainfall.rainfall_fraction_limit,
            ),
            spread_coefficients,
        }
    }

    fn index_to_pos(&self, index: usize) -> HexPoint {
        let perp_offset = (index as i16) - (self.config.radius as i16);
        match self.wind_direction.axis {
            HexAxis::X => HexPoint::new_xy(self.coax_offset, perp_offset),
            HexAxis::Y => HexPoint::new_yz(self.coax_offset, perp_offset),
            HexAxis::Z => HexPoint::new_xz(perp_offset, self.coax_offset),
        }
    }

    /// Calculate a scaling factor for how much rain a tile should get. Higher
    /// tiles get more rain. This simulates the clouds getting stuck up against
    /// mountains.
    fn calc_rainfall_factor(&self, tile: &TileBuilder) -> f64 {
        World::ELEVATION_RANGE
            .value(tile.elevation())
            .convert::<f64>()
            .map_to(self.rainfall_factor_range)
            .clamp()
            .inner()
    }

    /// Calculate how much water vapor this tile produces when the clouds pass
    /// over it.
    fn calc_evaporation(&self, tile: &TileBuilder) -> Meter3 {
        if tile.is_water_biome() {
            self.config.rainfall.evaporation_default
        } else {
            self.config.rainfall.evaporation_default
                * self.config.rainfall.evaporation_land_scale
        }
    }

    /// Simulation precipitation on the current line, then simulation
    /// evaporation. It's easiest to do this all together so we only have to
    /// each tile lookup once.
    fn precipitate_and_evaporate(
        &mut self,
        tiles: &mut HexPointMap<TileBuilder>,
    ) {
        let mut evaporation: Vec<Meter3> = iter::repeat(Meter3(0.0))
            .take(self.cloud_volumes.len())
            .collect();

        // drop the load
        #[allow(clippy::needless_range_loop)]
        for i in 0..self.cloud_volumes.len() {
            let pos = self.index_to_pos(i);
            if let Some(tile) = tiles.get_mut(&pos) {
                evaporation[i] = self.calc_evaporation(tile);
                // Each tile receives some fraction of the current water
                // available in the cloud
                let rainfall =
                    self.cloud_volumes[i] * self.calc_rainfall_factor(tile);
                self.cloud_volumes[i] -= rainfall;
                tile.set_rainfall(rainfall);
            }
        }

        // big succ
        // For each slot in the clouds, it draws water from n tiles to its
        // left and right. The exact distance, and the diminishing factor, are
        // determined by the config.
        let spread_dist =
            self.config.rainfall.evaporation_spread_distance as usize;
        for (i, v) in self.cloud_volumes.iter_mut().enumerate() {
            let evap: Meter3 = self
                .spread_coefficients
                .iter()
                .enumerate()
                .filter_map(|(j, coef)| {
                    let idx = (i + j).checked_sub(spread_dist)?;
                    let e = *evaporation.get(idx)?;
                    Some(e * coef)
                })
                .sum();
            *v += evap;
        }
    }

    fn advance(&mut self) {
        self.coax_offset += self.wind_direction.signum();
    }
}
