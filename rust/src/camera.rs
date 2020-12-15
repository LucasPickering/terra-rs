use crate::{config::InputConfig, util::NumRange};
use cgmath::{
    Angle, Deg, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3, Vector2,
    Vector3,
};
use std::f32::consts::PI;

/// Camera near frustum plane
const Z_NEAR: f32 = 0.1;
/// Camera far frustum plane
const Z_FAR: f32 = 1000.0;
// We need to narrow this slight from PI/2 to prevent it flipping over
// because of floating point errorSelf::VERT_ANGLE_RANGE
const PITCH_RANGE: NumRange<f32> = NumRange::new(0.0, PI / 2.0 - 0.01);
const DISTANCE_RANGE: NumRange<f32> = NumRange::new(5.0, 500.0);

/// The different input actions that can be applied to the camera
#[derive(Copy, Clone, Debug)]
pub enum CameraMovement {
    Forward,
    Backward,
    Left,
    Right,
    Up,
    Down,
}

/// Handler for an arcball camera. The camera has a focal point that it pans
/// around. Panning is controlled by the mouse, and the focal point can be moved
/// via keys.
pub struct Camera {
    /// Screen width, in pixels. Updated via [Self::set_size]
    width: u32,
    /// Screen height, in pixels. Updated via [Self::set_size]
    height: u32,
    /// The location that the camera is looking at
    target: Point3<f32>,
    /// The absolute distance between the target and the camera. Must be in the
    /// range [DISTANCE_RANGE].
    distance: f32,
    /// Vertical angle between the target and the camera. 0 means they're on
    /// the same horizontal plane. PI/2 means the camera is directly above the
    /// target. Must be in the range [PITCH_RANGE].
    pitch: Rad<f32>,
    /// Horizontal angle between the target and the camera. 0&PI means they are
    /// aligned parallel to the x axis. PI/2&3PI/2 means they're aligned
    /// parallel to the z axis.
    yaw: Rad<f32>,
    /// Mouse sensitivity, as a ratio of pixels moved to radians turned.
    pixels_to_rads: Rad<f32>,
    /// Vertical camera field of view
    fov: Deg<f32>,
}

impl Camera {
    pub fn new(config: &InputConfig) -> Self {
        Self {
            width: 0,
            height: 0,
            target: Point3::new(0.0, 50.0, 0.0),
            distance: 50.0,
            pitch: Rad(PI / 4.0),
            yaw: Rad(0.0),
            pixels_to_rads: Deg(config.mouse_sensitivity).into(),
            fov: Deg(config.fov),
        }
    }

    /// Calculate current view matrix based on camera position and orientation
    pub fn view(&self) -> Matrix4<f32> {
        // Find the x/z offset from the camera, based on distance and angle
        let xd = self.distance * self.yaw.sin() * self.pitch.cos();
        let zd = self.distance * self.yaw.cos() * self.pitch.cos();
        let yd = self.distance * self.pitch.sin();
        let offset = Vector3::new(xd, yd, zd);
        let eye = self.target + offset;

        Matrix4::look_at(eye, self.target, Vector3::unit_y())
    }

    /// Calculate the projection based on the current window width and height
    pub fn projection(&self) -> Matrix4<f32> {
        cgmath::perspective(
            self.fov,
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
        // Apply movement actions
        let translation: Vector3<f32> = match movement {
            CameraMovement::Forward => -Vector3::unit_z(),
            CameraMovement::Backward => Vector3::unit_z(),
            CameraMovement::Left => -Vector3::unit_x(),
            CameraMovement::Right => Vector3::unit_x(),
            CameraMovement::Up => Vector3::unit_y(),
            CameraMovement::Down => -Vector3::unit_y(),
        } * magnitude;
        // Rotate the translation by our current yaw, so that forward and
        // backward line up with our orientation, not with the x/z axes
        let yaw_quot = Quaternion::from_angle_y(self.yaw);
        let translation = yaw_quot.rotate_vector(translation);

        self.target += translation;
    }

    /// Pan the camera around the focal point
    pub fn pan_camera(&mut self, mouse_delta: Vector2<isize>) {
        let dyaw = self.pixels_to_rads * -mouse_delta.x as f32;
        let dpitch = self.pixels_to_rads * mouse_delta.y as f32;
        self.yaw = (self.yaw + dyaw).normalize();
        self.pitch = Rad(PITCH_RANGE.clamp((self.pitch + dpitch).0));
    }

    /// Zoom in or out, i.e. move closer to or further from the target.
    /// `zoom_in` is true to zoom in (get closer), false to zoom out.
    pub fn zoom_camera(&mut self, zoom_in: bool, magnitude: f32) {
        let next = if zoom_in {
            self.distance - magnitude
        } else {
            self.distance + magnitude
        };
        self.distance = DISTANCE_RANGE.clamp(next);
    }
}
