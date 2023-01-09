use bevy::{
    diagnostic::{Diagnostics, FrameTimeDiagnosticsPlugin},
    prelude::{Res, ResMut},
};
use bevy_egui::{
    egui::{Align2, Area, Color32},
    EguiContext,
};

/// UI for showing bevy diagnostics
pub fn render_diagnostics_ui(
    mut egui_context: ResMut<EguiContext>,
    diagnostics: Res<Diagnostics>,
) {
    Area::new("Diagnostics")
        .anchor(Align2::RIGHT_TOP, (0.0, 0.0))
        .movable(false)
        .show(egui_context.ctx_mut(), |ui| {
            ui.visuals_mut().override_text_color = Some(Color32::WHITE);
            ui.style_mut().wrap = Some(false);

            if let Some(fps) = diagnostics
                .get(FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.average())
            {
                ui.label(format!("{fps:.1} FPS"));
            }
        });
}
