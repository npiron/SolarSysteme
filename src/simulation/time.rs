//! Time management for the simulation.
//! Controls simulation speed, pause/resume, and current simulation date.

use crate::constants::DEFAULT_DAYS_PER_SECOND;

/// Discrete speed steps the user can cycle through (days per real second).
const SPEED_STEPS: &[f64] = &[0.1, 0.5, 1.0, 2.0, 5.0, 10.0, 50.0, 100.0];
const MIN_DAYS_PER_SECOND: f64 = 0.1;
const MAX_DAYS_PER_SECOND: f64 = 100.0;

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

    /// Set the simulation speed multiplier (clamped to valid range).
    pub fn set_speed(&mut self, days_per_second: f64) {
        self.days_per_second = days_per_second.clamp(MIN_DAYS_PER_SECOND, MAX_DAYS_PER_SECOND);
    }

    /// Toggle pause.
    pub fn toggle_pause(&mut self) {
        self.paused = !self.paused;
    }

    /// Advance to the next higher speed step.
    pub fn speed_up(&mut self) {
        let current = self.days_per_second;
        if let Some(&next) = SPEED_STEPS.iter().find(|&&s| s > current + 1e-9) {
            self.days_per_second = next;
        }
    }

    /// Retreat to the next lower speed step.
    pub fn speed_down(&mut self) {
        let current = self.days_per_second;
        if let Some(&prev) = SPEED_STEPS.iter().rev().find(|&&s| s < current - 1e-9) {
            self.days_per_second = prev;
        }
    }

    /// Return a human-readable speed label, e.g. `"×10"` or `"×0.5"`.
    pub fn speed_label(&self) -> String {
        let m = self.days_per_second / DEFAULT_DAYS_PER_SECOND;
        if m.fract() == 0.0 {
            format!("×{}", m as u64)
        } else {
            format!("×{m:.1}")
        }
    }
}
