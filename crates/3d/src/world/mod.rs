pub mod event;
mod mesh;
pub mod storage;

use crate::world::event::{GenerateWorldEvent, RenderWorldEvent};
use bevy::prelude::{
    debug, default, info, AlphaMode, App, Assets, Color, Commands,
    DespawnRecursiveExt, DirectionalLight, DirectionalLightBundle, Entity,
    EventReader, EventWriter, IntoSystemDescriptor, Mesh, PbrBundle, Plugin,
    Query, Res, ResMut, SpatialBundle, StandardMaterial, Transform, Vec3, With,
};
use terra::{RenderConfig, World, WorldConfig, WorldRenderer};

pub struct WorldPlugin;

impl Plugin for WorldPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(WorldConfig {
            radius: 50, // 100 is too slow right now
            ..default()
        })
        .insert_resource(RenderConfig::default())
        .add_event::<GenerateWorldEvent>()
        .add_event::<RenderWorldEvent>()
        .add_startup_system(init_scene)
        .add_startup_system(init_world)
        .add_system(generate_world)
        // Always delete *before* generating so we don't clobber new stuff
        .add_system(delete_world.before(generate_world))
        .add_system(render_world)
        // Always unrender before re-rendering
        .add_system(unrender_world.before(render_world));
    }
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

/// Trigger initial world generation
fn init_world(mut generate_world_events: EventWriter<GenerateWorldEvent>) {
    generate_world_events.send(GenerateWorldEvent);
}

/// Generate a world and add each tile as its own entity. This just creates the
/// underlying world data, nothing visual.
fn generate_world(
    mut commands: Commands,
    world_config: Res<WorldConfig>,
    mut generate_world_events: EventReader<GenerateWorldEvent>,
    mut render_world_events: EventWriter<RenderWorldEvent>,
) {
    for _ in generate_world_events.iter() {
        info!("Generating world");
        let world = World::generate(world_config.to_owned()).unwrap();

        // Spawn each tile as a separate entity
        commands.spawn(world);

        // We have a new world, it needs to be rendered now
        render_world_events.send(RenderWorldEvent);
    }
}

/// Delete the world before rendering a new one. This deletes the world
/// data *and* the associated visuals
fn delete_world(
    mut commands: Commands,
    world_query: Query<Entity, With<World>>,
    mut generate_world_events: EventReader<GenerateWorldEvent>,
) {
    for _ in generate_world_events.iter() {
        info!("Deleting old world");
        for entity in world_query.iter() {
            // Make sure to delete all _children_ too, which hold a lot of the
            // visuals
            commands.entity(entity).despawn_recursive();
        }
    }
}

/// Generate visual meshes and other components for each tile in the world.
/// Run whenever tiles are added to the world.
fn render_world(
    mut commands: Commands,
    world_query: Query<(Entity, &World)>,
    render_config: Res<RenderConfig>,
    mut render_world_events: EventReader<RenderWorldEvent>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    for _ in render_world_events.iter() {
        debug!("Rendering tiles");
        let renderer =
            WorldRenderer::new(*render_config).expect("Invalid render config");
        let (world_entity, world) = world_query.single();
        let tile_mesh_handle = meshes.add(mesh::build_mesh(world, &renderer));

        // Spawn the hex mesh onto the world
        commands.entity(world_entity).insert(PbrBundle {
            mesh: tile_mesh_handle,
            material: materials.add(StandardMaterial {
                base_color: Color::rgb(1.0, 1.0, 0.0),
                perceptual_roughness: 1.0,
                ..default()
            }),
            ..default()
        });
    }
}

/// Delete the visuals of all tiles, leaving the `Tile` components intact.
/// Runs whenever the render config changes
fn unrender_world(
    mut commands: Commands,
    world_query: Query<Entity, With<World>>,
    mut render_world_events: EventReader<RenderWorldEvent>,
) {
    // TODO Figure out how to remove "everything but tile" without having to
    // keep track of what we create during rendering
    for _ in render_world_events.iter() {
        debug!("Un-rendering tiles");
        for entity in world_query.iter() {
            commands
                .entity(entity)
                .remove::<SpatialBundle>()
                // Sure hope there aren't any children beside the visuals...
                .despawn_descendants();
        }
    }
}

fn water_material() -> StandardMaterial {
    StandardMaterial {
        base_color: Color::rgba(0.078, 0.302, 0.639, 0.5),
        alpha_mode: AlphaMode::Blend,
        metallic: 0.0,
        reflectance: 0.0,
        ..default()
    }
}
