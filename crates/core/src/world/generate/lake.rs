use crate::{
    util::Meter3,
    world::{
        generate::{Generate, WorldBuilder},
        GeoFeature,
    },
};

/// Any tile with at least this amount of runoff on it will become a lake
/// TODO move this into the config
const LAKE_RUNOFF_THRESHOLD: Meter3 = Meter3(10.0);

/// A generator that creates lakes based on runoff levels. This has to run AFTER
/// runoff simulation.
#[derive(Debug)]
pub struct LakeGenerator;

impl Generate for LakeGenerator {
    fn generate(&self, world: &mut WorldBuilder) -> anyhow::Result<()> {
        for tile in world.tiles.values_mut() {
            if tile.runoff()? >= LAKE_RUNOFF_THRESHOLD {
                tile.add_feature(GeoFeature::Lake)?;
            }
        }
        Ok(())
    }
}
