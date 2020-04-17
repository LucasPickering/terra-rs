mod render;
mod util;
mod world;

use crate::world::{Biome, HexPoint, Tile};
use kiss3d::{
    camera::{ArcBall, Camera},
    light::Light,
    planar_camera::PlanarCamera,
    post_processing::PostProcessingEffect,
    renderer::Renderer,
    window::{State, Window},
};
use nalgebra::Point3;

struct AppState {
    camera: Box<dyn Camera>,
}

impl State for AppState {
    fn step(&mut self, _: &mut Window) {}

    #[allow(clippy::type_complexity)]
    fn cameras_and_effect_and_renderer(
        &mut self,
    ) -> (
        Option<&mut dyn Camera>,
        Option<&mut dyn PlanarCamera>,
        Option<&mut dyn Renderer>,
        Option<&mut dyn PostProcessingEffect>,
    ) {
        (Some(self.camera.as_mut()), None, None, None)
    }
}

fn main() {
    let mut window = Window::new("Terra");
    render::init_meshes();

    let points = [
        HexPoint::new(0, 0, 0),
        HexPoint::new(1, 0, -1),
        HexPoint::new(1, -1, 0),
        HexPoint::new(0, -1, 1),
        HexPoint::new(-1, 0, 1),
        HexPoint::new(-1, 1, 0),
        HexPoint::new(0, 1, -1),
    ];
    let tiles: Vec<Tile> = points
        .iter()
        .map(|position| Tile {
            position: *position,
            elevation: 10.0,
            humidity: 1.0,
            biome: Biome::Alpine,
        })
        .collect();
    let mut node = window.add_group();
    for tile in &tiles {
        render::render_tile(&mut node, tile);
    }

    window.set_light(Light::StickToCamera);

    let state = AppState {
        camera: Box::new(ArcBall::new_with_frustrum(
            std::f32::consts::PI / 4.0,
            0.1,
            1024.0,
            Point3::new(-50.0, 50.0, -50.0),
            Point3::origin(),
        )),
    };

    window.render_loop(state)
}
