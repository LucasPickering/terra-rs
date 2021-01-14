use log::info;
use terra::{World, WorldConfig};
use wasm_bindgen::prelude::*;

/// A top-level interface for interacting with Terra from Wasm.
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Terra;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
impl Terra {
    /// Initialize global state needed for world generation. Should be called
    /// once per app instance.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(constructor))]
    pub fn initialize() -> Self {
        std::panic::set_hook(Box::new(console_error_panic_hook::hook));
        wasm_logger::init(wasm_logger::Config::default());
        Self
    }

    /// Generate a new world with the given config.
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
    pub fn generate_world(&self, config: JsValue) -> Result<World, JsValue> {
        info!("Loading config");
        let config: WorldConfig = serde_wasm_bindgen::from_value(config)?;

        World::generate(config).map_err(|err| {
            format!(
                "Error during world generation: {:?}\n{}\n",
                err,
                err.backtrace()
            )
            .into()
        })
    }
}
