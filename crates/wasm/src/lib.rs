//! This crate provides WebAssembly bindings for Terra. The [Terra] struct is
//! the main interface; you'll have to initialize a single instance of [Terra]
//! in order to do any Terra operations from JS. From there, [Terra] provides
//! functions for creating and validating world configs, then generating a
//! World from that config and
//!
//! You probably won't every want to include this crate in another Rust project.
//! Instead, use `wasm-pack` to build this into an npm package, then import that
//! from your JS project.
//!
//! See the [demo code](https://github.com/LucasPickering/terra-rs/tree/master/demo) for a usage example.

use terra::{
    anyhow, validator::Validate, Color3, Tile, TileLens, World, WorldConfig,
};
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

    /// Get the default world config as a JS object.
    #[wasm_bindgen]
    pub fn default_config(&self) -> WorldConfigObject {
        JsValue::from_serde(&WorldConfig::default())
            .unwrap()
            .unchecked_into()
    }

    /// Verify that the given JS object is a valid Terra world config. Return
    /// the validated config, with all defaults populated, if it's valid. Return
    /// an error if it isn't.
    pub fn validate_config(
        &self,
        input: WorldConfigObject,
    ) -> Result<WorldConfigObject, JsValue> {
        // Deserialize the config then validate it manually
        let config = self.deserialize_config(input)?;
        config.validate().map_err::<JsValue, _>(|err| {
            format!("Invalid config: {:?}", err).into()
        })?;
        // Re-serialize it back into a JS object
        Ok(JsValue::from_serde(&config).unwrap().unchecked_into())
    }

    /// Deserialize a JS object into a [WorldConfig]. The input should be an
    /// **object**, not a JSON string. Will return an error if deserialization
    /// fails in any way.
    #[wasm_bindgen]
    pub fn deserialize_config(
        &self,
        input: WorldConfigObject,
    ) -> Result<WorldConfig, JsValue> {
        JsValue::into_serde(&input).map_err(|err| {
            format!("Error deserializing config: {:?}", err).into()
        })
    }

    /// Generate a new world with the given config.
    #[wasm_bindgen]
    pub fn generate_world(
        &self,
        config: WorldConfig,
    ) -> Result<WasmWorld, JsValue> {
        let world = World::generate(config).map_err(to_js_error)?;
        Ok(WasmWorld(world))
    }

    /// A Wasm-friendly wrapper around [TileLens::tile_color]. wasm-bindgen
    /// doesn't support impls on enums so we have to do this
    /// https://github.com/rustwasm/wasm-bindgen/issues/1715
    #[wasm_bindgen]
    pub fn tile_color(&self, tile: &Tile, lens: TileLens) -> Color3 {
        lens.tile_color(tile)
    }
}

/// A wrapper around [terra::World] that provides a nice Wasm-friendly API.
#[wasm_bindgen]
pub struct WasmWorld(World);

#[wasm_bindgen]
impl WasmWorld {
    /// A type-hacked wrapper around [terra::World::tiles]. This typing can be
    /// cleaned up after https://github.com/rustwasm/wasm-bindgen/issues/111
    pub fn tiles(&self) -> TileArray {
        use js_sys::Array;

        self.0
            .tiles()
            .values()
            .map(|tile| JsValue::from(tile.clone()))
            .collect::<Array>()
            .unchecked_into()
    }

    /// See [terra::World::tile_render_height]
    pub fn tile_render_height(&self, tile: &Tile) -> f64 {
        self.0.tile_render_height(tile)
    }

    /// See [terra::World::to_json]
    pub fn to_json(&self) -> String {
        self.0.to_json()
    }

    /// See [terra::World::to_bin]
    pub fn to_bin(&self) -> Vec<u8> {
        self.0.to_bin()
    }

    /// See [terra::World::to_svg]
    pub fn to_svg(&self, lens: TileLens, show_features: bool) -> String {
        self.0.to_svg(lens, show_features)
    }

    /// See [terra::World::to_stl]
    pub fn to_stl(&self) -> Vec<u8> {
        self.0.to_stl()
    }
}

/// Helper to convert an anyhow error to a JS error value.
fn to_js_error(error: anyhow::Error) -> JsValue {
    format!("{:?}\n{}", error, error.backtrace()).into()
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
    render: {
        y_scale: number;
    };
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

    /// Type hack needed until https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen(typescript_type = "Tile[]")]
    pub type TileArray;
}
