use crate::{
    util::{Meter, NumRange},
    world::{
        generate::{Generate, WorldBuilder},
        hex::Cluster,
        Biome, World,
    },
    Meter3,
};
use rand::Rng;

const MAYBE_OCEAN_SIZE_RANGE: NumRange<f32> = NumRange::new(5000.0, 10000.0);
// Any ocean tile at or above this elevation will be coastal
const MIN_COAST_ELEV: Meter = Meter(-3.0);

/// A generator to create oceans at/below sea level.
#[derive(Debug)]
pub struct OceanGenerator;

impl Generate for OceanGenerator {
    fn generate(&self, world: &mut WorldBuilder) -> anyhow::Result<()> {
        // Find all clusters of tiles that are entirely below sea level
        let clusters = Cluster::predicate(&mut world.tiles, |tile| {
            Ok(tile.elevation()? <= World::SEA_LEVEL)
        })?;

        for cluster in clusters {
            // The odds of this cluster becoming an ocean are proportional to
            // its size. Clusters below the min "maybe" size have a chance of 0.
            // Clusters at/above the max size have a chance of 1. Anything in
            // between is proportional to its size.
            let threshold: f32 = world.rng.gen_range(MAYBE_OCEAN_SIZE_RANGE);
            if cluster.tiles().len() as f32 >= threshold {
                // Update every tile in this cluster to be coast/ocean
                for (_, tile) in cluster.into_tiles() {
                    let biome = if tile.elevation()? >= MIN_COAST_ELEV {
                        Biome::Coast
                    } else {
                        Biome::Ocean
                    };
                    tile.set_biome(biome);
                    // Initialize runoff tiles now, since this tile won't
                    // participate in runoff simulation later (because it's
                    // water)
                    tile.set_runoff(Meter3(0.0))?;
                    tile.set_runoff_egress(Default::default());
                }
            }
        }

        Ok(())
    }
}
