//! Splash-screen progress updates.
//!
//! Uses typed `wasm-bindgen` imports instead of `js_sys::eval` for
//! better performance, CSP compatibility, and type safety.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
    export function splash_update_step(stepId, status) {
        if (window.solaraUpdateStep) window.solaraUpdateStep(stepId, status);
    }
    export function splash_hide() {
        if (window.solaraHideSplash) window.solaraHideSplash();
    }
")]
extern "C" {
    fn splash_update_step(step_id: &str, status: &str);
    fn splash_hide();
}

/// Mark a step as `"loading"`, `"done"`, or `"pending"`.
///
/// `step_id` maps to the HTML element `id="step-{step_id}"`.
pub fn update_step(step_id: &str, status: &str) {
    splash_update_step(step_id, status);
}

/// Fade-out and remove the splash screen.
pub fn hide_splash() {
    splash_hide();
}
