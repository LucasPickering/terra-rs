use crate::{
    util::FloatRange,
    world::{generate::Generate, HexPoint, HexPointMap, Tile, WorldConfig},
};
use noise::{BasicMulti, MultiFractal, NoiseFn, Seedable};
use std::fmt::{self, Display, Formatter};

pub struct ElevationMetadata {
    pub elevation: f64,
}

/// Generate an elevation map using a noise function.
#[derive(Clone, Debug)]
pub struct ElevationGenerator {
    tile_pos_range: FloatRange,
    noise_fn: BasicMulti,
}

impl ElevationGenerator {
    pub fn new(config: &WorldConfig) -> Self {
        // Configure the noise function
        let noise_fn = BasicMulti::new()
            .set_seed(config.seed)
            .set_frequency(1.0)
            .set_lacunarity(4.0)
            .set_persistence(0.5)
            .set_octaves(12);

        let radius_f = config.tile_radius as f64;
        Self {
            tile_pos_range: FloatRange::new(-radius_f, radius_f),
            noise_fn,
        }
    }

    fn generate_elevation(&self, pos: HexPoint) -> f64 {
        // This value is in [-1, 1], we want to map it to the noise function's
        // domain
        let normalize = |v: isize| {
            self.tile_pos_range
                .map_to(&FloatRange::UNIT_RANGE, v as f64)
        };
        let normalized_pos =
            [normalize(pos.x), normalize(pos.y), normalize(pos.z)];

        // Generate noise from the position
        let elevation_unit = self.noise_fn.get(normalized_pos);

        // And map from the noise func's range to elevation
        FloatRange::UNIT_RANGE.map_to(&Tile::ELEVATION_RANGE, elevation_unit)
    }
}

impl Display for ElevationGenerator {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "ElevationGenerator")?;
        Ok(())
    }
}

impl Generate<(), ElevationMetadata> for ElevationGenerator {
    fn generate(
        &self,
        tiles: HexPointMap<()>,
    ) -> HexPointMap<ElevationMetadata> {
        tiles
            .into_iter()
            .map(|(pos, ())| {
                (
                    pos,
                    ElevationMetadata {
                        elevation: self.generate_elevation(pos),
                    },
                )
            })
            .collect()
    }
}
