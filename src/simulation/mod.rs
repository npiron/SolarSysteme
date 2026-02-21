pub mod body;
pub mod orbit;
pub mod time;

use glam::Vec3;

use body::CelestialBody;
use time::SimulationTime;

use crate::constants::GALACTIC_SPEED_DISPLAY;

/// Solar-apex direction (toward Hercules/Lyra, RA ≈ 18 h 28 m, Dec ≈ +30°)
/// expressed as a pre-normalised unit vector in the simulation's coordinate frame.
///
/// Computed from equatorial coordinates (RA = 277°, Dec = +30°):
///   x = cos(Dec)·cos(RA),  y = cos(Dec)·sin(RA),  z = sin(Dec)
/// The result is already a unit vector (magnitude ≈ 1.0).
const SOLAR_APEX: Vec3 = Vec3::new(0.10554, -0.85959, 0.50000);

/// The top-level simulation state: holds all celestial bodies and the clock.
pub struct Simulation {
    pub bodies: Vec<CelestialBody>,
    pub time: SimulationTime,
    /// Galactic drift velocity of the whole solar system (display-units / simulated day).
    ///
    /// Multiply by `time.current_days` to obtain the cumulative galactic offset that
    /// is added to every body's position each frame.
    pub galactic_velocity: Vec3,
}

impl Simulation {
    pub fn new(bodies: Vec<CelestialBody>) -> Self {
        let galactic_velocity = SOLAR_APEX * GALACTIC_SPEED_DISPLAY;
        Self {
            bodies,
            time: SimulationTime::new(),
            galactic_velocity,
        }
    }

    /// Advance the simulation by `dt_seconds` real-time seconds,
    /// then recompute all body positions.
    pub fn update(&mut self, dt_seconds: f64) {
        self.time.advance(dt_seconds);
        let t = self.time.current_days;
        let galactic_offset = self.galactic_velocity * t as f32;
        for body in &mut self.bodies {
            body.update(t, galactic_offset);
        }
    }

    /// Get the list of planets (everything that is not a star).
    #[allow(dead_code)]
    pub fn planets(&self) -> Vec<&CelestialBody> {
        self.bodies.iter().filter(|b| !b.is_star).collect()
    }
}
