//! Input handling — mouse, touch, keyboard, and planet-selection events.
//!
//! All closures capture an `Rc<RefCell<AppState>>` and mutate the camera
//! or input-tracking fields. Closures are leaked intentionally because
//! they must live for the entire application lifetime.

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::app::AppState;
use crate::constants::{
    CAMERA_DISTANCE, DEFAULT_DAYS_PER_SECOND, PLANET_CLICK_RADIUS_FACTOR, PLANET_ZOOM_FACTOR,
    TOUCH_ZOOM_MULTIPLIER,
};
use crate::renderer::camera::Camera;
use glam::Vec3;

/// Attach all input event listeners to the given canvas.
///
/// Every closure is `.forget()`-ed so it lives as long as the page.
/// This is safe because the app never needs to remove them.
pub fn setup_input(canvas: &HtmlCanvasElement, state: Rc<RefCell<AppState>>) {
    bind_mouse_events(canvas, &state);
    bind_wheel_event(canvas, &state);
    bind_touch_events(canvas, &state);
    bind_keyboard_events(&state);
}

// ── Planet selection helpers ─────────────────────────────────────────────

/// Cast a ray from the camera through `(mouse_x, mouse_y)` (in CSS pixels,
/// relative to the canvas) and return the index of the nearest body hit, if any.
fn raycast_planets(
    camera: &Camera,
    body_positions: &[(Vec3, f32)], // (position, display_radius)
    mouse_x: f32,
    mouse_y: f32,
    canvas_w: f32,
    canvas_h: f32,
) -> Option<usize> {
    if canvas_w == 0.0 || canvas_h == 0.0 {
        return None;
    }
    let ndc_x = (2.0 * mouse_x / canvas_w) - 1.0;
    let ndc_y = 1.0 - (2.0 * mouse_y / canvas_h);

    // Unproject through the combined view-projection matrix.
    let vp = camera.projection_matrix() * camera.view_matrix();
    let inv_vp = vp.inverse();

    let near_clip = glam::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
    let far_clip = glam::Vec4::new(ndc_x, ndc_y, 1.0, 1.0);

    let near_w = inv_vp * near_clip;
    let far_w = inv_vp * far_clip;

    let near_pos = near_w.truncate() / near_w.w;
    let far_pos = far_w.truncate() / far_w.w;

    let ray_origin = near_pos;
    let ray_dir = (far_pos - near_pos).normalize();

    let mut nearest: Option<(usize, f32)> = None;

    for (i, (center, display_radius)) in body_positions.iter().enumerate() {
        let radius = display_radius * PLANET_CLICK_RADIUS_FACTOR;
        let oc = ray_origin - *center;
        let b = oc.dot(ray_dir);
        let c = oc.dot(oc) - radius * radius;
        let discriminant = b * b - c;

        if discriminant >= 0.0 {
            let t = -b - discriminant.sqrt();
            let t = if t > 0.0 { t } else { -b + discriminant.sqrt() };
            if t > 0.0 && nearest.map_or(true, |(_, d)| t < d) {
                nearest = Some((i, t));
            }
        }
    }

    nearest.map(|(i, _)| i)
}

/// Select a celestial body by index: animate the camera toward it and update
/// the info panel.  Does nothing if `idx` is already selected.
fn select_planet(state: &mut AppState, idx: usize) {
    if state.selected_planet == Some(idx) {
        return;
    }
    if idx >= state.simulation.bodies.len() {
        return;
    }

    // Extract all data we need before mutating (avoids split-borrow issues).
    let (name, radius_km, dist_au, period_days, incl_rad, is_star, display_r, body_pos) = {
        let b = &state.simulation.bodies[idx];
        (
            b.name,
            b.real_radius_km,
            b.semi_major_axis_au,
            b.orbital_period_days,
            b.inclination_rad,
            b.is_star,
            b.display_radius,
            b.position,
        )
    };

    let zoom_dist = (display_r * PLANET_ZOOM_FACTOR)
        .max(state.renderer.camera.min_distance * 1.5);
    state.renderer.camera.set_target(body_pos, zoom_dist);

    // Changing selection clears any existing camera lock.
    state.camera_locked = false;
    state.selected_planet = Some(idx);

    show_planet_panel(name, radius_km, dist_au, period_days, incl_rad, is_star, false);
}

/// Deselect the current body and return the camera to the overview.
fn deselect_all(state: &mut AppState) {
    state.selected_planet = None;
    state.camera_locked = false;
    state.renderer.camera.set_target(Vec3::ZERO, CAMERA_DISTANCE);
    hide_planet_panel();
}

/// Toggle the camera-lock on the currently selected planet.
fn toggle_camera_lock(state: &mut AppState) {
    if state.selected_planet.is_none() {
        return;
    }
    state.camera_locked = !state.camera_locked;
    update_planet_panel_lock(state.camera_locked);
}

// ── DOM helpers ──────────────────────────────────────────────────────────

fn show_planet_panel(
    name: &str,
    radius_km: f64,
    dist_au: f64,
    period_days: f64,
    inclination_rad: f64,
    is_star: bool,
    locked: bool,
) {
    let Some(doc) = web_sys::window().and_then(|w| w.document()) else {
        return;
    };

    let set = |id: &str, html: &str| {
        if let Some(el) = doc.get_element_by_id(id) {
            el.set_inner_html(html);
        }
    };

    set("planet-name", name);
    set("planet-radius", &format!("{radius_km:.0} km"));

    if is_star {
        set("planet-distance", "Center of system");
        set("planet-period", "—");
    } else {
        set("planet-distance", &format!("{dist_au:.3} AU"));
        set("planet-period", &format!("{period_days:.1} days"));
    }

    set(
        "planet-inclination",
        &format!("{:.2}°", inclination_rad.to_degrees()),
    );
    set(
        "planet-lock-hint",
        if locked {
            "🔒 Locked · DOUBLE-CLICK to unlock"
        } else {
            "DOUBLE-CLICK to lock camera"
        },
    );

    if let Some(panel) = doc.get_element_by_id("planet-info") {
        let _ = panel.class_list().remove_1("hidden");
    }
}

fn hide_planet_panel() {
    if let Some(panel) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("planet-info"))
    {
        let _ = panel.class_list().add_1("hidden");
    }
}

fn update_planet_panel_lock(locked: bool) {
    if let Some(el) = web_sys::window()
        .and_then(|w| w.document())
        .and_then(|d| d.get_element_by_id("planet-lock-hint"))
    {
        el.set_inner_html(if locked {
            "🔒 Locked · DOUBLE-CLICK to unlock"
        } else {
            "DOUBLE-CLICK to lock camera"
        });
    }
}

// ── Mouse ────────────────────────────────────────────────────────────────

fn bind_mouse_events(canvas: &HtmlCanvasElement, state: &Rc<RefCell<AppState>>) {
    // Mouse down
    {
        let state = Rc::clone(state);
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let mut s = state.borrow_mut();
            s.mouse_down = true;
            s.last_mouse_x = e.client_x() as f32;
            s.last_mouse_y = e.client_y() as f32;
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        canvas
            .add_event_listener_with_callback("mousedown", closure.as_ref().unchecked_ref())
            .expect("Failed to bind mousedown listener");
        closure.forget();
    }

    // Mouse up
    {
        let state = Rc::clone(state);
        let closure = Closure::wrap(Box::new(move |_: web_sys::MouseEvent| {
            state.borrow_mut().mouse_down = false;
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        canvas
            .add_event_listener_with_callback("mouseup", closure.as_ref().unchecked_ref())
            .expect("Failed to bind mouseup listener");
        closure.forget();
    }

    // Mouse move
    {
        let state = Rc::clone(state);
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let mut s = state.borrow_mut();
            if s.mouse_down {
                let dx = e.client_x() as f32 - s.last_mouse_x;
                let dy = e.client_y() as f32 - s.last_mouse_y;
                s.renderer.camera.rotate(dx, dy);
            }
            s.last_mouse_x = e.client_x() as f32;
            s.last_mouse_y = e.client_y() as f32;
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        canvas
            .add_event_listener_with_callback("mousemove", closure.as_ref().unchecked_ref())
            .expect("Failed to bind mousemove listener");
        closure.forget();
    }

    // Click — raycasting to select a planet
    {
        let state = Rc::clone(state);
        let canvas_click = canvas.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let mut s = state.borrow_mut();
            let x = e.offset_x() as f32;
            let y = e.offset_y() as f32;
            let w = canvas_click.client_width() as f32;
            let h = canvas_click.client_height() as f32;

            // Snapshot body positions to avoid a split-borrow on `s`.
            let body_data: Vec<(Vec3, f32)> = s
                .simulation
                .bodies
                .iter()
                .map(|b| (b.position, b.display_radius))
                .collect();

            let hit = raycast_planets(&s.renderer.camera, &body_data, x, y, w, h);

            match hit {
                Some(idx) => select_planet(&mut s, idx),
                None => deselect_all(&mut s),
            }
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        canvas
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .expect("Failed to bind click listener");
        closure.forget();
    }

    // Double-click — toggle camera lock on selected planet
    {
        let state = Rc::clone(state);
        let canvas_dbl = canvas.clone();
        let closure = Closure::wrap(Box::new(move |e: web_sys::MouseEvent| {
            let mut s = state.borrow_mut();

            if s.selected_planet.is_some() {
                // Toggle lock on the already-selected planet.
                toggle_camera_lock(&mut s);
            } else {
                // Nothing selected yet — try to select and immediately lock.
                let x = e.offset_x() as f32;
                let y = e.offset_y() as f32;
                let w = canvas_dbl.client_width() as f32;
                let h = canvas_dbl.client_height() as f32;

                let body_data: Vec<(Vec3, f32)> = s
                    .simulation
                    .bodies
                    .iter()
                    .map(|b| (b.position, b.display_radius))
                    .collect();

                if let Some(idx) =
                    raycast_planets(&s.renderer.camera, &body_data, x, y, w, h)
                {
                    select_planet(&mut s, idx);
                    toggle_camera_lock(&mut s);
                }
            }
        }) as Box<dyn FnMut(web_sys::MouseEvent)>);
        canvas
            .add_event_listener_with_callback("dblclick", closure.as_ref().unchecked_ref())
            .expect("Failed to bind dblclick listener");
        closure.forget();
    }
}

// ── Scroll wheel ─────────────────────────────────────────────────────────

fn bind_wheel_event(canvas: &HtmlCanvasElement, state: &Rc<RefCell<AppState>>) {
    let state = Rc::clone(state);
    let closure = Closure::wrap(Box::new(move |e: web_sys::WheelEvent| {
        e.prevent_default();
        state.borrow_mut().renderer.camera.zoom(e.delta_y() as f32);
    }) as Box<dyn FnMut(web_sys::WheelEvent)>);

    let opts = web_sys::AddEventListenerOptions::new();
    opts.set_passive(false);
    canvas
        .add_event_listener_with_callback_and_add_event_listener_options(
            "wheel",
            closure.as_ref().unchecked_ref(),
            &opts,
        )
        .expect("Failed to bind wheel listener");
    closure.forget();
}

// ── Touch ────────────────────────────────────────────────────────────────

fn bind_touch_events(canvas: &HtmlCanvasElement, state: &Rc<RefCell<AppState>>) {
    let touch_opts = || {
        let o = web_sys::AddEventListenerOptions::new();
        o.set_passive(false);
        o
    };

    // Touch start
    {
        let state = Rc::clone(state);
        let closure = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            e.prevent_default();
            let mut s = state.borrow_mut();
            let touches = e.touches();
            if touches.length() == 1 {
                if let Some(t) = touches.get(0) {
                    s.last_touch_x = t.client_x() as f32;
                    s.last_touch_y = t.client_y() as f32;
                }
                s.touch_distance = None;
            } else if touches.length() == 2
                && let (Some(t0), Some(t1)) = (touches.get(0), touches.get(1))
            {
                let dx = (t1.client_x() - t0.client_x()) as f32;
                let dy = (t1.client_y() - t0.client_y()) as f32;
                s.touch_distance = Some((dx * dx + dy * dy).sqrt());
            }
        }) as Box<dyn FnMut(web_sys::TouchEvent)>);
        canvas
            .add_event_listener_with_callback_and_add_event_listener_options(
                "touchstart",
                closure.as_ref().unchecked_ref(),
                &touch_opts(),
            )
            .expect("Failed to bind touchstart listener");
        closure.forget();
    }

    // Touch move
    {
        let state = Rc::clone(state);
        let closure = Closure::wrap(Box::new(move |e: web_sys::TouchEvent| {
            e.prevent_default();
            let mut s = state.borrow_mut();
            let touches = e.touches();
            if touches.length() == 1 {
                if let Some(t) = touches.get(0) {
                    let x = t.client_x() as f32;
                    let y = t.client_y() as f32;
                    let dx = x - s.last_touch_x;
                    let dy = y - s.last_touch_y;
                    s.renderer.camera.rotate(dx, dy);
                    s.last_touch_x = x;
                    s.last_touch_y = y;
                }
            } else if touches.length() == 2
                && let (Some(t0), Some(t1)) = (touches.get(0), touches.get(1))
            {
                let dx = (t1.client_x() - t0.client_x()) as f32;
                let dy = (t1.client_y() - t0.client_y()) as f32;
                let new_dist = (dx * dx + dy * dy).sqrt();
                if let Some(old_dist) = s.touch_distance {
                    let delta = old_dist - new_dist;
                    s.renderer.camera.zoom(delta * TOUCH_ZOOM_MULTIPLIER);
                }
                s.touch_distance = Some(new_dist);
            }
        }) as Box<dyn FnMut(web_sys::TouchEvent)>);
        canvas
            .add_event_listener_with_callback_and_add_event_listener_options(
                "touchmove",
                closure.as_ref().unchecked_ref(),
                &touch_opts(),
            )
            .expect("Failed to bind touchmove listener");
        closure.forget();
    }
}

// ── Keyboard ─────────────────────────────────────────────────────────────

fn bind_keyboard_events(state: &Rc<RefCell<AppState>>) {
    let state = Rc::clone(state);
    let closure = Closure::wrap(Box::new(move |e: web_sys::KeyboardEvent| {
        match e.key().as_str() {
            // Space → toggle pause
            " " => {
                e.prevent_default();
                let mut s = state.borrow_mut();
                s.simulation.time.toggle_pause();
                crate::hud::update(
                    s.simulation.time.current_days,
                    s.simulation.time.days_per_second,
                    s.simulation.time.paused,
                    0.0,
                );
            }
            // Arrow Up / + → speed up
            "ArrowUp" | "+" => {
                e.prevent_default();
                let mut s = state.borrow_mut();
                s.simulation.time.speed_up();
                crate::hud::update(
                    s.simulation.time.current_days,
                    s.simulation.time.days_per_second,
                    s.simulation.time.paused,
                    0.0,
                );
            }
            // Arrow Down / - → slow down
            "ArrowDown" | "-" => {
                e.prevent_default();
                let mut s = state.borrow_mut();
                s.simulation.time.speed_down();
                crate::hud::update(
                    s.simulation.time.current_days,
                    s.simulation.time.days_per_second,
                    s.simulation.time.paused,
                    0.0,
                );
            }
            // R → reset speed to default
            "r" | "R" => {
                let mut s = state.borrow_mut();
                s.simulation.time.set_speed(DEFAULT_DAYS_PER_SECOND);
                s.simulation.time.paused = false;
                crate::hud::update(
                    s.simulation.time.current_days,
                    s.simulation.time.days_per_second,
                    s.simulation.time.paused,
                    0.0,
                );
            }
            // H → toggle HUD visibility
            "h" | "H" => {
                crate::hud::toggle();
            }
            // Home → re-center camera on Sun, deselect planet
            "Home" => {
                e.prevent_default();
                let mut s = state.borrow_mut();
                deselect_all(&mut s);
                // Reset camera to default distance & angles
                s.renderer.camera.set_target(
                    glam::Vec3::ZERO,
                    crate::constants::CAMERA_DISTANCE,
                );
            }
            // T → top-down view
            "t" | "T" => {
                e.prevent_default();
                let mut s = state.borrow_mut();
                s.renderer.camera.phi = crate::constants::PHI_CLAMP;  // look from above
                s.renderer.camera.theta = 0.0;
            }
            // Escape → deselect planet, return to overview
            "Escape" => {
                e.prevent_default();
                deselect_all(&mut state.borrow_mut());
            }
            // 1–8 → select Mercury through Neptune directly.
            // This relies on the fixed body ordering in data::solar_system:
            // index 0 = Sun, 1 = Mercury, …, 8 = Neptune.
            "1" => select_planet(&mut state.borrow_mut(), 1),
            "2" => select_planet(&mut state.borrow_mut(), 2),
            "3" => select_planet(&mut state.borrow_mut(), 3),
            "4" => select_planet(&mut state.borrow_mut(), 4),
            "5" => select_planet(&mut state.borrow_mut(), 5),
            "6" => select_planet(&mut state.borrow_mut(), 6),
            "7" => select_planet(&mut state.borrow_mut(), 7),
            "8" => select_planet(&mut state.borrow_mut(), 8),
            _ => {}
        }
    }) as Box<dyn FnMut(web_sys::KeyboardEvent)>);

    // Bind to document so it works even without canvas focus
    let document = web_sys::window()
        .and_then(|w| w.document())
        .expect("Failed to get document for keyboard events");
    document
        .add_event_listener_with_callback("keydown", closure.as_ref().unchecked_ref())
        .expect("Failed to bind keydown listener");
    closure.forget();
}
