use terra::{ElevationConfig, World, WorldConfig};

/// Sanity check, make sure the default world config doesn't horrifically crash
/// and burn.
///
/// **NOTE:** the default world config uses a random seed so this could
/// _potentially_ have flaky failures
#[test]
fn test_world_gen_default() {
    let config = WorldConfig::default();
    let world = World::generate(config.clone()).unwrap();
    // Default config uses a random seed each time, so we want to log the
    // config to make sure we can reproduce the failure
    assert_eq!(
        world.tiles().len(),
        30301,
        "Default config failed: {config:?}",
    );
}

/// This config had a bug in the past with a NaN elevation value, related to the
/// edge buffer
#[test]
fn test_world_gen_tiny() {
    let config = WorldConfig {
        seed: 12506774975058000.into(),
        radius: 2,
        elevation: ElevationConfig {
            edge_buffer_fraction: 0.25,
            ..Default::default()
        },
        ..Default::default()
    };
    let world = World::generate(config).unwrap();
    assert_eq!(world.tiles().len(), 19);
}

/// This config had issues in the past with runoff
#[test]
fn test_world_gen_large() {
    let config = WorldConfig {
        seed: 1021522790211909.into(),
        radius: 400,
        elevation: ElevationConfig {
            edge_buffer_fraction: 0.25,
            ..Default::default()
        },
        ..Default::default()
    };
    let world = World::generate(config).unwrap();
    assert_eq!(world.tiles().len(), 481201);
}

/// This config had a bug where the entire world was a giant lake. Runoff was
/// getting multipled by ~1000x during simulation. Runoff simluation contains
/// some safety checks to ensure simluation doesn't accidentally introduce
/// new runoff. One of those checks will fail if this bug is present, so as long
/// as we complete generation without panicking, the bug is gone.
#[test]
fn test_world_gen_mega_lake() {
    let config = WorldConfig {
        seed: 12507776774975058000.into(),
        radius: 60,
        elevation: ElevationConfig {
            edge_buffer_fraction: 0.025,
            ..Default::default()
        },
        ..Default::default()
    };
    let world = World::generate(config).unwrap();
    assert_eq!(world.tiles().len(), 10981);
}
