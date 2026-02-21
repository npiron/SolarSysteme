//! HUD (Head-Up Display) DOM updates.
//!
//! Follows the same wasm-bindgen inline-JS pattern as `splash.rs` for
//! CSP-compatible, type-safe DOM manipulation without `eval`.

use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = "
    export function hud_update(days, speed, paused, fps) {
        if (window.solaraUpdateHud) window.solaraUpdateHud(days, speed, paused, fps);
    }
    export function hud_toggle() {
        if (window.solaraToggleHud) window.solaraToggleHud();
    }
")]
extern "C" {
    fn hud_update(days: f64, speed: f64, paused: bool, fps: f32);
    fn hud_toggle();
}

/// Push current simulation telemetry to the HUD DOM elements.
///
/// - `current_days`   — simulation time in Earth-days since J2000
/// - `days_per_second`— simulation speed multiplier
/// - `paused`         — whether the simulation is paused
/// - `fps`            — raw frames-per-second for this frame
pub fn update(current_days: f64, days_per_second: f64, paused: bool, fps: f32) {
    hud_update(current_days, days_per_second, paused, fps);
}

/// Toggle HUD visibility (bound to the `H` key).
pub fn toggle() {
    hud_toggle();
}
