use crate::{
    util::Meter3,
    world::{
        generate::{Generate, WorldBuilder},
        hex::HexDirection,
        GeoFeature,
    },
};

/// Any tile with at least this amount of runoff on it will become a lake
/// TODO move this into the config
const LAKE_RUNOFF_THRESHOLD: Meter3 = Meter3(10.0);
const RIVER_RUNOFF_THRESHOLD: Meter3 = Meter3(100.0);

/// A generator that creates lakes and rivers based on runoff level, runoff
/// ingress, and runoff egress. This has to run AFTER runoff simulation.
#[derive(Debug)]
pub struct WaterFeatureGenerator;

impl Generate for WaterFeatureGenerator {
    fn generate(&self, world: &mut WorldBuilder) -> anyhow::Result<()> {
        for tile in world.tiles.values_mut() {
            // Lake
            if tile.runoff()? >= LAKE_RUNOFF_THRESHOLD {
                tile.add_feature(GeoFeature::Lake)?;
            }

            // River exit
            // We have to copy this into a new structure cause borrow checking
            let river_exit_dirs: Vec<HexDirection> = tile
                .runoff_egress()?
                .iter()
                .filter(|(_, runoff_egress)| {
                    **runoff_egress >= RIVER_RUNOFF_THRESHOLD
                })
                .map(|(dir, _)| *dir)
                .collect();
            for dir in river_exit_dirs {
                tile.add_feature(GeoFeature::RiverExit(dir))?;
            }
        }
        Ok(())
    }
}
