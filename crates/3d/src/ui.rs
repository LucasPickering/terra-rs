use crate::world::event::GenerateWorldEvent;
use bevy::prelude::{
    debug, App, EventWriter, IntoSystemDescriptor, Plugin, ResMut, Resource,
};
use bevy_egui::{
    egui::{self, Slider},
    EguiContext, EguiPlugin,
};
use std::ops::Deref;
use terra::{RenderConfig, TileLens, WorldConfig};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource::<UiState>(UiState::default())
            .add_system(world_config_ui)
            .add_system(render_config_ui.after(world_config_ui));
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

/// UI for editing world config
fn world_config_ui(
    mut egui_context: ResMut<EguiContext>,
    mut world_config: ResMut<WorldConfig>,
    mut generate_world_events: EventWriter<GenerateWorldEvent>,
    mut ui_state: ResMut<UiState>,
) {
    egui::Window::new("World Config").show(egui_context.ctx_mut(), |ui| {
        ui.label("JSON");
        let json_text_edit =
            ui.text_edit_multiline(&mut ui_state.world_config_text);

        // Any mutable access to world_config will mark it as changed, so we
        // want to defer mutable acces until we know we actually change
        // something. So we'll copy the config, then re-assign at the end if the
        // copy changed
        let mut controls_config = *world_config;
        ui.label("General");
        ui.add(
            Slider::new(&mut controls_config.radius, 0..=500)
                .step_by(10.0)
                .text("World Radius"),
        );

        ui.label("Edge Buffer");
        ui.add(
            Slider::new(
                &mut controls_config.elevation.edge_buffer_exponent,
                0.0..=3.0,
            )
            .step_by(0.1)
            .text("Edge Buffer Fraction"),
        );

        let generate_button = ui.button("Generate World");

        // If clicked, trigger a world gen
        if generate_button.clicked() {
            generate_world_events.send(GenerateWorldEvent);
        }

        // If any of the controls changed, update the config. We defer this so
        // we don't mark the world config as changed unless it actually did
        if world_config.deref() != &controls_config {
            *world_config = controls_config;

            debug!("World config changed, syncing JSON text: {world_config:?}");
            ui_state.world_config_text =
                serde_json::to_string_pretty(world_config.deref()).unwrap();
        } else if json_text_edit.changed() {
            // If the JSON text was changed, try to deserialize it and update
            // the config. If deserialization fails, assume the user is still
            // making changes so just leave it be.
            if let Ok(deserialized_config) =
                serde_json::from_str(&ui_state.world_config_text)
            {
                debug!(
                    "JSON text changed, using deserialized config: {:?}",
                    deserialized_config
                );
                *world_config = deserialized_config;
            }
        }
    });
}

/// UI for editing render config
fn render_config_ui(
    mut egui_context: ResMut<EguiContext>,
    mut render_config: ResMut<RenderConfig>,
) {
    egui::Window::new("Render Config").show(egui_context.ctx_mut(), |ui| {
        ui.label("Lens");
        ui.vertical(|ui| {
            let mut lens = render_config.tile_lens;
            ui.radio_value(&mut lens, TileLens::Surface, "Surface");
            ui.radio_value(&mut lens, TileLens::Biome, "Biome");
            ui.radio_value(&mut lens, TileLens::Elevation, "Elevation");
            ui.radio_value(&mut lens, TileLens::Humidity, "Humidity");
            ui.radio_value(&mut lens, TileLens::Runoff, "Runoff");

            // Defer updating the render config until we know something actuall
            // changed. Otherwise we'll trigger a "change" in the render config
            // on every frame
            if lens != render_config.tile_lens {
                render_config.tile_lens = lens;
            }
        });
    });
}
