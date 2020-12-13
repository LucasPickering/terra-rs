use crate::{
    util::NumRange,
    world::{
        generate::Generate, hex::HexPointMap, tile::TileBuilder, Biome, World,
    },
};
use derive_more::Display;
use rand::Rng;
use rand_pcg::Pcg64;

const MAYBE_OCEAN_SIZE_RANGE: NumRange<f32> = NumRange::new(50.0, 100.0);
// Any ocean tile at or above this elevation will be coastal
const MIN_COAST_DEPTH: f64 = -3.0;

/// A generator to create oceans at/below sea level.
#[derive(Copy, Clone, Debug, Default, Display)]
#[display(fmt = "Ocean Generator")]
pub struct OceanGenerator;

impl Generate for OceanGenerator {
    fn generate(&self, tiles: &mut HexPointMap<TileBuilder>, rng: &mut Pcg64) {
        // Find all clusters of tiles that are entirely below sea level
        let clusters = tiles
            .clusters_predicate(|tile| tile.elevation() <= World::SEA_LEVEL);

        for cluster in clusters {
            // The odds of this cluster becoming an ocean are proportional to
            // its size. Clusters below the min "maybe" size have a chance of 0.
            // Clusters at/above the max size have a chance of 1. Anything in
            // between is proportional to its size.
            let ocean_chance =
                MAYBE_OCEAN_SIZE_RANGE.normalize(cluster.0.len() as f32);
            // This value is [0, 1), so if our chance is <=0, it will always be
            // false, and it the chance is >=1 it will always be true.
            let r: f32 = rng.gen();
            if r < ocean_chance {
                // Update every tile in this cluster to be coast/ocean
                for (_, tile) in cluster.0 {
                    let biome = if tile.elevation() >= MIN_COAST_DEPTH {
                        Biome::Coast
                    } else {
                        Biome::Ocean
                    };
                    tile.set_biome(biome);
                }
            }
        }
    }
}
