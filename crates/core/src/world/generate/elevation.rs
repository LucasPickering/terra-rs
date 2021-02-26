use crate::{
    util::{Meter, NumRange},
    world::{
        generate::{noise::TileNoiseFn, Generate, WorldBuilder},
        hex::{HasHexPosition, HexPoint},
        World,
    },
};

/// Generate an elevation map using a noise function.
#[derive(Debug)]
pub struct ElevationGenerator;

impl Generate for ElevationGenerator {
    fn generate(&self, world: &mut WorldBuilder) -> anyhow::Result<()> {
        let config = world.config;
        let normal_range = NumRange::normal_range();
        let noise_fn: TileNoiseFn<Meter> =
            TileNoiseFn::new(&mut world.rng, &config.elevation, normal_range);

        // Buffer size is given as a fraction of the total radius, we need
        // to convert that to a [start,stop] range
        let radius = config.radius as f64;
        let buffer_size = (radius * config.edge_buffer_fraction).round();
        // +1 because the lower bound is inclusive
        let buffer_range =
            NumRange::new((radius - buffer_size + 1.0) as f64, radius);

        for tile in world.tiles.values_mut() {
            let pos = tile.position();
            let d = pos.distance_to(HexPoint::ORIGIN) as f64;

            let elev_range: NumRange<Meter, f64> = if buffer_range.contains(d) {
                // This tile is near the edge of the world, so we want to push
                // it down a bit. The further out it is, the more we push it.

                // We do this by varying the maximum of the elevation range.
                // At the innermost ring of the buffer, it'll be pretty much
                // normal. At the outermost ring, it'll be sea level. This
                // guarantees at least one ring of ocean at the edge.
                let elev_max = buffer_range
                    .value(d)
                    .normalize()
                    .invert()
                    // We now have a value where 0 is the outermost ring and 1
                    // is the innermost ring OF THE BUFFER
                    .apply(|v| v.powf(config.edge_buffer_exponent)) // Use a smooth gradient
                    .convert::<Meter>()
                    .map_to(NumRange::new(
                        World::SEA_LEVEL,
                        World::ELEVATION_RANGE.max,
                    ))
                    .inner();

                NumRange::new(World::ELEVATION_RANGE.min, elev_max)
            } else {
                // Tile is close to the middle, use the normal elevation range
                World::ELEVATION_RANGE
            };

            let elev = normal_range
                .value(noise_fn.get(pos))
                .convert::<Meter>()
                .map_to(elev_range)
                .inner();
            tile.set_elevation(elev)?;
        }

        Ok(())
    }
}
