use terra::{World, WorldConfig};

/// Sanity check, make sure the default world config doesn't horrifically crash
/// and burn.
///
/// **NOTE:** the default world config uses a random seed so this could
/// _potentially_ have flaky failures
#[test]
fn test_world_gen_default() {
    let config = WorldConfig::default();
    let world = World::generate(config).unwrap();
    // Default config uses a random seed each time, so we want to log the
    // config to make sure we can reproduce the failure
    assert_eq!(
        world.tiles().len(),
        30301,
        "Default config failed: {:?}",
        config
    );
}

/// This config had a bug in the past with a NaN elevation value, related to the
/// edge buffer
#[test]
fn test_world_gen_tiny() {
    let config = WorldConfig {
        seed: 12506774975058000,
        radius: 2,
        edge_buffer_fraction: 0.25,
        ..Default::default()
    };
    let world = World::generate(config).unwrap();
    assert_eq!(world.tiles().len(), 19);
}

/// This config had issues in the past with runoff
#[test]
fn test_world_gen_large() {
    let config = WorldConfig {
        seed: 1021522790211909,
        radius: 400,
        edge_buffer_fraction: 0.25,
        ..Default::default()
    };
    let world = World::generate(config).unwrap();
    assert_eq!(world.tiles().len(), 481201);
}
