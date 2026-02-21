//! Application state — the single shared structure that connects
//! the renderer, simulation, and input systems.
//!
//! Wrapped in `Rc<RefCell<…>>` so event closures and the render loop
//! can all mutate it safely.

use crate::renderer::Renderer;
use crate::simulation::Simulation;

/// Everything the app needs at runtime, bundled together.
pub struct AppState {
    pub renderer: Renderer,
    pub simulation: Simulation,

    // ── Input tracking ──
    pub mouse_down: bool,
    pub last_mouse_x: f32,
    pub last_mouse_y: f32,
    pub last_touch_x: f32,
    pub last_touch_y: f32,
    pub touch_distance: Option<f32>,
}

impl AppState {
    /// Build a new `AppState` from an already-initialized renderer and simulation.
    pub fn new(renderer: Renderer, simulation: Simulation) -> Self {
        Self {
            renderer,
            simulation,
            mouse_down: false,
            last_mouse_x: 0.0,
            last_mouse_y: 0.0,
            last_touch_x: 0.0,
            last_touch_y: 0.0,
            touch_distance: None,
        }
    }
}
