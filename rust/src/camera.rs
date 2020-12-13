use crate::util::NumRange;
use cgmath::{
    Angle, Matrix4, Point3, Quaternion, Rad, Rotation, Rotation3, Vector2,
    Vector3,
};
use std::f32::consts::PI;

const FOVY: Rad<f32> = Rad(std::f32::consts::FRAC_PI_2);
const Z_NEAR: f32 = 0.1;
const Z_FAR: f32 = 1000.0;

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
    width: u32,
    height: u32,
    target: Point3<f32>,
    distance: f32,
    pitch: Rad<f32>,
    yaw: Rad<f32>,
}

impl Camera {
    const HORIZ_ANGLE_RANGE: NumRange<f32> = NumRange::new(0.0, PI * 2.0);
    // We need to narrow this slight from PI/2 to prevent it flipping over
    // because of floating point errorSelf::VERT_ANGLE_RANGE
    const VERT_ANGLE_RANGE: NumRange<f32> = NumRange::new(0.0, PI / 2.0 - 0.01);

    pub fn new() -> Self {
        Self {
            width: 0,
            height: 0,
            target: Point3::new(0.0, 50.0, 0.0),
            distance: 50.0,
            pitch: Rad(PI / 4.0),
            yaw: Rad(0.0),
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
        // Map the pixel delta to the range of possible values. A full movement
        // of the screen width will do a 360. Full height movement will do
        // 0->90.
        let dyaw = -NumRange::new(0.0, self.width as f32)
            .map_to(&Self::HORIZ_ANGLE_RANGE.zeroed(), mouse_delta.x as f32);
        let dpitch = NumRange::new(0.0, self.height as f32)
            .map_to(&Self::VERT_ANGLE_RANGE.zeroed(), mouse_delta.y as f32);
        self.yaw = (self.yaw + Rad(dyaw)).normalize();
        self.pitch =
            Rad(Self::VERT_ANGLE_RANGE.clamp((self.pitch + Rad(dpitch)).0));
    }
}
