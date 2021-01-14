use terra::{World, WorldConfig};

#[test]
fn test_world_gen_default() {
    let world = World::generate(WorldConfig::default()).unwrap();
    assert_eq!(world.tiles().len(), 30301);
}

#[test]
fn test_world_gen_large() {
    // This config had issues in the past with runoff
    let config = WorldConfig {
        seed: 1021522790211909,
        radius: 400,
        edge_buffer_fraction: 0.25,
        ..Default::default()
    };
    let world = World::generate(config).unwrap();
    assert_eq!(world.tiles().len(), 481201);
}
