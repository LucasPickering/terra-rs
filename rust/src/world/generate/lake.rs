use crate::{
    util::Meter3,
    world::{
        generate::{Generate, WorldBuilder},
        Biome,
    },
};

/// Any tile with at least this amount of runoff on it will become a lake
const LAKE_RUNOFF_THRESHOLD: Meter3 = Meter3(1.0);

/// A generator that creates lakes based on runoff levels. This has to run AFTER
/// runoff simulation.
#[derive(Debug)]
pub struct LakeGenerator;

impl Generate for LakeGenerator {
    fn generate(&self, world: &mut WorldBuilder) {
        for tile in world.tiles.iter_mut() {
            match tile.runoff() {
                Some(runoff) if runoff >= LAKE_RUNOFF_THRESHOLD => {
                    tile.set_biome(Biome::Lake);
                }
                _ => {}
            }
        }
    }
}
