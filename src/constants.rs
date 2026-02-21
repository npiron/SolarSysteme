//! Shared constants used across the simulation and renderer.
//!
//! Centralizing magic numbers here makes the codebase easier to tune
//! and prevents value drift between modules.

// ─── Display scaling ────────────────────────────────────────────────────

/// How many display units correspond to 1 Astronomical Unit.
/// Adjusting this single value rescales the entire solar system.
pub const AU_TO_DISPLAY: f32 = 40.0;

// ─── Orbit rendering ────────────────────────────────────────────────────

/// Number of line segments used to approximate each circular orbit.
pub const ORBIT_SEGMENTS: usize = 128;

// ─── Sphere mesh ─────────────────────────────────────────────────────────

/// Longitude subdivisions for the planet sphere mesh.
pub const SPHERE_SEGMENTS: u32 = 32;

/// Latitude subdivisions for the planet sphere mesh.
pub const SPHERE_RINGS: u32 = 24;

// ─── Saturn ring ─────────────────────────────────────────────────────────

/// Inner radius of Saturn's ring (in body-radius units).
pub const RING_INNER_RADIUS: f32 = 1.3;

/// Outer radius of Saturn's ring (in body-radius units).
pub const RING_OUTER_RADIUS: f32 = 2.3;

/// Number of segments for the ring annulus mesh.
pub const RING_SEGMENTS: u32 = 64;

// ─── Starfield ───────────────────────────────────────────────────────────

/// Number of background stars in the skybox.
pub const STARFIELD_COUNT: usize = 3000;

/// Distance of stars from the origin (should exceed camera far plane).
pub const STARFIELD_RADIUS: f32 = 2000.0;

// ─── Camera defaults ────────────────────────────────────────────────────

/// Initial horizontal angle (radians).
pub const CAMERA_THETA: f32 = 0.3;

/// Initial vertical angle (radians).
pub const CAMERA_PHI: f32 = 0.6;

/// Initial distance from target.
pub const CAMERA_DISTANCE: f32 = 200.0;

/// Minimum zoom distance.
pub const CAMERA_MIN_DISTANCE: f32 = 5.0;

/// Maximum zoom distance.
pub const CAMERA_MAX_DISTANCE: f32 = 1500.0;

/// Field of view in degrees (converted to radians at use-site).
pub const CAMERA_FOV_DEGREES: f32 = 45.0;

/// Near clipping plane.
pub const CAMERA_NEAR: f32 = 0.1;

/// Far clipping plane.
pub const CAMERA_FAR: f32 = 5000.0;

// ─── Input sensitivity ──────────────────────────────────────────────────

/// Mouse drag rotation sensitivity.
pub const ROTATE_SENSITIVITY: f32 = 0.005;

/// Scroll wheel zoom sensitivity.
pub const ZOOM_SENSITIVITY: f32 = 0.001;

/// Maximum vertical angle (radians) to prevent gimbal lock.
pub const PHI_CLAMP: f32 = 1.4;

/// Touch pinch zoom multiplier.
pub const TOUCH_ZOOM_MULTIPLIER: f32 = 2.0;

// ─── Galactic motion ─────────────────────────────────────────────────────────

/// Real orbital speed of the Sun around the galactic centre (km/s).
#[allow(dead_code)]
pub const GALACTIC_ORBITAL_SPEED_KM_S: f64 = 220.0;

/// Approximate galactic orbital period of the Sun (years).
#[allow(dead_code)]
pub const GALACTIC_PERIOD_YEARS: f64 = 230_000_000.0;

/// Galactic drift speed used for display, in display-units per simulated day.
///
/// Derived from the real speed (220 km/s → ~0.127 AU/day → ~5.1 display-units/day
/// with `AU_TO_DISPLAY = 40`).  Reduce this value to slow the visible drift,
/// or increase it to exaggerate the galactic motion for demonstration purposes.
pub const GALACTIC_SPEED_DISPLAY: f32 = 5.086;

// ─── Simulation defaults ────────────────────────────────────────────────

/// Default simulation speed: Earth-days per real second.
pub const DEFAULT_DAYS_PER_SECOND: f64 = 1.0;

/// Maximum frame delta (seconds) to prevent physics explosions.
pub const MAX_FRAME_DT: f64 = 0.1;

/// Assumed dt for the first frame (~60 fps).
pub const FIRST_FRAME_DT: f64 = 0.016;
