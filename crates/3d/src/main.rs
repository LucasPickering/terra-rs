mod camera;
mod ui;
mod world;

use crate::{camera::CameraPlugin, ui::UiPlugin, world::WorldPlugin};
use bevy::{
    diagnostic::FrameTimeDiagnosticsPlugin,
    log::LogPlugin,
    prelude::{App, PluginGroup},
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
        .add_plugin(FrameTimeDiagnosticsPlugin::default())
        .add_plugin(UiPlugin)
        .add_plugin(WorldPlugin)
        .add_plugin(CameraPlugin)
        .run();
}
