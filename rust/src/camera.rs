use crate::util::NumRange;
use cgmath::{
    InnerSpace, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3, Vector3,
};
use log::debug;
use serde::Deserialize;
use std::f32::consts::PI;

const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 1000.0;

/// The different input actions that can be applied to the camera
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
    /* RotateUp,
     * RotateDown,
     * RotateLeft,
     * RotateRight, */
}

pub struct Camera {
    width: u32,
    height: u32,
    /// Eye location, in 3D space
    position: Point3<f32>,
    /// Vertical angle. 0 is level (where the x/z plane is horizontal)
    pitch: Rad<f32>,
    /// Rotation about the y axis. 0 is looking parallel to the x axis
    yaw: Rad<f32>,
}

impl Camera {
    const CAMERA_SPACE: NumRange<f32> = NumRange::new(-1.0, 1.0);

    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            position: Point3::new(0.0, 25.0, 0.0),
            pitch: Rad(-PI / 4.0),
            yaw: Rad(0.0),
        }
    }

    /// Calculate current view matrix based on camera position and orientation
    pub fn view(&self) -> Matrix4<f32> {
        Matrix4::look_at_dir(
            self.position,
            Vector3::new(
                self.yaw.0.sin(),
                self.pitch.0.sin(),
                self.yaw.0.cos(),
            )
            .normalize(),
            Vector3::unit_y(),
        )
    }

    /// Calculate the projection based on the current window width and height
    pub fn projection(&self) -> Matrix4<f32> {
        cgmath::perspective(
            FOVY,
            self.width as f32 / self.height as f32,
            Z_NEAR,
            Z_FAR,
        )
    }

    /// Set the camera's width/height, in pixels
    pub fn set_size(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
    }

    /// Apply a camera movement action
    pub fn move_camera(&mut self, movement: CameraMovement, magnitude: f32) {
        // // Apply rotation actions
        // let (pitch, yaw): (Rad<f32>, Rad<f32>) = match action {
        //     CameraMovement::RotateUp => (Rad(1.0), Rad(0.0)),
        //     CameraMovement::RotateDown => (Rad(-1.0), Rad(0.0)),
        //     CameraMovement::RotateLeft => (Rad(0.0), Rad(1.0)),
        //     CameraMovement::RotateRight => (Rad(0.0), Rad(-1.0)),
        //     _ => (Rad(0.0), Rad(0.0)),
        // };
        // let (pitch, yaw) = (pitch * magnitude, yaw * magnitude);
        // self.pitch += pitch;
        // self.yaw += yaw;

        // Apply movement actions
        let translation: Vector3<f32> = match movement {
            CameraMovement::Forward => Vector3::unit_z(),
            CameraMovement::Backward => Vector3::unit_z() * -1.0,
            CameraMovement::Left => Vector3::unit_x(),
            CameraMovement::Right => Vector3::unit_x() * -1.0,
            CameraMovement::Up => Vector3::unit_y(),
            CameraMovement::Down => Vector3::unit_y() * -1.0,
            _ => Vector3::new(0.0, 0.0, 0.0),
        } * magnitude;
        // Rotate the translation by our current yaw, so that forward and
        // backward line up with our orientation, not with the x/z axes
        let yaw_quot = Quaternion::from_angle_y(self.yaw);
        let translation = yaw_quot.rotate_vector(translation);

        self.position += translation;
    }

    /// Turn the camera to point at the current mouse x/y
    pub fn pan_camera(&mut self, x: isize, y: isize) {
        // Convert from pixels to [-1,1]
        let x = NumRange::new(0.0, self.width as f32)
            .map_to(&Self::CAMERA_SPACE, x as f32);
        let y = NumRange::new(0.0, self.height as f32)
            .map_to(&Self::CAMERA_SPACE, y as f32);
        let z = (x * x + y * y).sqrt();
    }
}
