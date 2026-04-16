//! https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html

use bevy::{
    app::{Startup, Update},
    camera::Camera3d,
    ecs::{message::MessageReader, query::With},
    input::{
        mouse::{MouseMotion, MouseWheel},
        ButtonInput,
    },
    prelude::{
        Commands, Component, Mat3, MouseButton, Plugin, Projection, Quat,
        Query, Res, ResMut, Transform, Vec2, Vec3,
    },
    window::{PrimaryWindow, Window},
};
use bevy_egui::input::EguiWantsInput;

const ZOOM_SPEED: f32 = 0.02;
const MIN_RADIUS: f32 = 0.1;

pub struct CameraPlugin;

impl Plugin for CameraPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_systems(Startup, spawn_camera)
            .add_systems(Update, pan_orbit_camera);
    }
}

/// Spawn a camera
fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(200.0, 250.0, -200.0);
    let radius = translation.length();

    commands.spawn((
        Camera3d::default(),
        PanOrbitCamera {
            radius,
            ..Default::default()
        },
        Transform::from_translation(translation)
            .looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
struct PanOrbitCamera {
    /// The "focus point" to orbit around. It is automatically updated when
    /// panning the camera
    pub focus: Vec3,
    pub radius: f32,
    pub upside_down: bool,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            focus: Vec3::ZERO,
            radius: 5.0,
            upside_down: false,
        }
    }
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with
/// right mouse click.
fn pan_orbit_camera(
    windows: Query<&Window, With<PrimaryWindow>>,
    egui_input: ResMut<EguiWantsInput>,
    mut ev_motion: MessageReader<MouseMotion>,
    mut ev_scroll: MessageReader<MouseWheel>,
    input_mouse: Res<ButtonInput<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // If the UI has the cursor captured, ignore all input events
    if egui_input.is_pointer_over_area() || egui_input.wants_pointer_input() {
        return;
    }

    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Left;

    let mut pan = Vec2::ZERO;
    let mut rotation_move = Vec2::ZERO;
    let mut scroll = 0.0;
    let mut orbit_button_changed = false;

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.read() {
            rotation_move += ev.delta;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        for ev in ev_motion.read() {
            pan += ev.delta;
        }
    }
    for ev in ev_scroll.read() {
        scroll += ev.y;
    }
    if input_mouse.just_released(orbit_button)
        || input_mouse.just_pressed(orbit_button)
    {
        orbit_button_changed = true;
    }

    for (mut pan_orbit, mut transform, projection) in query.iter_mut() {
        if orbit_button_changed {
            // only check for upside down when orbiting started or ended this
            // frame if the camera is "upside" down, panning
            // horizontally would be inverted, so invert the input to make it
            // correct
            let up = transform.rotation * Vec3::Y;
            pan_orbit.upside_down = up.y <= 0.0;
        }

        let mut any = false;
        if rotation_move.length_squared() > 0.0 {
            any = true;
            let window = get_primary_window_size(windows);
            let delta_x = {
                let delta =
                    rotation_move.x / window.x * std::f32::consts::PI * 2.0;
                if pan_orbit.upside_down {
                    -delta
                } else {
                    delta
                }
            };
            let delta_y = rotation_move.y / window.y * std::f32::consts::PI;
            let yaw = Quat::from_rotation_y(-delta_x);
            let pitch = Quat::from_rotation_x(-delta_y);
            transform.rotation = yaw * transform.rotation; // rotate around global y axis
            transform.rotation *= pitch; // rotate around local x axis
        } else if pan.length_squared() > 0.0 {
            any = true;
            // make panning distance independent of resolution and FOV,
            let window = get_primary_window_size(windows);
            if let Projection::Perspective(projection) = projection {
                pan *= Vec2::new(
                    projection.fov * projection.aspect_ratio,
                    projection.fov,
                ) / window;
            }
            // translate by local axes
            let right = transform.rotation * Vec3::X * -pan.x;
            let up = transform.rotation * Vec3::Y * pan.y;
            // make panning proportional to distance away from focus point
            let translation = (right + up) * pan_orbit.radius;
            pan_orbit.focus += translation;
        } else if scroll.abs() > 0.0 {
            any = true;
            pan_orbit.radius -= scroll * pan_orbit.radius * ZOOM_SPEED;
            // dont allow zoom to reach zero or you get stuck
            pan_orbit.radius = f32::max(pan_orbit.radius, MIN_RADIUS);
        }

        if any {
            // emulating parent/child to make the yaw/y-axis rotation behave
            // like a turntable parent = x and y rotation
            // child = z-offset
            let rot_matrix = Mat3::from_quat(transform.rotation);
            transform.translation = pan_orbit.focus
                + rot_matrix.mul_vec3(Vec3::new(0.0, 0.0, pan_orbit.radius));
        }
    }
}

fn get_primary_window_size(
    windows: Query<&Window, With<PrimaryWindow>>,
) -> Vec2 {
    let window = windows.iter().next().unwrap();
    Vec2::new(window.width(), window.height())
}
