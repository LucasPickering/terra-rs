use crate::{
    world::{generate::Generate, hex::WorldMap, tile::TileBuilder, Biome},
    WorldConfig,
};
use rand::Rng;

/// Any tile with at least this amount of runoff on it will become a lake
const LAKE_RUNOFF_THRESHOLD: f64 = 0.1;

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
            if tile.runoff() >= LAKE_RUNOFF_THRESHOLD {
                tile.set_biome(Biome::Lake);
            }
        }
    }
}
