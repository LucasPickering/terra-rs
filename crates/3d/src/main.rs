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
use std::io::{self, Write};

const LOG_FILTER: &str =
    "info,wgpu_core=warn,wgpu_hal=warn,terra=debug,terra-3d=debug";

fn main() {
    #[cfg(target_arch = "wasm32")]
    console_error_panic_hook::set_once();

    // Opening the window steals focus. So in dev, ask for permission to open
    // it. This allows the watching task to compile on changes without
    // constantly stealing focus.
    if cfg!(debug_assertions) {
        print!("Press Enter to open window");
        let _ = io::stdout().flush();
        let _ = io::stdin().read_line(&mut String::new());
    }

    App::new()
        .add_plugins(DefaultPlugins.set(LogPlugin {
            filter: LOG_FILTER.into(),
            level: bevy::log::Level::DEBUG,
            ..Default::default()
        }))
        .add_plugins(FrameTimeDiagnosticsPlugin::default())
        .add_plugins(UiPlugin)
        .add_plugins(WorldPlugin)
        .add_plugins(CameraPlugin)
        .run();
}
