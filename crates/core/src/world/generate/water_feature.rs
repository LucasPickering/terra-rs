use crate::{
    world::{
        generate::{Generate, WorldBuilder},
        hex::TileDirection,
        GeoFeature,
    },
    Meter3,
};

/// A generator that creates lakes and rivers based on runoff level, runoff
/// ingress, and runoff egress. This has to run AFTER runoff simulation.
#[derive(Debug)]
pub struct WaterFeatureGenerator;

impl Generate for WaterFeatureGenerator {
    fn generate(&self, world: &mut WorldBuilder) {
        let cfg = world.config.geo_feature;
        for tile in world.tiles.values_mut() {
            // Lake
            if tile.runoff() >= cfg.lake_runoff_threshold {
                tile.add_feature(GeoFeature::Lake);
            }

            // River exit
            // We have to copy this into a vec before mutating cause borrow ck
            let runoff_traversed: Vec<(TileDirection, Meter3)> = tile
                .runoff_traversed()
                .iter()
                .map(|(k, v)| (*k, *v))
                .collect();
            for (dir, runoff_net) in runoff_traversed {
                if runoff_net > cfg.river_runoff_traversed_threshold {
                    tile.add_feature(GeoFeature::RiverEntrance {
                        direction: dir,
                        volume: runoff_net,
                    });
                } else if runoff_net < -cfg.river_runoff_traversed_threshold {
                    tile.add_feature(GeoFeature::RiverExit {
                        direction: dir,
                        volume: -runoff_net,
                    });
                }
            }
        }
    }
}
