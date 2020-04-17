use crate::world::Tile;
use kiss3d::{
    resource::{Mesh, MeshManager},
    scene::SceneNode,
};
use nalgebra::{Point3, Translation3, Vector3};
use std::{cell::RefCell, rc::Rc};

const TILE_SIDE_LENGTH: f32 = 1.0;
const TILE_INSIDE_RADIUS: f32 = TILE_SIDE_LENGTH * 0.866_025; // approx sqrt(3)/2
const TILE_WIDTH: f32 = TILE_SIDE_LENGTH * 2;
const TILE_MESH_NAME: &str = "tile";

fn get_tile_mesh() -> Mesh {
    Mesh::new(
        vec![
            // Each of these starts at the center, then goes to the top-right,
            // then clockwise from there
            // Bottom
            Point3::new(0.0, 0.0, 0.0),
            Point3::new(TILE_SIDE_LENGTH / 2.0, 0.0, TILE_INSIDE_RADIUS),
            Point3::new(TILE_SIDE_LENGTH, 0.0, 0.0),
            Point3::new(TILE_SIDE_LENGTH / 2.0, 0.0, -TILE_INSIDE_RADIUS),
            Point3::new(-TILE_SIDE_LENGTH / 2.0, 0.0, -TILE_INSIDE_RADIUS),
            Point3::new(-TILE_SIDE_LENGTH, 0.0, 0.0),
            Point3::new(-TILE_SIDE_LENGTH / 2.0, 0.0, TILE_INSIDE_RADIUS),
            // Top
            Point3::new(0.0, 1.0, 0.0),
            Point3::new(TILE_SIDE_LENGTH / 2.0, 1.0, TILE_INSIDE_RADIUS),
            Point3::new(TILE_SIDE_LENGTH, 1.0, 0.0),
            Point3::new(TILE_SIDE_LENGTH / 2.0, 1.0, -TILE_INSIDE_RADIUS),
            Point3::new(-TILE_SIDE_LENGTH / 2.0, 1.0, -TILE_INSIDE_RADIUS),
            Point3::new(-TILE_SIDE_LENGTH, 1.0, 0.0),
            Point3::new(-TILE_SIDE_LENGTH / 2.0, 1.0, TILE_INSIDE_RADIUS),
        ],
        vec![
            // Bottom face
            Point3::new(2, 1, 0),
            Point3::new(3, 2, 0),
            Point3::new(4, 3, 0),
            Point3::new(5, 4, 0),
            Point3::new(6, 5, 0),
            Point3::new(1, 6, 0),
            // Side 1
            Point3::new(1, 2, 8),
            Point3::new(2, 9, 8),
            // Side 2
            Point3::new(2, 3, 9),
            Point3::new(3, 10, 9),
            // Side 3
            Point3::new(3, 4, 10),
            Point3::new(4, 11, 10),
            // Side 4
            Point3::new(4, 5, 11),
            Point3::new(5, 12, 11),
            // Side 5
            Point3::new(5, 6, 12),
            Point3::new(6, 13, 12),
            // Side 6
            Point3::new(6, 1, 13),
            Point3::new(1, 8, 13),
            // Top face
            Point3::new(7, 8, 9),
            Point3::new(7, 9, 10),
            Point3::new(7, 10, 11),
            Point3::new(7, 11, 12),
            Point3::new(7, 12, 13),
            Point3::new(7, 13, 8),
        ],
        None,
        None,
        false,
    )
}

pub fn init_meshes() {
    let mesh = Rc::new(RefCell::new(get_tile_mesh()));
    MeshManager::get_global_manager(move |mm| {
        mm.add(mesh.clone(), TILE_MESH_NAME)
    });
}

pub fn render_tile(parent: &mut SceneNode, tile: &Tile) -> SceneNode {
    let mut node = parent
        .add_geom_with_name(
            TILE_MESH_NAME,
            Vector3::new(1.0, tile.elevation as f32, 1.0),
        )
        .unwrap();

    // Shift tile based on its position
    let translation: (f64, f64) =
        tile.position.get_pixel_pos(TILE_WIDTH as f64);
    node.set_local_translation(Translation3::new(
        translation.0 as f32,
        0.0,
        translation.1 as f32,
    ));

    // Set color
    let color = tile.get_color();
    node.set_color(color.red(), color.green(), color.blue());

    node
}
