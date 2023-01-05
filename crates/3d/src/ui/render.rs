use crate::{ui::enum_radio_select, world::event::RenderWorldEvent};
use bevy::prelude::{DetectChanges, EventWriter, ResMut};
use bevy_egui::{egui::Window, EguiContext};
use terra::{RenderConfig, TileLens};

/// UI for editing render config
pub fn render_config_ui(
    mut egui_context: ResMut<EguiContext>,
    mut render_config: ResMut<RenderConfig>,
    mut render_world_events: EventWriter<RenderWorldEvent>,
) {
    Window::new("Render Config").show(egui_context.ctx_mut(), |ui| {
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
        render_world_events.send(RenderWorldEvent);
    }
}
