use crate::{ui::enum_radio_select, world::message::RenderWorldMessage};
use bevy::prelude::{DetectChanges, MessageWriter, ResMut};
use bevy_egui::{egui::Window, EguiContexts};
use terra::{RenderConfig, TileLens};

/// UI for editing render config
pub fn render_config_ui(
    mut egui: EguiContexts,
    mut render_config: ResMut<RenderConfig>,
    mut render_world_events: MessageWriter<RenderWorldMessage>,
) {
    Window::new("Render Config").show(egui.ctx_mut().unwrap(), |ui| {
        ui.label("Lens");
        let mut lens = render_config.tile_lens;
        ui.vertical(enum_radio_select(
            &mut lens,
            [
                TileLens::Surface,
                TileLens::Biome,
                TileLens::Elevation,
                TileLens::Humidity,
                TileLens::Runoff,
            ]
            .into_iter(),
        ));
        // Defer updating the render config until we know something actually
        // changed. Otherwise we'll trigger a "change" in the render config
        // on every frame
        if lens != render_config.tile_lens {
            render_config.tile_lens = lens;
        }
    });

    // If we changed the config at all, then trigger a new render
    if render_config.is_changed() {
        render_world_events.write(RenderWorldMessage);
    }
}
