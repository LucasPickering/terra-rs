#![feature(const_fn)]
#![feature(trait_alias)]

use crate::{config::TerraConfig, render::Scene, world::World};
use wasm_bindgen::prelude::*;

mod camera;
mod config;
mod input;
mod render;
mod util;
mod world;

/// Top-level struct for a Terra instance. This holds every we need to render
/// Terra from the outside.
#[wasm_bindgen]
pub struct Terra {
    world: World,
    scene: Scene,
}

#[wasm_bindgen]
impl Terra {
    /// Initialize a Terra instance
    #[wasm_bindgen]
    pub async fn load(canvas_id: String) -> Result<Terra, js_sys::Error> {
        async fn helper(canvas_id: String) -> anyhow::Result<Terra> {
            std::panic::set_hook(Box::new(console_error_panic_hook::hook));
            wasm_logger::init(wasm_logger::Config::default());

            let config = TerraConfig::load().await?;
            let world = World::generate(config.world);
            let scene = Scene::new(&canvas_id, config.input, &world)?;

            Ok(Terra { world, scene })
        }

        helper(canvas_id)
            .await
            .map_err(|err| js_sys::Error::new(&err.to_string()))
    }

    /// Run a single render. This is the entrypoint into the game loop, so it
    /// should be called once per frame.
    #[wasm_bindgen]
    pub fn render(&mut self) {
        self.scene.render().unwrap_throw();
    }
}
