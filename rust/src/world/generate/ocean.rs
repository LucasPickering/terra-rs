use crate::{
    util::NumRange,
    world::{
        generate::Generate, hex::WorldMap, tile::TileBuilder, Biome, World,
    },
    WorldConfig,
};
use derive_more::Display;
use rand::Rng;

const MAYBE_OCEAN_SIZE_RANGE: NumRange<f32> = NumRange::new(5000.0, 10000.0);
// Any ocean tile at or above this elevation will be coastal
const MIN_COAST_DEPTH: f64 = -3.0;

/// A generator to create oceans at/below sea level.
#[derive(Copy, Clone, Debug, Default, Display)]
#[display(fmt = "Ocean Generator")]
pub struct OceanGenerator;

impl Generate for OceanGenerator {
    fn generate(
        &self,
        _: &WorldConfig,
        rng: &mut impl Rng,
        tiles: &mut WorldMap<TileBuilder>,
    ) {
        // Find all clusters of tiles that are entirely below sea level
        let clusters = tiles.clusters_predicate(|tile| {
            tile.elevation().unwrap() <= World::SEA_LEVEL
        });

        for cluster in clusters {
            // The odds of this cluster becoming an ocean are proportional to
            // its size. Clusters below the min "maybe" size have a chance of 0.
            // Clusters at/above the max size have a chance of 1. Anything in
            // between is proportional to its size.
            let threshold: f32 = rng.gen_range(MAYBE_OCEAN_SIZE_RANGE);
            if cluster.tiles.len() as f32 >= threshold {
                // Update every tile in this cluster to be coast/ocean
                for (_, tile) in cluster.tiles {
                    let biome = if tile.elevation().unwrap() >= MIN_COAST_DEPTH
                    {
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
