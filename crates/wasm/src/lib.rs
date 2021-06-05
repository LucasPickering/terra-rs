//! This crate provides WebAssembly bindings for Terra. The [Terra] struct is
//! the main interface; you'll have to initialize a single instance of [Terra]
//! in order to do any Terra operations from JS. From there, [Terra] provides
//! functions for creating and validating world configs, then generating a
//! World from that config and
//!
//! You probably won't ever want to include this crate in another Rust project.
//! Instead, use `wasm-pack` to build this into an npm package, then import that
//! into your JS project.
//!
//! See the [demo code](https://github.com/LucasPickering/terra-rs/tree/master/demo) for a usage example.

mod util;

use crate::util::ResultExt;
use terra::{RenderConfig, World, WorldConfig, WorldRenderer};
use wasm_bindgen::{prelude::*, JsCast};

/// A top-level interface for interacting with Terra from Wasm.
#[wasm_bindgen]
pub struct Terra;

#[wasm_bindgen]
impl Terra {
    /// Initialize global state needed for world generation. Should be called
    /// once per app instance.
    #[wasm_bindgen(constructor)]
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_logger::init(wasm_logger::Config::default());
        Self
    }

    /// Validate the given config and return it as a strictly typed JS object.
    /// Any missing values will be populated with defaults. If the given value
    /// fails to serialize, or has any invalid values, this will fail.
    pub fn validate_world_config(
        &self,
        input: JsValue,
    ) -> Result<WorldConfigObject, JsValue> {
        util::validate_config::<WorldConfig, WorldConfigObject>(input)
    }

    /// Validate the given config and return it as a strictly typed JS object.
    /// Any missing values will be populated with defaults. If the given value
    /// fails to serialize, or has any invalid values, this will fail.
    pub fn validate_render_config(
        &self,
        input: JsValue,
    ) -> Result<RenderConfigObject, JsValue> {
        util::validate_config::<RenderConfig, RenderConfigObject>(input)
    }

    /// Generate a new world with the given config.
    ///
    /// The config is given as a JS object. It will be deserialized and
    /// validated, and if either of those fail this will return an error.
    pub fn generate_world(
        &self,
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
    pub fn build_renderer(
        &self,
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
    pub fn copy_tiles(&self, world: &World) -> TileArray {
        use js_sys::Array;

        world
            .tiles()
            .values()
            .map(|tile| JsValue::from(tile.clone()))
            .collect::<Array>()
            .unchecked_into()
    }
}

#[wasm_bindgen(typescript_custom_section)]
const TS_APPEND_CONTENT: &'static str = r#"
/**
 * See description in the `extern "C"` section below
 */
export interface WorldConfigObject {
    seed: string | number;
    radius: number;
    edge_buffer_fraction: number;
    edge_buffer_exponent: number;
    rainfall: {
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
    elevation: {
        noise_type: 'basic_multi' | 'billow' | 'fbm' | 'hybrid_multi' | 'ridged_multi';
        octaves: number;
        frequency: number;
        lacunarity: number;
        persistence: number;
        exponent: number;
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
