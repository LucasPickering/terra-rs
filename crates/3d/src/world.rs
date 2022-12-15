use bevy::{
    prelude::{
        debug, default, info, Added, App, AssetEvent, AssetServer, Assets,
        BuildChildren, Color, Commands, DespawnRecursiveExt, DirectionalLight,
        DirectionalLightBundle, Entity, EventReader, Handle,
        IntoSystemDescriptor, Mesh, PbrBundle, Plugin, Query, Res, ResMut,
        Resource, SpatialBundle, StandardMaterial, Transform, Vec3, With,
    },
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_common_assets::json::JsonAssetPlugin;
use terra::{
    GeoFeature, HasHexPosition, HexDirection, Point2, RenderConfig, Tile,
    TilePoint, VertexDirection, World, WorldConfig, WorldRenderer,
};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(JsonAssetPlugin::<WorldConfig>::new(&["terra.json"]))
            .insert_resource(RenderConfig::default())
            .add_startup_system(load_config)
            .add_startup_system(init_scene)
            .add_system(generate_world)
            // Always delete *before* generating so we don't clobber new stuff
            .add_system(delete_world.before(generate_world))
            .add_system(render_world)
            // Always unrender before re-rendering
            .add_system(unrender_world.before(render_world));
    }
}

#[derive(Resource)]
struct WorldConfigHandle(Handle<WorldConfig>);

fn load_config(mut commands: Commands, asset_server: Res<AssetServer>) {
    let config_handle =
        WorldConfigHandle(asset_server.load("worlds/medium.terra.json"));
    commands.insert_resource(config_handle);
}

/// Add static entities to the scene
fn init_scene(mut commands: Commands) {
    // Directional light emulates the sun
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        // This determines the direction. Actual position doesn't matter though,
        // it's just there to determine rotation from .looking_at
        transform: Transform::from_xyz(500.0, 100.0, 500.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

/// Generate a world and add each tile as its own entity. This just creates the
/// underlying world data, nothing visual.
fn generate_world(
    mut commands: Commands,
    config_assets: Res<Assets<WorldConfig>>,
    mut asset_events: EventReader<AssetEvent<WorldConfig>>,
) {
    // Generate the world
    for event in asset_events.iter() {
        // On first config load, or whenever it's changed, generate a new world
        if let AssetEvent::Created { handle }
        | AssetEvent::Modified { handle } = event
        {
            // We know the asset is loaded by now
            let world_config = config_assets.get(handle).unwrap();

            info!("Generating world");
            let world = World::generate(world_config.to_owned()).unwrap();

            // Spawn each tile as a separate entity
            for tile in world.into_tiles().into_values() {
                commands.spawn(tile);
            }
        }
    }
}

/// Delete the world whenever the config asset changes. This deletes the world
/// data *and* the associated visuals
fn delete_world(
    mut commands: Commands,
    tile_query: Query<Entity, With<Tile>>,
    mut asset_events: EventReader<AssetEvent<WorldConfig>>,
) {
    for event in asset_events.iter() {
        if let AssetEvent::Modified { .. } = event {
            info!("Deleting old world");
            for entity in tile_query.iter() {
                commands.entity(entity).despawn();
            }
        }
    }
}

/// Generate visual meshes and other components for each tile in the world.
/// Run whenever tiles are added to the world.
fn render_world(
    mut commands: Commands,
    tile_query: Query<(Entity, &Tile)>,
    tile_added_query: Query<(), Added<Tile>>,
    render_config: Res<RenderConfig>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Check for anything that should trigger a re-render. We split the tile
    // query into two so we can still access all tiles when config changes
    if !render_config.is_changed() && tile_added_query.is_empty() {
        return;
    }

    debug!("Rendering tiles");
    let renderer =
        WorldRenderer::new(*render_config).expect("Invalid render config");
    let tile_mesh_handle = meshes.add(tile_mesh(&renderer));
    let water_material_handle = materials.add(water_material());

    // For each tile entity, we'll attach additional visual components
    for (entity, tile) in tile_query.iter() {
        let position_2d = renderer.hex_to_screen_space(tile.position());
        let tile_height = renderer.tile_height(tile) as f32;
        let color = renderer.tile_color(tile);

        // We'll add a root transform that provides x/z position. Then add
        // actual visual objects as children. This makes sure transform stay
        // isolated, e.g. tile height doesn't affect water or vice/versa
        commands
            .entity(entity)
            .insert(SpatialBundle {
                transform: Transform::from_xyz(
                    position_2d.x as f32,
                    0.0,
                    position_2d.y as f32,
                ),
                ..default()
            })
            .with_children(|parent| {
                // Spawn the hex mesh
                parent.spawn(PbrBundle {
                    mesh: tile_mesh_handle.clone(),
                    material: materials.add(
                        Color::rgb(color.red, color.green, color.blue).into(),
                    ),
                    transform: Transform::from_scale(
                        [1.0, tile_height, 1.0].into(),
                    ),
                    ..default()
                });

                // A transform to the top-center of the tile
                let transform_tile_top =
                    Transform::from_xyz(0.0, tile_height, 0.0);

                // Add water for ocean tiles
                if tile.is_water_biome() {
                    // Span the distance between the tile and sea level
                    let sea_level_height = renderer.sea_level_height() as f32;
                    let transform = transform_tile_top.with_scale(
                        [1.0, sea_level_height - tile_height, 1.0].into(),
                    );
                    parent.spawn(PbrBundle {
                        mesh: tile_mesh_handle.clone(),
                        material: water_material_handle.clone(),
                        transform,
                        ..default()
                    });
                }

                // Add water for lakes (which is a *feature*, not a biome)
                if tile.features().contains(&GeoFeature::Lake) {
                    // TODO move this math into the renderer or Tile
                    let runoff_height = renderer.elevation_to_height(
                        tile.elevation() + (tile.runoff() / Tile::AREA),
                    ) as f32;
                    // Span the distance between the tile and sea level
                    let transform = transform_tile_top.with_scale(
                        [1.0, runoff_height - tile_height, 1.0].into(),
                    );
                    parent.spawn(PbrBundle {
                        mesh: tile_mesh_handle.clone(),
                        material: water_material_handle.clone(),
                        transform,
                        ..default()
                    });
                }
            });
    }
}

/// Delete the visuals of all tiles, leaving the `Tile` components intact.
/// Runs whenever the render config changes
fn unrender_world(
    mut commands: Commands,
    tile_query: Query<Entity, With<Tile>>,
    render_config: Res<RenderConfig>,
) {
    // TODO this is kinda jank, could be improved:
    // - Create an intermediate WorldEvent type, to consolidate the conditions
    // to trigger a re-generate vs re-render in one place
    // - Figure out how to remove "everyting but tile" without having to
    // keep track of what we create during rendering
    if render_config.is_changed() {
        debug!("Un-rendering tiles");
        for entity in tile_query.iter() {
            commands
                .entity(entity)
                .remove::<SpatialBundle>()
                // Sure hope there aren't any children beside the visuals...
                .despawn_descendants();
        }
    }
}

/// Build a 3d mesh of a hexagonal prism, representing a tile.
///
/// TODO replace this with a cylinder in bevy 0.10
fn tile_mesh(renderer: &WorldRenderer) -> Mesh {
    let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);

    // A tile has 12 vertices, 6 on top and 6 on bottom. In this order:
    // Bot-N, Bot-ENE, Bot-ESE, Bot-S, Bot-WSW, Bot-WNW
    // Top-N, Top-ENE, Top-ESE, Top-S, Top-WSW, Top-WNW
    let vertices_2d: Vec<Point2> = VertexDirection::CLOCKWISE
        .iter()
        .copied()
        .map(|direction| {
            renderer.hex_to_screen_space(TilePoint::ORIGIN.vertex(direction))
        })
        .collect();
    let positions: Vec<[f32; 3]> = vertices_2d
        .iter()
        // Bottom 6
        .map(|point2| [point2.x as f32, 0.0, point2.y as f32])
        // Top 6
        .chain(
            vertices_2d
                .iter()
                .map(|point2| [point2.x as f32, 1.0, point2.y as f32]),
        )
        .collect();

    // Normals are just the vertex vectors, but normalized
    let normals: Vec<Vec3> = positions
        .iter()
        .map(|position| Vec3::new(position[0], 0.0, position[2]).normalize())
        .collect();

    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);

    // REMEMBER: all vertices are specified CLOCKWISE

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

    // A tile is made up of 16 polygons: 2 per side plus 4 on top
    // We *skip* the bottom because it's not visible anyway
    // Each polygon is 3 vertices
    let mut indices: Vec<u32> = vec![
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

fn water_material() -> StandardMaterial {
    let mut material: StandardMaterial =
        Color::rgba(0.078, 0.302, 0.639, 0.5).into();
    material.metallic = 0.0;
    material
}
