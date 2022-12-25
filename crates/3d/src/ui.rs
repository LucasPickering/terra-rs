use crate::world::event::GenerateWorldEvent;
use bevy::prelude::{
    trace, App, EventWriter, IntoSystemDescriptor, Plugin, ResMut, Resource,
};
use bevy_egui::{
    egui::{self, RichText, Slider, Ui, WidgetText},
    EguiContext, EguiPlugin,
};
use std::{
    fmt::Display,
    ops::{Deref, RangeInclusive},
};
use terra::{Meter3, NoiseFnType, RenderConfig, TileLens, WorldConfig};

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_plugin(EguiPlugin)
            .insert_resource::<UiState>(UiState::default())
            .add_system(world_config_ui)
            .add_system(render_config_ui.after(world_config_ui));
    }
}

/// Standard slider range for normal (0-1) fields
const NORMAL_RANGE: RangeInclusive<f64> = 0.0..=1.0;
/// Standard slider step size for normal (0-1) fields
const NORMAL_STEP: f64 = 0.05;

/// Standard slider range for exponent fields
const EXPONENT_RANGE: RangeInclusive<f64> = 0.0..=3.0;
/// Standard slider step size for exponent fields
const EXPONENT_STEP: f64 = 0.1;

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
        // Directly edit the config JSON
        let mut json_changed = false;
        ui.collapsing("JSON", |ui| {
            json_changed = ui
                .text_edit_multiline(&mut ui_state.world_config_text)
                .changed();
        });

        // Any mutable access to world_config will mark it as changed, so we
        // want to defer mutable acces until we know we actually change
        // something. So we'll copy the config, then re-assign at the end if the
        // copy changed
        let mut controls_config = *world_config;

        // ===== General =====
        ui.scope(section("General", |ui| {
            // TODO seed text field

            ui.add(
                Slider::new(&mut controls_config.radius, 0..=500)
                    .step_by(10.0)
                    .text("World Radius"),
            );
        }));

        // ===== Edge Buffer =====
        ui.scope(section("Edge Buffer", |ui| {
            ui.add(
                Slider::new(
                    &mut controls_config.elevation.edge_buffer_fraction,
                    NORMAL_RANGE,
                )
                .step_by(NORMAL_STEP)
                .text("Fraction"),
            );

            ui.add(
                Slider::new(
                    &mut controls_config.elevation.edge_buffer_exponent,
                    EXPONENT_RANGE,
                )
                .step_by(EXPONENT_STEP)
                .text("Exponent"),
            );
        }));

        // ===== Elevation =====
        ui.scope(section("Elevation", |ui| {
            ui.label("Noise Type");
            ui.vertical(enum_radio_select(
                &mut controls_config.elevation.noise_fn.noise_type,
                [
                    NoiseFnType::BasicMulti,
                    NoiseFnType::Billow,
                    NoiseFnType::Fbm,
                    NoiseFnType::HybridMulti,
                    NoiseFnType::RidgedMulti,
                ]
                .into_iter(),
            ));

            ui.add(
                Slider::new(
                    &mut controls_config.elevation.noise_fn.octaves,
                    1..=10,
                )
                .step_by(1.0)
                .text("Octaves"),
            );

            ui.add(
                Slider::new(
                    &mut controls_config.elevation.noise_fn.frequency,
                    0.1..=5.0,
                )
                .step_by(0.1)
                .text("Frequency"),
            );

            ui.add(
                Slider::new(
                    &mut controls_config.elevation.noise_fn.lacunarity,
                    0.5..=10.0,
                )
                .step_by(0.5)
                .text("Lacunarity"),
            );

            ui.add(
                Slider::new(
                    &mut controls_config.elevation.noise_fn.persistence,
                    0.0..=2.0,
                )
                .step_by(0.1)
                .text("Persistence"),
            );

            ui.add(
                Slider::new(
                    &mut controls_config.elevation.noise_fn.exponent,
                    EXPONENT_RANGE,
                )
                .step_by(EXPONENT_STEP)
                .text("Exponent"),
            );
        }));

        // ===== Rainfall =====
        ui.scope(section("Rainfall", |ui| {
            ui.checkbox(&mut controls_config.rainfall.enabled, "Enabled?");

            // Disable the rest of the controls if rainfall gen is disabled
            ui.add_enabled_ui(controls_config.rainfall.enabled, |ui| {
                ui.add(
                    Slider::new(
                        &mut controls_config.rainfall.evaporation_default.0,
                        0.0..=10.0,
                    )
                    .step_by(0.5)
                    .custom_formatter(format_meter3)
                    .text("Default Evaporation Volume"),
                );

                ui.add(
                    Slider::new(
                        &mut controls_config.rainfall.evaporation_land_scale,
                        NORMAL_RANGE,
                    )
                    .step_by(NORMAL_STEP)
                    .text("Land Evaporation Scale"),
                );

                ui.add(
                    Slider::new(
                        &mut controls_config
                            .rainfall
                            .evaporation_spread_distance,
                        0..=100,
                    )
                    .step_by(5.0)
                    .text("Evaporation Spread Distance"),
                );

                ui.add(
                    Slider::new(
                        &mut controls_config
                            .rainfall
                            .evaporation_spread_exponent,
                        EXPONENT_RANGE,
                    )
                    .step_by(EXPONENT_STEP)
                    .text("Evaporation Spread Exponent"),
                );

                ui.add(
                    Slider::new(
                        &mut controls_config.rainfall.rainfall_fraction_limit,
                        0.0..=0.5,
                    )
                    .step_by(0.05)
                    .text("Rainfall Fraction Limit"),
                );
            });
        }));

        // ===== Geographic Features =====
        ui.scope(section("Geographic Features", |ui| {
            ui.add(
                Slider::new(
                    &mut controls_config
                        .geo_feature
                        .river_runoff_traversed_threshold
                        .0,
                    0.0..=1000.0,
                )
                .step_by(50.0)
                .custom_formatter(format_meter3)
                .text("River Runoff-Traversed Threshold"),
            );
        }));

        // If clicked, trigger a world gen
        if ui.button("Generate World").clicked() {
            generate_world_events.send(GenerateWorldEvent);
        }

        // If any of the controls changed, update the config. We defer this so
        // we don't mark the world config as changed unless it actually did
        if world_config.deref() != &controls_config {
            *world_config = controls_config;

            trace!("World config changed, syncing JSON text: {world_config:?}");
            ui_state.world_config_text =
                serde_json::to_string_pretty(world_config.deref()).unwrap();
        } else if json_changed {
            // If the JSON text was changed, try to deserialize it and update
            // the config. If deserialization fails, assume the user is still
            // making changes so just leave it be.
            if let Ok(deserialized_config) =
                serde_json::from_str(&ui_state.world_config_text)
            {
                trace!(
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
        // Defer updating the render config until we know something actuall
        // changed. Otherwise we'll trigger a "change" in the render config
        // on every frame
        if lens != render_config.tile_lens {
            render_config.tile_lens = lens;
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
            ui.label(WidgetText::RichText(RichText::new(heading).heading()));
            add_contents(ui);
        });
    }
}

/// Format a Meter3 (cubic meter) as a string
fn format_meter3(value: f64, _: RangeInclusive<usize>) -> String {
    Meter3(value).to_string()
}
