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
        .unwrap()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .unwrap()
}

// ─── Entry point ─────────────────────────────────────────────────────────

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).ok();

    log::info!("🚀 SOLARA is starting...");

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

    // ── Simulation ──
    let bodies = data::solar_system::create_solar_system();
    let simulation = Simulation::new(bodies.clone());

    // ── Renderer ──
    let renderer = Renderer::new(gl, width, height, &bodies)?;
    log::info!("✨ Renderer initialized ({width}×{height})");

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
        window.add_event_listener_with_callback("resize", closure.as_ref().unchecked_ref())?;
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

        {
            let mut s = state.borrow_mut();
            s.simulation.update(dt);
            let bodies = s.simulation.bodies.clone();
            s.renderer.render(&bodies, dt as f32);
        }

        request_animation_frame(f.borrow().as_ref().unwrap());
    }) as Box<dyn FnMut(f64)>));

    request_animation_frame(g.borrow().as_ref().unwrap());

    log::info!("🌍 SOLARA render loop started");
    Ok(())
}

// ─── Tests ───────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use crate::data::solar_system::create_solar_system;
    use crate::simulation::Simulation;

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
    fn simulation_advances_time() {
        let bodies = create_solar_system();
        let mut sim = Simulation::new(bodies);
        assert_eq!(sim.time.current_days, 0.0);
        sim.update(1.0);
        assert!(sim.time.current_days > 0.0);
    }
}
