use crate::{
    ui::{enum_radio_select, section},
    world::event::GenerateWorldEvent,
};
use bevy::prelude::{trace, EventWriter, Res, ResMut, Resource};
use bevy_egui::{
    egui::{Slider, Ui, Window},
    EguiContext,
};
use std::ops::{Deref, RangeInclusive};
use terra::{Meter3, NoiseFnType, WorldConfig};

/// Standard slider range for normal (0-1) fields
const NORMAL_RANGE: RangeInclusive<f64> = 0.0..=1.0;
/// Standard slider step size for normal (0-1) fields
const NORMAL_STEP: f64 = 0.05;

/// Standard slider range for exponent fields
const EXPONENT_RANGE: RangeInclusive<f64> = 0.0..=3.0;
/// Standard slider step size for exponent fields
const EXPONENT_STEP: f64 = 0.1;

#[derive(Default, Resource)]
pub struct WorldConfigUiState {
    /// Raw JSON editor for the world config
    config_json: String,
}

impl WorldConfigUiState {
    /// Reset UI state to match the current world congig
    fn reset(&mut self, world_config: &WorldConfig) {
        self.config_json = serde_json::to_string_pretty(world_config).unwrap();
    }
}

/// Initialize UI state to match the initial world config
pub(super) fn init_world_config_ui(
    world_config: Res<WorldConfig>,
    mut ui_state: ResMut<WorldConfigUiState>,
) {
    ui_state.reset(&world_config);
}

/// UI for editing world config
pub(super) fn world_config_ui(
    mut egui_context: ResMut<EguiContext>,
    mut world_config: ResMut<WorldConfig>,
    mut generate_world_events: EventWriter<GenerateWorldEvent>,
    mut ui_state: ResMut<WorldConfigUiState>,
) {
    // Did the JSON editor change on this frame?
    let mut json_changed = false;

    // Any mutable access to world_config will mark it as changed, so we want to
    // defer mutable acces until we know we actually change something. So we'll
    // copy the config, then re-assign at the end if the copy changed
    let mut controls_config = world_config.clone();

    Window::new("World Config").vscroll(true).show(
        egui_context.ctx_mut(),
        |ui| {
            // Directly edit the config JSON
            ui.collapsing("JSON", |ui| {
                json_changed =
                    ui.text_edit_multiline(&mut ui_state.config_json).changed();
            });

            // Render all the controls that can edit the config
            controls_ui(ui, &mut controls_config);

            ui.horizontal(|ui| {
                // Button to reset config to default value
                if ui.button("Reset to Default").clicked() {
                    controls_config = WorldConfig::default();
                }

                // Button to trigger a world gen
                if ui.button("Generate World").clicked() {
                    generate_world_events.send(GenerateWorldEvent);
                }
            });
        },
    );

    // If any of the controls changed, update the config. We defer this
    // so we don't mark the world config as changed unless it actually
    // did
    if world_config.deref() != &controls_config {
        *world_config = controls_config;
        trace!("World config changed, syncing JSON text: {world_config:?}");
        // Reset JSON editor to match current config
        ui_state.reset(&world_config);
    } else if json_changed {
        // If the JSON text was changed, try to deserialize it and
        // update the config. If deserialization fails, assume the user
        // is still making changes so just leave it be.
        if let Ok(deserialized_config) =
            serde_json::from_str(&ui_state.config_json)
        {
            trace!(
                "JSON text changed, using deserialized config: {:?}",
                deserialized_config
            );
            *world_config = deserialized_config;
        }
    }
}

/// Render all the controls for editing individual config fields
fn controls_ui(ui: &mut Ui, world_config: &mut WorldConfig) {
    // ===== General =====
    ui.scope(section("General", |ui| {
        // This is a little funky - we need to get a textual version of the
        // current seed so it can be edited in a text box. If the seed is an
        // int, we'll need to convert it to a string. Then we'll convert it
        // back after editing, if it's still an int. This does some cloning
        // that probably shouldn't be necessary, but who cares.
        ui.label("Seed");
        let mut seed_text = match &world_config.seed {
            terra::Seed::Int(seed) => seed.to_string(),
            terra::Seed::Text(seed) => seed.clone(),
        };
        if ui.text_edit_singleline(&mut seed_text).changed() {
            // Convert text back to a seed. This will parse it as an int if
            // possible, then fall back to storing it as a string
            world_config.seed = seed_text.as_str().into();
        }

        ui.add(
            Slider::new(&mut world_config.radius, 0..=500)
                .step_by(10.0)
                .text("World Radius"),
        );
    }));

    // ===== Edge Buffer =====
    ui.scope(section("Edge Buffer", |ui| {
        ui.add(
            Slider::new(
                &mut world_config.elevation.edge_buffer_fraction,
                NORMAL_RANGE,
            )
            .step_by(NORMAL_STEP)
            .text("Fraction"),
        );

        ui.add(
            Slider::new(
                &mut world_config.elevation.edge_buffer_exponent,
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
            &mut world_config.elevation.noise_fn.noise_type,
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
            Slider::new(&mut world_config.elevation.noise_fn.octaves, 1..=10)
                .step_by(1.0)
                .text("Octaves"),
        );

        ui.add(
            Slider::new(
                &mut world_config.elevation.noise_fn.frequency,
                0.1..=5.0,
            )
            .step_by(0.1)
            .text("Frequency"),
        );

        ui.add(
            Slider::new(
                &mut world_config.elevation.noise_fn.lacunarity,
                0.5..=10.0,
            )
            .step_by(0.5)
            .text("Lacunarity"),
        );

        ui.add(
            Slider::new(
                &mut world_config.elevation.noise_fn.persistence,
                0.0..=2.0,
            )
            .step_by(0.1)
            .text("Persistence"),
        );

        ui.add(
            Slider::new(
                &mut world_config.elevation.noise_fn.exponent,
                EXPONENT_RANGE,
            )
            .step_by(EXPONENT_STEP)
            .text("Exponent"),
        );
    }));

    // ===== Rainfall =====
    ui.scope(section("Rainfall", |ui| {
        ui.checkbox(&mut world_config.rainfall.enabled, "Enabled?");

        // Disable the rest of the controls if rainfall gen is disabled
        ui.add_enabled_ui(world_config.rainfall.enabled, |ui| {
            ui.add(
                Slider::new(
                    &mut world_config.rainfall.evaporation_default.0,
                    0.0..=10.0,
                )
                .step_by(0.5)
                .custom_formatter(format_meter3)
                .text("Default Evaporation Volume"),
            );

            ui.add(
                Slider::new(
                    &mut world_config.rainfall.evaporation_land_scale,
                    NORMAL_RANGE,
                )
                .step_by(NORMAL_STEP)
                .text("Land Evaporation Scale"),
            );

            ui.add(
                Slider::new(
                    &mut world_config.rainfall.evaporation_spread_distance,
                    0..=100,
                )
                .step_by(5.0)
                .text("Evaporation Spread Distance"),
            );

            ui.add(
                Slider::new(
                    &mut world_config.rainfall.evaporation_spread_exponent,
                    EXPONENT_RANGE,
                )
                .step_by(EXPONENT_STEP)
                .text("Evaporation Spread Exponent"),
            );

            ui.add(
                Slider::new(
                    &mut world_config.rainfall.rainfall_fraction_limit,
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
                &mut world_config
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
}

/// Format a Meter3 (cubic meter) as a string
fn format_meter3(value: f64, _: RangeInclusive<usize>) -> String {
    Meter3(value).to_string()
}
