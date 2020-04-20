use crate::world::{Tile, TileLens, World};
use std::{cell::RefCell, time::Instant};
use three::{
    camera::Camera, color, controls::orbit::Orbit, material, Background,
    Geometry, Object, Text, Window,
};

const TILE_HEIGHT_SCALE: f32 = 0.5;

pub fn init_scene(window: &mut Window, world: &World) {
    window.scene.background = Background::Color(color::CYAN);

    // let geometry = window
    //     .factory
    //     .upload_geometry(Geometry::cylinder(0.5, 0.5, 1.0, 6));

    let light = window.factory.point_light(color::WHITE, 2.0);
    light.set_position([0.0, 100.0, 0.0]);
    window.scene.add(light);

    for (pos, tile) in world.tiles() {
        let height = (tile.elevation() - Tile::ELEVATION_RANGE.min) as f32
            * TILE_HEIGHT_SCALE;
        let geometry = Geometry::cylinder(0.5, 0.5, height, 6);
        let material = material::Lambert {
            color: tile.color(TileLens::Composite).hex(),
            flat: false,
        };
        // let mesh = window.factory.create_instanced_mesh(&geometry, material);
        let mesh = window.factory.mesh(geometry, material);
        let [pos_x, pos_z] = pos.get_pixel_pos(1.0);
        mesh.set_position([pos_x as f32, height / 2.0, pos_z as f32]);
        window.scene.add(&mesh);
    }
}

pub fn init_camera(window: &mut Window) -> (Camera, Orbit) {
    let camera = window.factory.perspective_camera(60.0, 0.1..);
    let orbit = Orbit::builder(&camera)
        .position([-10.0, 50.0, -10.0])
        .target([0.0, 0.0, 0.0])
        .up([0.0, 1.0, 0.0])
        .build();
    (camera, orbit)
}

pub fn init_fps_text(window: &mut Window) -> Text {
    let font = window.factory.load_font_karla();
    let mut text = window.factory.ui_text(&font, "FPS: 0");
    text.set_pos([10.0, 10.0]);
    text.set_font_size(30.0);
    text.set_color(color::BLACK);
    window.scene.add(&text);
    text
}

/// Create a closure that will track frames per second. The closure should be
/// called once per frame, and each time it will emit the current frame rate.
pub fn make_fps_tracker() -> impl Fn() -> f32 {
    let last_frame_start = RefCell::new(Instant::now());

    move || {
        let now = Instant::now();
        let frame_time =
            (now - *(&last_frame_start.borrow() as &Instant)).as_secs_f32();
        let fps = 1.0 / frame_time;
        last_frame_start.replace(now);
        fps
    }
}
