mod camera;
mod world;

use crate::{camera::CameraPlugin, world::WorldPlugin};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log::LogPlugin,
    prelude::{
        default, App, Commands, PluginGroup, PointLight, PointLightBundle,
        Transform,
    },
    DefaultPlugins,
};

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,terra=debug,terra-3d=debug".into(),
            level: bevy::log::Level::DEBUG,
        }))
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WorldPlugin)
        .add_plugin(CameraPlugin)
        .add_startup_system(setup)
        .run();
}

/// set up a simple 3D scene
fn setup(mut commands: Commands) {
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 15000.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(0.0, 250.0, 0.0),
        ..default()
    });
}
