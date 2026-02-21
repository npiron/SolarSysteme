use glam::Vec3;

use crate::constants::AU_TO_DISPLAY;

/// Represents a celestial body in the solar system.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CelestialBody {
    /// Display name
    pub name: &'static str,
    /// RGB color (0.0–1.0)
    pub color: [f32; 3],
    /// Visual display radius (log-scaled for visibility)
    pub display_radius: f32,
    /// Real equatorial radius in km (for info display)
    pub real_radius_km: f64,
    /// Semi-major axis of the orbit in AU
    pub semi_major_axis_au: f64,
    /// Orbital period in Earth days
    pub orbital_period_days: f64,
    /// Orbital inclination in radians (relative to ecliptic)
    pub inclination_rad: f64,
    /// Starting orbital angle in radians (longitude at epoch)
    pub start_angle_rad: f64,
    /// Whether this body has rings (Saturn)
    pub has_rings: bool,
    /// Whether this body is the central star
    pub is_star: bool,
    /// Texture filename (e.g. "earth.jpg"), if any
    pub texture_file: Option<&'static str>,
    /// Current computed 3D position (updated each frame)
    pub position: Vec3,
}

impl CelestialBody {
    /// Compute the position of this body at a given simulation time (in Earth days).
    /// Uses simplified circular Kepler orbits.
    pub fn position_at(&self, time_days: f64) -> Vec3 {
        if self.is_star {
            return Vec3::ZERO;
        }

        // Mean angular velocity: ω = 2π / T
        let omega = std::f64::consts::TAU / self.orbital_period_days;

        // Current angle: θ = θ₀ + ωt
        let angle = self.start_angle_rad + omega * time_days;

        let display_distance = self.semi_major_axis_au as f32 * AU_TO_DISPLAY;

        // Position in the orbital plane, then tilt by inclination
        let cos_a = angle.cos() as f32;
        let sin_a = angle.sin() as f32;
        let cos_i = self.inclination_rad.cos() as f32;
        let sin_i = self.inclination_rad.sin() as f32;

        Vec3::new(
            display_distance * cos_a,
            display_distance * sin_a * sin_i,
            display_distance * sin_a * cos_i,
        )
    }

    /// Update the body's position for the current simulation time.
    pub fn update(&mut self, time_days: f64) {
        self.position = self.position_at(time_days);
    }
}
