mod render;
mod util;
mod world;

use crate::world::{World, WorldConfig};
use three::Window;

fn main() {
    env_logger::init();
    let mut window = Window::new("Terra");

    let world_config = WorldConfig::load().unwrap();
    let world = World::generate(world_config);

    render::init_scene(&mut window, &world);
    let (camera, mut orbit) = render::init_camera(&mut window);

    while window.update() {
        window.render(&camera);
        orbit.update(&window.input);
    }
}
