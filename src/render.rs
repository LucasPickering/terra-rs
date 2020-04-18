use crate::world::{Tile, TileLens, World};
use three::{
    camera::Camera, color, controls::orbit::Orbit, material, Background,
    Geometry, Object, Window,
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
        let height = (tile.elevation - Tile::ELEVATION_RANGE.min) as f32
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
