//! Time management for the simulation.
//! Controls simulation speed, pause/resume, and current simulation date.

use crate::constants::DEFAULT_DAYS_PER_SECOND;

#[derive(Debug, Clone)]
pub struct SimulationTime {
    /// Current simulation time in Earth days since epoch (J2000)
    pub current_days: f64,
    /// Simulation speed: how many Earth days pass per real second
    pub days_per_second: f64,
    /// Whether the simulation is paused
    pub paused: bool,
}

impl Default for SimulationTime {
    fn default() -> Self {
        Self {
            current_days: 0.0,
            days_per_second: DEFAULT_DAYS_PER_SECOND,
            paused: false,
        }
    }
}

impl SimulationTime {
    pub fn new() -> Self {
        Self::default()
    }

    /// Advance the simulation by `dt_seconds` real seconds.
    pub fn advance(&mut self, dt_seconds: f64) {
        if !self.paused {
            self.current_days += dt_seconds * self.days_per_second;
        }
    }

    /// Set the simulation speed multiplier.
    #[allow(dead_code)]
    pub fn set_speed(&mut self, days_per_second: f64) {
        self.days_per_second = days_per_second.max(0.0);
    }

    /// Toggle pause.
    #[allow(dead_code)]
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }
}
