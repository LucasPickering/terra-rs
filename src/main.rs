mod render;
mod util;
mod world;

use crate::world::{World, WorldConfig};

fn main() {
    env_logger::init();
    let world = World::generate(WorldConfig {
        seed: 4,
        tile_radius: 16,
    });
    render::run(world);
}
