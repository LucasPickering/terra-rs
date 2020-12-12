#![feature(const_fn)]
#![feature(trait_alias)]

use crate::{
    config::TerraConfig, input::InputEvent, render::Scene, world::World,
};
use input::InputHandler;
use wasm_bindgen::prelude::*;

mod camera;
mod config;
mod input;
mod render;
mod util;
mod world;

/// Top-level struct for a Terra instance. This holds every we need to render
/// and interact with Terra from the outside. All interaction to/from wasm
/// should go through this struct.
#[wasm_bindgen]
pub struct Terra {
    world: World,
    input_handler: InputHandler,
    scene: Scene,
}

#[wasm_bindgen]
impl Terra {
    /// Initialize a Terra instance
    #[wasm_bindgen]
    pub async fn load(canvas_id: String) -> Result<Terra, JsValue> {
        async fn helper(canvas_id: String) -> anyhow::Result<Terra> {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            wasm_logger::init(wasm_logger::Config::default());

            let config = TerraConfig::load().await?;
            let world = World::generate(config.world);
            let input_handler = InputHandler::new(config.input);
            let scene = Scene::new(&canvas_id, &world)?;

            Ok(Terra {
                world,
                input_handler,
                scene,
            })
        }

        helper(canvas_id)
            .await
            .map_err(|err| err.to_string().into())
    }

    /// Register a single input event. This should be called directly by any
    /// input event callback in TS. The given value should deserialize into an
    /// [InputEvent].
    #[wasm_bindgen]
    pub fn handle_event(&self, event: JsValue) -> Result<(), JsValue> {
        // Deserialize the JS value into our Rust type before forwarding it
        let event: InputEvent = serde_wasm_bindgen::from_value(event)?;
        self.input_handler
            .ingest(event)
            .map_err(|err| err.to_string().into())
    }

    /// Run a single render. This is the entrypoint into the game loop, so it
    /// should be called once per frame.
    #[wasm_bindgen]
    pub fn render(&mut self) {
        self.scene.render(&mut self.input_handler).unwrap_throw();
    }
}
