mod render;
mod util;
mod world;

use crate::world::{World, WorldConfig};

fn main() {
    env_logger::init();
    let world = World::generate(WorldConfig {
        seed: 239_239_230,
        tile_radius: 25,
    });
    render::run(world);
}
