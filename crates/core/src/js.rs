//! JS-compatibility code. This code is meant for Wasm contexts, specifically
//! where the library is being used to generate worlds, but the world data is
//! then being processed in JS/TS. This module/feature should **not** be
//! enabled when compiling to Wasm in a pure-Rust context.
//!
//! There are some other wasm-bindgen-enabled functions and types elsewhere in
//! the crate. This module is the home for stuff that is **exclusively** for
//! Wasm boundary usage.
//!
//! You probably won't ever want to enable this feature when including this
//! crate in another Rust project. Instead, use `wasm-pack` to build this into
//! an npm package, then import that into your JS project.
//!
//! See the [demo code](https://github.com/LucasPickering/terra-rs/tree/master/demo) for a usage example.

mod util;

use crate::{
    js::util::ResultExt, RenderConfig, World, WorldConfig, WorldRenderer,
};
use wasm_bindgen::{prelude::*, JsCast};

/// Executed when the Wasm module is first loaded
#[wasm_bindgen(start)]
pub fn main() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());
}

/// Validate the given config and return it as a strictly typed JS object.
/// Any missing values will be populated with defaults. If the given value
/// fails to serialize, or has any invalid values, this will fail.
#[wasm_bindgen]
pub fn validate_world_config(
    input: JsValue,
) -> Result<WorldConfigObject, JsValue> {
    util::validate_config::<WorldConfig, WorldConfigObject>(input)
}

/// Validate the given config and return it as a strictly typed JS object.
/// Any missing values will be populated with defaults. If the given value
/// fails to serialize, or has any invalid values, this will fail.
#[wasm_bindgen]
pub fn validate_render_config(
    input: JsValue,
) -> Result<RenderConfigObject, JsValue> {
    util::validate_config::<RenderConfig, RenderConfigObject>(input)
}

/// Generate a new world with the given config.
///
/// The config is given as a JS object. It will be deserialized and
/// validated, and if either of those fail this will return an error.
#[wasm_bindgen]
pub fn generate_world(
    world_config: WorldConfigObject,
) -> Result<World, JsValue> {
    // Deserialize the config JS object into a Rust value
    let world_config = JsValue::into_serde(&world_config).into_js()?;
    // This will validate the config
    World::generate(world_config).into_js()
}

/// Create a world renderer that can be used to render any world into
/// various visual formats. A renderer must be configured at creation
/// using the given config, but from then it can be used to render any
/// number of worlds.
///
/// The config is given as a JS object. It will be deserialized and
/// validated, and if either of those fail this will return an error.
#[wasm_bindgen]
pub fn build_renderer(
    render_config: RenderConfigObject,
) -> Result<WorldRenderer, JsValue> {
    // Deserialize the config JS object into a Rust value
    let render_config = JsValue::into_serde(&render_config).into_js()?;
    // This will validate the config
    WorldRenderer::new(render_config).into_js()
}

/// A type-hacked wrapper around [terra::World::tiles]. This typing can be
/// cleaned up after https://github.com/rustwasm/wasm-bindgen/issues/111,
/// then we can use the built-in `.tiles()` on the world instead.
#[wasm_bindgen]
pub fn copy_tiles(world: &World) -> TileArray {
    use js_sys::Array;

    world
        .tiles()
        .values()
        .map(|tile| JsValue::from(tile.clone()))
        .collect::<Array>()
        .unchecked_into()
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * See description in the `extern "C"` section below
 */
export interface WorldConfigObject {
    seed: string | number;
    radius: number;
    elevation: {
        noise_fn: {
            noise_type: 'basic_multi' | 'billow' | 'fbm' | 'hybrid_multi' | 'ridged_multi';
            octaves: number;
            frequency: number;
            lacunarity: number;
            persistence: number;
            exponent: number;
        };
        rounding_interval: number | undefined;
        edge_buffer_fraction: number;
        edge_buffer_exponent: number;
    };
    rainfall: {
        enabled: boolean;
        evaporation_default: number;
        evaporation_land_scale: number;
        evaporation_spread_distance: number;
        evaporation_spread_exponent: number;
        rainfall_fraction_limit: number;
    };
    geo_feature: {
        lake_runoff_threshold: number;
        river_runoff_traversed_threshold: number;
    };
}

/**
 * See description in the `extern "C"` section below
 */
export interface RenderConfigObject {
    vertical_scale: number;
    tile_lens: 'surface' | 'biome' | 'elevation' | 'humidity' | 'runoff';
    show_features: boolean;
}
"#;

#[wasm_bindgen]
extern "C" {
    /// A TS version of the [WorldConfig] type from the core crate. This needs
    /// to be mapped manually because some types change between Rust and TS.
    /// This type represents what **can be deserialized into a
    /// [WorldConfig]**.
    ///
    /// **It is very important that this stays up to date with the [WorldConfig]
    /// type**.
    #[wasm_bindgen(typescript_type = "WorldConfigObject")]
    pub type WorldConfigObject;

    /// Similar to [WorldConfigObject], but for the render config instead.
    #[wasm_bindgen(typescript_type = "RenderConfigObject")]
    pub type RenderConfigObject;

    /// Type hack needed until https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen(typescript_type = "Tile[]")]
    pub type TileArray;
}
