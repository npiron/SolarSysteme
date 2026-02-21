//! Orbital camera controller.
//!
//! Uses spherical coordinates (theta, phi, distance) to orbit around a
//! target point. All magic numbers come from [`constants`] so they can
//! be tuned in one place.

use glam::{Mat4, Vec3};

use crate::constants::*;

/// Orbital camera that looks at a target from spherical coordinates.
pub struct Camera {
    /// Horizontal angle in radians.
    pub theta: f32,
    /// Vertical angle in radians (clamped to avoid gimbal lock).
    pub phi: f32,
    /// Distance from target.
    pub distance: f32,
    /// Point the camera orbits around.
    pub target: Vec3,
    /// Minimum zoom distance.
    pub min_distance: f32,
    /// Maximum zoom distance.
    pub max_distance: f32,
    /// Field of view in radians.
    pub fov: f32,
    /// Viewport aspect ratio (width / height).
    pub aspect: f32,
    /// Desired target for smooth transition (`None` when no animation is active).
    pub lerp_target: Option<Vec3>,
    /// Desired orbit distance for smooth transition (`None` when no animation is active).
    pub lerp_distance: Option<f32>,
}

impl Camera {
    pub fn new(aspect: f32) -> Self {
        Self {
            theta: CAMERA_THETA,
            phi: CAMERA_PHI,
            distance: CAMERA_DISTANCE,
            target: Vec3::ZERO,
            min_distance: CAMERA_MIN_DISTANCE,
            max_distance: CAMERA_MAX_DISTANCE,
            fov: CAMERA_FOV_DEGREES.to_radians(),
            aspect,
            lerp_target: None,
            lerp_distance: None,
        }
    }

    /// Camera world position derived from spherical coordinates.
    pub fn eye_position(&self) -> Vec3 {
        let x = self.distance * self.phi.cos() * self.theta.cos();
        let y = self.distance * self.phi.sin();
        let z = self.distance * self.phi.cos() * self.theta.sin();
        self.target + Vec3::new(x, y, z)
    }

    /// View matrix (look-at, right-handed).
    pub fn view_matrix(&self) -> Mat4 {
        Mat4::look_at_rh(self.eye_position(), self.target, Vec3::Y)
    }

    /// Perspective projection matrix.
    pub fn projection_matrix(&self) -> Mat4 {
        Mat4::perspective_rh_gl(self.fov, self.aspect, CAMERA_NEAR, CAMERA_FAR)
    }

    /// Rotate from mouse/touch drag deltas (pixels).
    pub fn rotate(&mut self, dx: f32, dy: f32) {
        self.theta -= dx * ROTATE_SENSITIVITY;
        self.phi += dy * ROTATE_SENSITIVITY;
        self.phi = self.phi.clamp(-PHI_CLAMP, PHI_CLAMP);
    }

    /// Zoom from scroll-wheel delta.
    pub fn zoom(&mut self, delta: f32) {
        self.distance *= 1.0 + delta * ZOOM_SENSITIVITY;
        self.distance = self.distance.clamp(self.min_distance, self.max_distance);
    }

    /// Update aspect ratio (on canvas resize).
    pub fn set_aspect(&mut self, aspect: f32) {
        self.aspect = aspect;
    }

    /// Begin a smooth camera transition to a new `target` point and orbit `distance`.
    pub fn set_target(&mut self, target: Vec3, distance: f32) {
        self.lerp_target = Some(target);
        self.lerp_distance = Some(distance.clamp(self.min_distance, self.max_distance));
    }

    /// Advance any active camera-transition animations.
    ///
    /// Call once per frame with the real elapsed time in seconds.
    pub fn update_transition(&mut self, dt: f32) {
        let alpha = (dt * CAMERA_LERP_SPEED).min(1.0);

        if let Some(tgt) = self.lerp_target {
            self.target = self.target.lerp(tgt, alpha);
            if self.target.distance(tgt) < 0.01 {
                self.target = tgt;
                self.lerp_target = None;
            }
        }

        if let Some(dist) = self.lerp_distance {
            self.distance += (dist - self.distance) * alpha;
            if (self.distance - dist).abs() < 0.01 {
                self.distance = dist;
                self.lerp_distance = None;
            }
        }
    }
}
