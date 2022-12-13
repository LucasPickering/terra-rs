mod camera;
mod world;

use crate::{camera::CameraPlugin, world::WorldPlugin};
use bevy::{
    diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},
    log::LogPlugin,
    prelude::{App, AssetPlugin, PluginGroup},
    DefaultPlugins,
};
use bevy_inspector_egui::WorldInspectorPlugin;

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: "info,wgpu_core=warn,wgpu_hal=warn,terra=debug,terra-3d=debug".into(),
            level: bevy::log::Level::DEBUG,
        }).set(AssetPlugin {
            watch_for_changes: true,
            ..Default::default()
        }))
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(LogDiagnosticsPlugin::default())
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(WorldPlugin)
        .add_plugin(CameraPlugin)
        .run();
}
