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

    // ── Planet selection ──
    /// Index into `simulation.bodies` of the currently selected body, if any.
    pub selected_planet: Option<usize>,
    /// When `true`, the camera target is updated every frame to follow the
    /// selected planet as it orbits.
    pub camera_locked: bool,
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
            selected_planet: None,
            camera_locked: false,
        }
    }

    /// Advance the simulation and render one frame.
    ///
    /// This method exists so Rust can see that `self.simulation.bodies` and
    /// `self.renderer` are disjoint borrows — avoiding the need to clone
    /// the body list every frame.
    pub fn tick(&mut self, dt: f64) {
        self.simulation.update(dt);

        let fps = if dt > 0.0 { (1.0 / dt).min(1000.0) as f32 } else { 0.0 };
        crate::hud::update(
            self.simulation.time.current_days,
            self.simulation.time.days_per_second,
            self.simulation.time.paused,
            fps,
        );

        // If locked, keep the lerp target on the moving planet so the camera
        // continuously follows it.
        if self.camera_locked {
            if let Some(idx) = self.selected_planet {
                if idx < self.simulation.bodies.len() {
                    self.renderer.camera.lerp_target =
                        Some(self.simulation.bodies[idx].position);
                }
            }
        } else {
            // Default: keep camera centred on the Sun so it follows galactic drift.
            if let Some(sun) = self.simulation.bodies.iter().find(|b| b.is_star) {
                self.renderer.camera.lerp_target = Some(sun.position);
            }
        }

        self.renderer.camera.update_transition(dt as f32);
        self.renderer.render(&self.simulation.bodies, dt as f32);
    }
}
