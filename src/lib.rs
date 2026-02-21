//! SOLARA — The solar system, forged in Rust, alive in your browser.
//!
//! This crate compiles to WebAssembly and drives a real-time solar system
//! simulation rendered with WebGL2.
//!
//! ## Module layout
//!
//! | Module        | Purpose                                          |
//! |---------------|--------------------------------------------------|
//! | [`app`]       | Shared application state                         |
//! | [`constants`] | Centralised tuneable values                      |
//! | [`data`]      | NASA-sourced solar system data                   |
//! | [`input`]     | Browser event → camera mutations                 |
//! | [`renderer`]  | WebGL2 draw pipeline, shaders, textures, meshes  |
//! | [`simulation`]| Kepler orbits, time control, celestial bodies    |

mod app;
mod constants;
mod data;
mod input;
mod renderer;
mod simulation;
mod splash;

use std::cell::RefCell;
use std::rc::Rc;

use app::AppState;
use constants::*;
use renderer::Renderer;
use simulation::Simulation;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext as GL;

// ─── Helpers ─────────────────────────────────────────────────────────────

/// Schedule the next animation frame.
fn request_animation_frame(f: &Closure<dyn FnMut(f64)>) -> i32 {
    web_sys::window()
        .and_then(|w| w.request_animation_frame(f.as_ref().unchecked_ref()).ok())
        .unwrap_or(0)
}

// ─── Entry point ─────────────────────────────────────────────────────────

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).ok();

    log::info!("🚀 SOLARA is starting...");

    splash::update_step("wasm", "done");
    splash::update_step("webgl", "loading");

    // ── Canvas & WebGL2 ──
    let window = web_sys::window().ok_or("No window")?;
    let document = window.document().ok_or("No document")?;
    let canvas = document
        .get_element_by_id("solara-canvas")
        .ok_or("Canvas #solara-canvas not found")?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;

    let dpr = window.device_pixel_ratio();
    let width = (window.inner_width()?.as_f64().unwrap() * dpr) as u32;
    let height = (window.inner_height()?.as_f64().unwrap() * dpr) as u32;
    canvas.set_width(width);
    canvas.set_height(height);

    let gl: GL = canvas
        .get_context("webgl2")?
        .ok_or("WebGL2 not supported")?
        .dyn_into::<GL>()?;
    gl.viewport(0, 0, width as i32, height as i32);

    splash::update_step("webgl", "done");
    splash::update_step("simulation", "loading");

    // ── Simulation ──
    let bodies = data::solar_system::create_solar_system();
    let simulation = Simulation::new(bodies.clone());

    splash::update_step("simulation", "done");
    splash::update_step("renderer", "loading");

    // ── Renderer ──
    let renderer = Renderer::new(gl, width, height, &bodies)?;
    log::info!("✨ Renderer initialized ({width}×{height})");

    splash::update_step("renderer", "done");

    // ── Textures (async) ──
    let gl_ref = renderer.gl_handle();
    let tex_ref = renderer.textures_handle();
    renderer::texture::start_loading_textures(&gl_ref, &tex_ref, &bodies);
    log::info!("📥 Texture loading started for {} bodies", bodies.len());

    // ── Shared state ──
    let state = Rc::new(RefCell::new(AppState::new(renderer, simulation)));

    // ── Input ──
    input::setup_input(&canvas, Rc::clone(&state));

    // ── Window resize ──
    {
        let state_resize = Rc::clone(&state);
        let canvas_resize = canvas.clone();
        let closure = Closure::wrap(Box::new(move |_: web_sys::Event| {
            let win = web_sys::window().unwrap();
            let dpr = win.device_pixel_ratio();
            let w = (win.inner_width().unwrap().as_f64().unwrap() * dpr) as u32;
            let h = (win.inner_height().unwrap().as_f64().unwrap() * dpr) as u32;
            canvas_resize.set_width(w);
            canvas_resize.set_height(h);
            state_resize.borrow_mut().renderer.resize(w, h);
        }) as Box<dyn FnMut(web_sys::Event)>);
        window.add_event_listener_with_callback("solara-resize", closure.as_ref().unchecked_ref())?;
        closure.forget();
    }

    // ── Render loop ──
    type AnimFrameClosure = Rc<RefCell<Option<Closure<dyn FnMut(f64)>>>>;
    let f: AnimFrameClosure = Rc::new(RefCell::new(None));
    let g = Rc::clone(&f);
    let last_time = Rc::new(RefCell::new(0.0_f64));

    *g.borrow_mut() = Some(Closure::wrap(Box::new(move |timestamp: f64| {
        let mut lt = last_time.borrow_mut();
        let dt = if *lt == 0.0 {
            FIRST_FRAME_DT
        } else {
            ((timestamp - *lt) / 1000.0).min(MAX_FRAME_DT)
        };
        *lt = timestamp;

        state.borrow_mut().tick(dt);

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    log::info!("🌍 SOLARA render loop started");
    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::constants::*;
    use crate::data::solar_system::create_solar_system;
    use crate::renderer::camera::Camera;
    use crate::renderer::mesh;
    use crate::simulation::orbit;
    use crate::simulation::time::SimulationTime;
    use crate::simulation::Simulation;

    // ── Solar system data ──

    #[test]
    fn all_planets_initialized() {
        let bodies = create_solar_system();
        let sim = Simulation::new(bodies);
        assert_eq!(sim.planets().len(), 8);
    }

    #[test]
    fn sun_is_at_origin() {
        let bodies = create_solar_system();
        let sim = Simulation::new(bodies);
        let sun = sim.bodies.iter().find(|b| b.is_star).unwrap();
        assert_eq!(sun.position, glam::Vec3::ZERO);
    }

    #[test]
    fn earth_returns_after_one_orbit() {
        let bodies = create_solar_system();
        let earth = bodies.iter().find(|b| b.name == "Earth").unwrap();
        let pos_start = earth.position_at(0.0);
        let pos_full = earth.position_at(365.25);
        let distance = pos_start.distance(pos_full);
        assert!(
            distance < 0.1,
            "Earth should return near start after one period, got {distance}"
        );
    }

    #[test]
    fn mercury_faster_than_neptune() {
        let bodies = create_solar_system();
        let mercury = bodies.iter().find(|b| b.name == "Mercury").unwrap();
        let neptune = bodies.iter().find(|b| b.name == "Neptune").unwrap();
        let merc = mercury.position_at(0.0).distance(mercury.position_at(100.0));
        let nept = neptune.position_at(0.0).distance(neptune.position_at(100.0));
        assert!(merc > nept, "Mercury should move faster than Neptune");
    }

    #[test]
    fn all_bodies_have_texture_files() {
        let bodies = create_solar_system();
        for body in &bodies {
            assert!(
                body.texture_file.is_some(),
                "{} should have a texture file",
                body.name
            );
        }
    }

    // ── Simulation / time ──

    #[test]
    fn simulation_advances_time() {
        let bodies = create_solar_system();
        let mut sim = Simulation::new(bodies);
        assert_eq!(sim.time.current_days, 0.0);
        sim.update(1.0);
        assert!(sim.time.current_days > 0.0);
    }

    #[test]
    fn simulation_pause_stops_time() {
        let mut time = SimulationTime::new();
        time.toggle_pause();
        time.advance(10.0);
        assert_eq!(time.current_days, 0.0, "Time should not advance while paused");
    }

    #[test]
    fn simulation_speed_multiplier() {
        let mut time = SimulationTime::new();
        time.set_speed(10.0);
        time.advance(1.0); // 1 real second
        assert!(
            (time.current_days - 10.0).abs() < 1e-9,
            "10 days/sec × 1 sec = 10 days, got {}",
            time.current_days
        );
    }

    #[test]
    fn simulation_speed_cannot_be_negative() {
        let mut time = SimulationTime::new();
        time.set_speed(-5.0);
        assert_eq!(time.days_per_second, 0.0);
    }

    #[test]
    fn simulation_default_speed() {
        let time = SimulationTime::new();
        assert_eq!(time.days_per_second, DEFAULT_DAYS_PER_SECOND);
    }

    // ── Camera ──

    #[test]
    fn camera_defaults_from_constants() {
        let cam = Camera::new(16.0 / 9.0);
        assert_eq!(cam.theta, CAMERA_THETA);
        assert_eq!(cam.phi, CAMERA_PHI);
        assert_eq!(cam.distance, CAMERA_DISTANCE);
        assert_eq!(cam.min_distance, CAMERA_MIN_DISTANCE);
        assert_eq!(cam.max_distance, CAMERA_MAX_DISTANCE);
    }

    #[test]
    fn camera_eye_not_at_target() {
        let cam = Camera::new(1.0);
        let eye = cam.eye_position();
        assert!(
            eye.distance(cam.target) > 1.0,
            "Eye should be away from target"
        );
    }

    #[test]
    fn camera_zoom_clamps() {
        let mut cam = Camera::new(1.0);
        // Zoom way in
        for _ in 0..5000 {
            cam.zoom(-100.0);
        }
        assert!(
            cam.distance >= cam.min_distance,
            "Should not go below min distance"
        );
        // Zoom way out
        for _ in 0..5000 {
            cam.zoom(100.0);
        }
        assert!(
            cam.distance <= cam.max_distance,
            "Should not exceed max distance"
        );
    }

    #[test]
    fn camera_rotate_clamps_phi() {
        let mut cam = Camera::new(1.0);
        // Rotate up a lot
        for _ in 0..10000 {
            cam.rotate(0.0, -10.0);
        }
        assert!(cam.phi >= -PHI_CLAMP);
        assert!(cam.phi <= PHI_CLAMP);
    }

    #[test]
    fn camera_set_aspect() {
        let mut cam = Camera::new(1.0);
        cam.set_aspect(2.0);
        assert_eq!(cam.aspect, 2.0);
    }

    #[test]
    fn camera_matrices_are_finite() {
        let cam = Camera::new(16.0 / 9.0);
        let view = cam.view_matrix();
        let proj = cam.projection_matrix();
        for col in view.to_cols_array() {
            assert!(col.is_finite(), "View matrix has non-finite element");
        }
        for col in proj.to_cols_array() {
            assert!(col.is_finite(), "Projection matrix has non-finite element");
        }
    }

    // ── Mesh generation ──

    #[test]
    fn sphere_has_vertices_and_indices() {
        let sphere = mesh::generate_sphere();
        assert!(!sphere.vertices.is_empty(), "Sphere should have vertices");
        assert!(!sphere.indices.is_empty(), "Sphere should have indices");
        // 8 floats per vertex (pos.xyz + norm.xyz + uv.xy)
        assert_eq!(sphere.vertices.len() % 8, 0, "Vertex data should be 8-float aligned");
    }

    #[test]
    fn ring_has_vertices_and_indices() {
        let ring = mesh::generate_ring();
        assert!(!ring.vertices.is_empty());
        assert!(!ring.indices.is_empty());
    }

    #[test]
    fn sphere_custom_resolution() {
        let lo = mesh::generate_sphere_custom(8, 6);
        let hi = mesh::generate_sphere_custom(64, 48);
        assert!(
            hi.vertices.len() > lo.vertices.len(),
            "Higher resolution should produce more vertices"
        );
    }

    // ── Orbit geometry ──

    #[test]
    fn orbit_path_is_closed_loop() {
        let path = orbit::generate_orbit_path(1.0, 0.0);
        assert_eq!(path.len(), ORBIT_SEGMENTS + 1, "Path should have SEGMENTS+1 points");
        let first = path.first().unwrap();
        let last = path.last().unwrap();
        assert!(
            first.distance(*last) < 0.01,
            "First and last points should nearly coincide"
        );
    }

    #[test]
    fn orbit_radius_scales_with_au() {
        let inner = orbit::generate_orbit_path(1.0, 0.0);
        let outer = orbit::generate_orbit_path(5.0, 0.0);
        let r_inner = inner[0].length();
        let r_outer = outer[0].length();
        assert!(
            r_outer > r_inner * 4.0,
            "5 AU orbit should be much larger than 1 AU orbit"
        );
    }

    #[test]
    fn orbit_with_inclination_has_y_component() {
        let flat = orbit::generate_orbit_path(1.0, 0.0);
        let tilted = orbit::generate_orbit_path(1.0, 0.3);
        let max_y_flat: f32 = flat.iter().map(|p| p.y.abs()).fold(0.0, f32::max);
        let max_y_tilted: f32 = tilted.iter().map(|p| p.y.abs()).fold(0.0, f32::max);
        assert!(
            max_y_tilted > max_y_flat + 0.1,
            "Tilted orbit should have larger Y extent"
        );
    }

    // ── Constants consistency ──

    #[test]
    fn camera_near_less_than_far() {
        assert!(CAMERA_NEAR < CAMERA_FAR);
    }

    #[test]
    fn starfield_radius_exceeds_camera_max() {
        assert!(
            STARFIELD_RADIUS > CAMERA_MAX_DISTANCE,
            "Stars should be beyond max zoom-out distance"
        );
    }
}
