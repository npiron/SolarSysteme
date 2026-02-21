pub mod body;
pub mod orbit;
pub mod time;

use body::CelestialBody;
use time::SimulationTime;

/// The top-level simulation state: holds all celestial bodies and the clock.
pub struct Simulation {
    pub bodies: Vec<CelestialBody>,
    pub time: SimulationTime,
}

impl Simulation {
    pub fn new(bodies: Vec<CelestialBody>) -> Self {
        Self {
            bodies,
            time: SimulationTime::new(),
        }
    }

    /// Advance the simulation by `dt_seconds` real-time seconds,
    /// then recompute all body positions.
    pub fn update(&mut self, dt_seconds: f64) {
        self.time.advance(dt_seconds);
        let t = self.time.current_days;
        for body in &mut self.bodies {
            body.update(t);
        }
    }

    /// Get the list of planets (everything that is not a star).
    #[allow(dead_code)]
    pub fn planets(&self) -> Vec<&CelestialBody> {
        self.bodies.iter().filter(|b| !b.is_star).collect()
    }
}
