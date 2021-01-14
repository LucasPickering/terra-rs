use crate::world::{
    generate::{Generate, WorldBuilder},
    hex::HexDirection,
    GeoFeature,
};

/// A generator that creates lakes and rivers based on runoff level, runoff
/// ingress, and runoff egress. This has to run AFTER runoff simulation.
#[derive(Debug)]
pub struct WaterFeatureGenerator;

impl Generate for WaterFeatureGenerator {
    fn generate(&self, world: &mut WorldBuilder) -> anyhow::Result<()> {
        let cfg = world.config.geo_feature;
        for tile in world.tiles.values_mut() {
            // Lake
            if tile.runoff()? >= cfg.lake_runoff_threshold {
                tile.add_feature(GeoFeature::Lake)?;
            }

            // River exit
            // We have to copy this into a new structure cause borrow checking
            let river_exit_dirs: Vec<HexDirection> = tile
                .runoff_egress()?
                .iter()
                .filter(|(_, runoff_egress)| {
                    **runoff_egress >= cfg.river_runoff_traversed_threshold
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
