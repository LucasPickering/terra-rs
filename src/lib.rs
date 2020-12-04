use wasm_bindgen::prelude::*;

mod render;
mod util;
mod world;

#[wasm_bindgen(start)]
pub fn main() {
    // env_logger::init();
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    wasm_logger::init(wasm_logger::Config::default());
    render::run();
}
