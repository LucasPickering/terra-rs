mod render_config;
mod world_config;

use crate::ui::world_config::WorldConfigUiState;
use bevy::{
    app::Startup,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    ecs::{schedule::IntoScheduleConfigs, system::Res},
    prelude::{App, Plugin},
};
use bevy_egui::{
    egui::{Align2, Area, Color32, RichText, TextWrapMode, Ui, WidgetText},
    EguiContexts, EguiPlugin, EguiPrimaryContextPass,
};
use std::fmt::Display;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugins(EguiPlugin::default())
            .insert_resource(WorldConfigUiState::default())
            .add_systems(Startup, world_config::init_world_config_ui)
            .add_systems(EguiPrimaryContextPass, render_diagnostics_ui)
            .add_systems(
                EguiPrimaryContextPass,
                (
                    world_config::world_config_ui,
                    render_config::render_config_ui,
                )
                    .chain(),
            );
    }
}

/// UI for showing bevy diagnostics
fn render_diagnostics_ui(
    mut egui: EguiContexts,
    diagnostics: Res<DiagnosticsStore>,
) {
    Area::new("Diagnostics".into())
        .anchor(Align2::RIGHT_TOP, (0.0, 0.0))
        .movable(false)
        .show(egui.ctx_mut().unwrap(), |ui| {
            ui.visuals_mut().override_text_color = Some(Color32::WHITE);
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);

            if let Some(fps) = diagnostics
                .get(&FrameTimeDiagnosticsPlugin::FPS)
                .and_then(|fps| fps.average())
            {
                ui.label(format!("{fps:.1} FPS"));
            }
        });
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
            ui.label(WidgetText::RichText(
                RichText::new(heading).heading().into(),
            ));
            add_contents(ui);
        });
    }
}
