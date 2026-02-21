//! Orbital path geometry for rendering orbit lines.
//!
//! The actual position computation lives in [`CelestialBody::position_at()`].
//! This module only generates the static circular path vertices.

use glam::Vec3;

use crate::constants::{AU_TO_DISPLAY, ORBIT_SEGMENTS};

/// Generate the vertices for a circular orbit line in 3D.
/// Returns a Vec of Vec3 positions forming a closed loop.
pub fn generate_orbit_path(
    semi_major_axis_au: f64,
    inclination_rad: f64,
) -> Vec<Vec3> {
    let display_distance = semi_major_axis_au as f32 * AU_TO_DISPLAY;
    let cos_i = inclination_rad.cos() as f32;
    let sin_i = inclination_rad.sin() as f32;

    (0..=ORBIT_SEGMENTS)
        .map(|i| {
            let angle = (i as f64 / ORBIT_SEGMENTS as f64) * std::f64::consts::TAU;
            let cos_a = angle.cos() as f32;
            let sin_a = angle.sin() as f32;

            Vec3::new(
                display_distance * cos_a,
                display_distance * sin_a * sin_i,
                display_distance * sin_a * cos_i,
            )
        })
        .collect()
}
