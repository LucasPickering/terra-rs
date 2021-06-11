use crate::{
    util::{self, range::NumRange, unit::Meter},
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
    fn generate(&self, world: &mut WorldBuilder) {
        let elev_config = world.config.elevation;
        let noise_fn: TileNoiseFn<Meter> = TileNoiseFn::new(
            &mut world.rng,
            elev_config.noise_fn,
            World::ELEVATION_RANGE,
        );

        // Buffer size is given as a fraction of the total radius, we need
        // to convert that to a [start,stop] range
        let radius = world.config.radius as f64;
        let buffer_size = (radius * elev_config.edge_buffer_fraction).round();
        // +1 because the lower bound is inclusive
        let buffer_range = NumRange::new(radius - buffer_size + 1.0, radius);

        for tile in world.tiles.values_mut() {
            let pos = tile.position();
            let d = pos.distance_to(HexPoint::ORIGIN) as f64;

            // Determine the range of potential elevation outputs for this tile.
            // For most tiles it's static, but for some the edge buffer will
            // restrict that range
            let elev_range: NumRange<Meter, f64> = if buffer_range.contains(d) {
                // This tile is near the edge of the world, so we want to push
                // it down a bit. The further out it is, the more we push it.

                // We do this by varying the maximum of the elevation range.
                // At the innermost ring of the buffer, it'll be pretty much
                // normal. At the outermost ring, it'll be sea level. This
                // guarantees at least one ring of ocean at the edge.
                let elev_max = buffer_range
                    // Convert the value to a fraction representing its distance
                    // from the outermost edge. 0 will be the outermost ring,
                    // 1 will be the innermost ring **of the buffer**
                    .value(d)
                    .normalize()
                    .invert()
                    // Apply exponent curve
                    .apply(|v| v.powf(elev_config.edge_buffer_exponent)) // Use a smooth gradient
                    .convert::<Meter>()
                    // Pick a new upper bound on elevation, somewhere between
                    // sea level and the standard upper bound. For the
                    // outermost ring, this will be sea level, innermost will
                    // remain the standard value
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

            // This noise value will span the elevation range (ish)
            // TODO https://github.com/LucasPickering/terra-rs/issues/19
            // Figure out why values aren't spanning the full elevation range
            let elevation: Meter = noise_fn
                .get(pos)
                // Map to our output range which may be compressed by the buffer
                .map_to(elev_range)
                // Round to nearest multiple of the specified interval (if any)
                .apply(|val| match elev_config.rounding_interval {
                    Some(rounding_interval) => {
                        util::round(val, rounding_interval)
                    }
                    None => val,
                })
                .inner();
            tile.set_elevation(elevation);
        }
    }
}
