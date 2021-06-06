use crate::{
    util::{cmp_unwrap, range::NumRange, unit::Meter},
    world::{
        generate::{Generate, WorldBuilder},
        Biome, World,
    },
};

/// A benchmark point that defines what biome to use for a particular elevation
/// and humidity. Elevation is first, humidity is second.
struct BiomePoint(Biome, Meter, f64);

impl BiomePoint {
    fn distance_to(&self, elevation_norm: Meter, humidity: f64) -> f64 {
        (self.1 - elevation_norm).0.abs() + (self.2 - humidity).abs()
    }
}

const POINTS: &[BiomePoint] = &[
    // Desert
    BiomePoint(Biome::Desert, Meter(0.25), 0.06),
    BiomePoint(Biome::Desert, Meter(0.50), 0.06),
    // Snow
    BiomePoint(Biome::Snow, Meter(0.75), 0.25),
    BiomePoint(Biome::Snow, Meter(0.55), 0.75),
    // Alpine
    BiomePoint(Biome::Alpine, Meter(0.60), 0.25),
    BiomePoint(Biome::Alpine, Meter(0.50), 0.75),
    // Plains
    BiomePoint(Biome::Plains, Meter(0.20), 0.20),
    // Forest
    BiomePoint(Biome::Forest, Meter(0.25), 0.60),
    // Jungle
    BiomePoint(Biome::Jungle, Meter(0.25), 0.85),
];

/// Generate a biome for every tile that doesn't already have one. Biomes are
/// defined based on elevation and humidity. In order to define the mapping,
/// we turn each tile's elevation and humidity into a 2D point. Then we have a
/// list of benchmark points that map to different biomes. For each tile, we
/// find the closest benchmark point and use its biome.
#[derive(Debug)]
pub struct BiomeGenerator;

impl Generate for BiomeGenerator {
    fn generate(&self, world: &mut WorldBuilder) {
        // We're going to normalize all the elevations so we can use a
        // consistent set of coefficients below. We don't want to map from the
        // full range though, because 99% of the tiles below sea level we won't
        // be touching (since they're already set to ocean). So map from just
        // above-sea-level elevations. We may end up with a few tiles outside
        // the target range of [0,1], but that's fine because the logic will
        // still give them a biome of some sort.
        let elev_input_range =
            NumRange::new(World::SEA_LEVEL, World::ELEVATION_RANGE.max);

        // Set the biome for each tile, but don't overwrite any existing biomes
        for tile in world
            .tiles
            .values_mut()
            .filter(|tile| tile.biome_opt().is_none())
        {
            // Normalize these values so we don't have to update this code when
            // we change the elevation/humidity range bounds
            let elevation_norm = elev_input_range.normalize(tile.elevation());
            let humidity = tile.humidity();

            // Do a naive search to find the nearest point. This is O(n) which
            // isn't particularly efficient, but since the number of points is
            // pretty low, it's fine.
            let (biome, _) = POINTS
                .iter()
                .map(|p| (p.0, p.distance_to(elevation_norm, humidity)))
                .min_by(|(_, d_a), (_, d_b)| cmp_unwrap(d_a, d_b))
                .unwrap(); // safe because we know POINTS is never empty
            tile.set_biome(biome);
        }
    }
}
