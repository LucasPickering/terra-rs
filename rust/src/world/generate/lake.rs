use crate::{
    util::Meter3,
    world::{
        generate::{Generate, TileBuilder},
        hex::WorldMap,
        Biome,
    },
    WorldConfig,
};
use rand::Rng;

/// Any tile with at least this amount of runoff on it will become a lake
const LAKE_RUNOFF_THRESHOLD: Meter3 = Meter3(1.0);

/// A generator that creates lakes based on runoff levels. This has to run AFTER
/// runoff simulation.
#[derive(Debug)]
pub struct LakeGenerator;

impl Generate for LakeGenerator {
    fn generate(
        &self,
        _: &WorldConfig,
        _: &mut impl Rng,
        tiles: &mut WorldMap<TileBuilder>,
    ) {
        for tile in tiles.iter_mut() {
            match tile.runoff() {
                Some(runoff) if runoff >= LAKE_RUNOFF_THRESHOLD => {
                    tile.set_biome(Biome::Lake);
                }
                _ => {}
            }
        }
    }
}
