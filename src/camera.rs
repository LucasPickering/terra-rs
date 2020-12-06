use std::f32::consts::PI;

use cgmath::{
    InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3,
};
use log::debug;

const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 1000.0;

/// The different input actions that can be applied to the camera
#[derive(Copy, Clone, Debug)]
pub enum CameraAction {
    MoveForward,
    MoveBackward,
    MoveLeft,
    MoveRight,
    RotateUp,
    RotateDown,
    RotateLeft,
    RotateRight,
}

pub struct Camera {
    // view: Matrix4<f32>,
    /// Eye location, in 3D space
    position: Point3<f32>,
    /// Vertical angle. 0 is level (where the x/z plane is horizontal)
    pitch: Rad<f32>,
    /// Rotation about the y axis. 0 is looking parallel to the x axis
    yaw: Rad<f32>,
    projection: Matrix4<f32>,
}

impl Camera {
    pub fn new() -> Self {
        Self {
            position: Point3::new(2.0, 2.0, 2.0),
            pitch: Rad(-PI / 4.0),
            yaw: Rad(-3.0 * PI / 4.0),
            projection: cgmath::perspective(
                FOVY, 1.0, // TODO
                Z_NEAR, Z_FAR,
            ),
        }
    }

    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_dir(
            self.position,
            Vector3::new(
                self.yaw.0.cos(),
                self.pitch.0.sin(),
                self.yaw.0.sin(),
            )
            .normalize(),
            Vector3::unit_y(),
        )
    }

    pub fn projection(&self) -> Matrix4<f32> {
        self.projection
    }

    pub fn apply_action(&mut self, action: CameraAction, magnitude: f32) {
        // Apply rotation actions
        let (pitch, yaw): (Rad<f32>, Rad<f32>) = match action {
            CameraAction::RotateUp => (Rad(1.0), Rad(0.0)),
            CameraAction::RotateDown => (Rad(-1.0), Rad(0.0)),
            CameraAction::RotateLeft => (Rad(0.0), Rad(-1.0)),
            CameraAction::RotateRight => (Rad(0.0), Rad(1.0)),
            _ => (Rad(0.0), Rad(0.0)),
        };
        let (pitch, yaw) = (pitch * magnitude, yaw * magnitude);
        self.pitch += pitch;
        self.yaw += yaw;

        // Apply movement actions
        let translation: Vector3<f32> = match action {
            CameraAction::MoveForward => Vector3::unit_z(),
            CameraAction::MoveBackward => Vector3::unit_z() * -1.0,
            CameraAction::MoveLeft => Vector3::unit_x(),
            CameraAction::MoveRight => Vector3::unit_x() * -1.0,
            _ => Vector3::new(0.0, 0.0, 0.0),
        } * magnitude;
        // Rotate the translation by our current yaw, so that forward and
        // backward line up with our orientation, not with the x/z axes
        let yaw_quot = Quaternion::from_angle_y(self.yaw);
        let translation = yaw_quot.rotate_vector(translation);
        debug!("translation={:?}", translation);

        self.position += translation;

        debug!(
            "position={:?}; pitch={:?}; yaw={:?}",
            self.position, self.pitch, self.yaw
        );
    }
}
