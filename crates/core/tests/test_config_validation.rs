use terra::{
    GeoFeatureConfig, Meter3, NoiseFnConfig, NoiseFnType, RainfallConfig,
    RenderConfig, TileLens, World, WorldConfig, WorldRenderer,
};
use validator::ValidationErrors;

#[test]
fn test_world_config_validation() {
    let config = WorldConfig {
        seed: 0,
        radius: 10001,              // invalid (too big)
        edge_buffer_fraction: -0.1, // invalid
        edge_buffer_exponent: -1.0, // valid (but weird)
        rainfall: RainfallConfig {
            enabled: true,
            evaporation_default: Meter3(-1.0), // can't validate Meter3s :(
            evaporation_land_scale: -1.0,      // invalid
            evaporation_spread_distance: 0,    // valid
            evaporation_spread_exponent: -1.0, // valid (but weird)
            rainfall_fraction_limit: 5.0,      // invalid
        },
        geo_feature: GeoFeatureConfig {
            // Unfortunately we can't validate Meter3s right now
            lake_runoff_threshold: Meter3(-1.0),
            river_runoff_traversed_threshold: Meter3(-1.0),
        },
        elevation: NoiseFnConfig {
            noise_type: NoiseFnType::Fbm,
            octaves: 0,        // valid (but weird)
            frequency: -1.0,   // invalid
            lacunarity: -1.0,  // valid (but weird)
            persistence: -1.0, // valid (but weird)
            exponent: -1.0,    // valid (but weird)
        },
    };

    // This is a bit of a lazy check but it works well enough
    let err = World::generate(config).unwrap_err();
    let validation_errors = err.downcast::<ValidationErrors>().unwrap();
    let mut error_fields = validation_errors
        .errors()
        .keys()
        .copied()
        .collect::<Vec<&str>>();
    error_fields.sort_unstable();
    assert_eq!(
        error_fields,
        vec!["edge_buffer_fraction", "elevation", "radius", "rainfall"],
        "incorrect validation errors in {:#?}",
        validation_errors
    );
}

#[test]
fn test_render_config_validation() {
    let render_config = RenderConfig {
        vertical_scale: 0.0,          // invalid
        tile_lens: TileLens::Surface, // valid
        show_features: false,         // valid
    };

    // This is a bit of a lazy check but it works well enough
    let err = WorldRenderer::new(render_config).unwrap_err();
    let validation_errors = err.downcast::<ValidationErrors>().unwrap();
    let mut error_fields = validation_errors
        .errors()
        .keys()
        .copied()
        .collect::<Vec<&str>>();
    error_fields.sort_unstable();
    assert_eq!(
        error_fields,
        vec!["vertical_scale"],
        "incorrect validation errors in {:#?}",
        validation_errors
    );
}
