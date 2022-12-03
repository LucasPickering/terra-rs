mod world;

use crate::world::WorldPlugin;
use bevy::{
    prelude::{
        default, App, Camera3dBundle, Commands, PointLight, PointLightBundle,
        Transform, Vec3,
    },
    DefaultPlugins,
};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldPlugin)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 250.0, 0.0),
        ..default()
    });
    // camera
    commands.spawn(Camera3dBundle {
        transform: Transform::from_xyz(200.0, 250.0, -200.0)
            .looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}
