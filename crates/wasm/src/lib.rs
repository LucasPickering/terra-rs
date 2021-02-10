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

// TODO after https://github.com/rust-lang/cargo/pull/9030 set
// wasm32-unknown-unknown as the default target for this crate, and kill this
#![cfg(target_arch = "wasm32")]

use terra::{validator::Validate, World, WorldConfig};
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
    ) -> Result<World, JsValue> {
        World::generate(config).map_err(|err| {
            format!(
                "Error during world generation: {:?}\n{}\n",
                err,
                err.backtrace()
            )
            .into()
        })
    }

    /// A type-hacked accessor to get all tiles in a world from Wasm. This
    /// typing can be cleaned up after https://github.com/rustwasm/wasm-bindgen/issues/111
    #[wasm_bindgen]
    pub fn tiles_array(&self, world: &World) -> TileArray {
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
        octaves: number;
        frequency: number;
        lacunarity: number;
        persistence: number;
        exponent: number;
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
