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
    let mut fps_text = render::init_fps_text(&mut window);

    let track_fps = render::make_fps_tracker();
    while window.update() {
        let fps = track_fps();
        fps_text.set_text(format!("FPS: {:.0}", fps));

        orbit.update(&window.input);
        window.render(&camera);
    }
}
