use bevy::{
    prelude::{
        default, App, Assets, Camera3dBundle, Color, Commands, Mesh, PbrBundle,
        PointLight, PointLightBundle, ResMut, StandardMaterial, Transform,
        Vec3,
    },
    render::{mesh::Indices, render_resource::PrimitiveTopology},
    DefaultPlugins,
};
use terra::{
    HasHexPosition, HexDirection, Point2, Tile, VertexDirection, World,
    WorldConfig, WorldRenderer,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let world = World::generate(WorldConfig {
        radius: 0,
        ..default()
    })
    .unwrap();
    let world_renderer = WorldRenderer::new(default()).unwrap();

    for tile in world.tiles().values() {
        commands.spawn(PbrBundle {
            mesh: meshes.add(tile_mesh(&world_renderer, tile)),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        });
    }

    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(-2.0, 2.5, 5.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

fn tile_mesh(world_renderer: &WorldRenderer, tile: &Tile) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // A tile has 12 vertices, 6 on top and 6 on bottom. In this order:
    // Bot-N, Bot-ENE, Bot-ESE, Bot-S, Bot-WSW, Bot-WNW
    // Top-N, Top-ENE, Top-ESE, Top-S, Top-WSW, Top-WNW
    let vertices_2d: Vec<Point2> = VertexDirection::CLOCKWISE
        .iter()
        .copied()
        .map(|direction| {
            world_renderer
                .hex_to_screen_space(tile.position().vertex(direction))
        })
        .collect();
    let vertices: Vec<[f32; 3]> = vertices_2d
        .iter()
        // Bottom 6
        .map(|point2| [point2.x as f32, 0.0, point2.y as f32])
        // Top 6
        .chain(
            vertices_2d
                .iter()
                // TODO use elevation
                .map(|point2| [point2.x as f32, 1.0, point2.y as f32]),
        )
        .collect();
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, vertices);

    // REMEMBER: We use the right-hand rule, so all vertices are
    // COUNTER-CLOCKWISE when looking at the visible side.

    //   Bottom
    //      0
    //     / \
    //    /   \
    //   / T1  \
    //  /       \
    // 5.........1
    // |       ..|
    // | T2  ..  |
    // |   .. T3 |
    // | ..      |
    // 4.........2
    //  \       /
    //   \  T4 /
    //    \   /
    //     \ /
    //      3

    //   Top
    //      6
    //     / \
    //    /   \
    //   / T1  \
    //  /       \
    // 11........7
    // |       ..|
    // | T2  ..  |
    // |   .. T3 |
    // | ..      |
    // 10........8
    //  \       /
    //   \  T4 /
    //    \   /
    //     \ /
    //      9

    // A tile is made up of 20 polygons: 2 per side plus 4 on each end
    // Each polygon is 3 vertices
    let mut indices: Vec<u32> = vec![
        // Bottom - these are CCW instead of CW because we want them
        // to render upside-down
        0, 5, 1, // T1
        1, 5, 4, // T2
        1, 4, 2, // T3
        2, 4, 3, // T4
        // Top
        6, 7, 11, // T1
        7, 10, 11, // T2
        7, 8, 10, // T3
        8, 9, 10, // T4
    ];

    // For each side of the hexagon, draw 2 triangles
    for i in 0..6 {
        // The side has 4 vertices
        let bottom_right = i as u32;
        let bottom_left = (bottom_right + 1) % 6;
        let top_right = bottom_right + 6;
        let top_left = bottom_left + 6;
        // Split the rectangle into two triangles
        indices.extend([
            // Bottom-right triangle
            bottom_right,
            bottom_left,
            top_right,
            // Top-left triangle
            bottom_left,
            top_left,
            top_right,
        ]);
    }

    mesh.set_indices(Some(Indices::U32(indices)));
    mesh
}
