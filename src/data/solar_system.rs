//! Real NASA solar system data.
//! Sources: NASA Planetary Fact Sheet (https://nssdc.gsfc.nasa.gov/planetary/factsheet/)
//!
//! Display radii are log-scaled from real radii so all planets remain visible.
//! The Sun is scaled down significantly, otherwise it would dwarf everything.

use crate::simulation::body::CelestialBody;
use glam::Vec3;

/// Convert a hex color (#RRGGBB) to [f32; 3] in 0.0–1.0 range.
const fn hex(r: u8, g: u8, b: u8) -> [f32; 3] {
    [r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0]
}

/// Compute a log-scaled display radius from real radius in km.
/// Tuned so Earth ≈ 1.0 display unit, Jupiter ≈ 2.0, Mercury ≈ 0.6.
fn display_radius(real_km: f64) -> f32 {
    // log₁₀(6371) ≈ 3.804 → we want ~1.0 for Earth
    let log_r = (real_km.log10() - 3.0) as f32;
    log_r.max(0.3) * 0.8
}

/// Create all solar system bodies with real orbital data.
pub fn create_solar_system() -> Vec<CelestialBody> {
    vec![
        // ☀ Sun — central star
        CelestialBody {
            name: "Sun",
            color: hex(255, 204, 51),        // warm yellow
            display_radius: 3.0,              // fixed large size
            real_radius_km: 695_700.0,
            semi_major_axis_au: 0.0,
            orbital_period_days: 1.0,         // not used
            inclination_rad: 0.0,
            start_angle_rad: 0.0,
            has_rings: false,
            is_star: true,
            texture_file: Some("sun.jpg"),
            position: Vec3::ZERO,
        },

        // ☿ Mercury
        CelestialBody {
            name: "Mercury",
            color: hex(181, 181, 181),        // #b5b5b5
            display_radius: display_radius(2_439.7),
            real_radius_km: 2_439.7,
            semi_major_axis_au: 0.387,
            orbital_period_days: 87.97,
            inclination_rad: 7.0_f64.to_radians(),
            start_angle_rad: 0.0,
            has_rings: false,
            is_star: false,
            texture_file: Some("mercury.jpg"),
            position: Vec3::ZERO,
        },

        // ♀ Venus
        CelestialBody {
            name: "Venus",
            color: hex(232, 205, 160),        // #e8cda0
            display_radius: display_radius(6_051.8),
            real_radius_km: 6_051.8,
            semi_major_axis_au: 0.723,
            orbital_period_days: 224.70,
            inclination_rad: 3.39_f64.to_radians(),
            start_angle_rad: 0.9,
            has_rings: false,
            is_star: false,
            texture_file: Some("venus.jpg"),
            position: Vec3::ZERO,
        },

        // 🜨 Earth
        CelestialBody {
            name: "Earth",
            color: hex(79, 163, 224),         // #4fa3e0
            display_radius: display_radius(6_371.0),
            real_radius_km: 6_371.0,
            semi_major_axis_au: 1.0,
            orbital_period_days: 365.25,
            inclination_rad: 0.0,             // reference plane
            start_angle_rad: 1.75,
            has_rings: false,
            is_star: false,
            texture_file: Some("earth.jpg"),
            position: Vec3::ZERO,
        },

        // ♂ Mars
        CelestialBody {
            name: "Mars",
            color: hex(193, 68, 14),          // #c1440e
            display_radius: display_radius(3_389.5),
            real_radius_km: 3_389.5,
            semi_major_axis_au: 1.524,
            orbital_period_days: 687.0,
            inclination_rad: 1.85_f64.to_radians(),
            start_angle_rad: 3.2,
            has_rings: false,
            is_star: false,
            texture_file: Some("mars.jpg"),
            position: Vec3::ZERO,
        },

        // ♃ Jupiter
        CelestialBody {
            name: "Jupiter",
            color: hex(200, 139, 58),         // #c88b3a
            display_radius: display_radius(69_911.0),
            real_radius_km: 69_911.0,
            semi_major_axis_au: 5.203,
            orbital_period_days: 4_332.59,
            inclination_rad: 1.31_f64.to_radians(),
            start_angle_rad: 4.8,
            has_rings: false,
            is_star: false,
            texture_file: Some("jupiter.jpg"),
            position: Vec3::ZERO,
        },

        // ♄ Saturn
        CelestialBody {
            name: "Saturn",
            color: hex(228, 209, 145),        // #e4d191
            display_radius: display_radius(58_232.0),
            real_radius_km: 58_232.0,
            semi_major_axis_au: 9.537,
            orbital_period_days: 10_759.22,
            inclination_rad: 2.49_f64.to_radians(),
            start_angle_rad: 5.5,
            has_rings: true,
            is_star: false,
            texture_file: Some("saturn.jpg"),
            position: Vec3::ZERO,
        },

        // ♅ Uranus
        CelestialBody {
            name: "Uranus",
            color: hex(125, 232, 232),        // #7de8e8
            display_radius: display_radius(25_362.0),
            real_radius_km: 25_362.0,
            semi_major_axis_au: 19.191,
            orbital_period_days: 30_688.5,
            inclination_rad: 0.77_f64.to_radians(),
            start_angle_rad: 2.1,
            has_rings: false,
            is_star: false,
            texture_file: Some("uranus.jpg"),
            position: Vec3::ZERO,
        },

        // ♆ Neptune
        CelestialBody {
            name: "Neptune",
            color: hex(63, 84, 186),          // #3f54ba
            display_radius: display_radius(24_622.0),
            real_radius_km: 24_622.0,
            semi_major_axis_au: 30.069,
            orbital_period_days: 60_182.0,
            inclination_rad: 1.77_f64.to_radians(),
            start_angle_rad: 0.4,
            has_rings: false,
            is_star: false,
            texture_file: Some("neptune.jpg"),
            position: Vec3::ZERO,
        },
    ]
}
