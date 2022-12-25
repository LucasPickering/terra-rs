mod render;
mod world;

use bevy::prelude::{App, IntoSystemDescriptor, Plugin, Resource};
use bevy_egui::{
    egui::{RichText, Ui, WidgetText},
    EguiPlugin,
};
use std::fmt::Display;
use terra::WorldConfig;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource::<UiState>(UiState::default())
            .add_system(world::world_config_ui)
            .add_system(render::render_config_ui.after(world::world_config_ui));
    }
}

#[derive(Resource)]
struct UiState {
    world_config_text: String,
}

impl Default for UiState {
    fn default() -> Self {
        let world_config_text =
            serde_json::to_string_pretty(&WorldConfig::default()).unwrap();
        Self { world_config_text }
    }
}

/// Create a radio select widget for an enum. There will be one option for each
/// given enum variant.
fn enum_radio_select<'a, T: Display + PartialEq>(
    value: &'a mut T,
    options: impl Iterator<Item = T> + 'a,
) -> impl FnOnce(&mut Ui) + 'a {
    move |ui| {
        for option in options {
            let label = option.to_string();
            ui.radio_value::<T>(value, option, label);
        }
    }
}

fn section(
    heading: impl Into<String>,
    add_contents: impl FnOnce(&mut Ui),
) -> impl FnOnce(&mut Ui) {
    move |ui| {
        ui.group(|ui| {
            ui.label(WidgetText::RichText(RichText::new(heading).heading()));
            add_contents(ui);
        });
    }
}
