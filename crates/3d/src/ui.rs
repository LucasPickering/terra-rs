use bevy::prelude::{App, Plugin, ResMut};
use bevy_egui::{egui, EguiContext, EguiPlugin};
use terra::{RenderConfig, TileLens};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin).add_system(render_config_ui);
    }
}

/// UI for editing render config
fn render_config_ui(
    mut egui_context: ResMut<EguiContext>,
    mut render_config: ResMut<RenderConfig>,
) {
    egui::Window::new("Render Settings").show(egui_context.ctx_mut(), |ui| {
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
