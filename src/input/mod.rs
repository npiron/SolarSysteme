//! Input handling — mouse, touch, and scroll event binding.
//!
//! All closures capture an `Rc<RefCell<AppState>>` and mutate the camera
//! or input-tracking fields. Closures are leaked intentionally because
//! they must live for the entire application lifetime.

use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::app::AppState;
use crate::constants::{DEFAULT_DAYS_PER_SECOND, TOUCH_ZOOM_MULTIPLIER};

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
                state.borrow_mut().simulation.time.toggle_pause();
            }
            // Arrow Up → double speed
            "ArrowUp" => {
                e.prevent_default();
                let mut s = state.borrow_mut();
                let spd = s.simulation.time.days_per_second;
                s.simulation.time.set_speed(spd * 2.0);
            }
            // Arrow Down → halve speed
            "ArrowDown" => {
                e.prevent_default();
                let mut s = state.borrow_mut();
                let spd = s.simulation.time.days_per_second;
                s.simulation.time.set_speed((spd * 0.5).max(0.125));
            }
            // R → reset speed to default
            "r" | "R" => {
                state
                    .borrow_mut()
                    .simulation
                    .time
                    .set_speed(DEFAULT_DAYS_PER_SECOND);
            }
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
