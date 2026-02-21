//! Shader compilation, uniform caching, and helper functions.
//!
//! [`ShaderProgram`] wraps a `WebGlProgram` and caches uniform locations
//! so they are resolved once at init instead of every draw call.

use std::collections::HashMap;

use glam::Mat4;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

// ─── Shader compilation ──────────────────────────────────────────────────

/// Compile a single GLSL shader (vertex or fragment).
fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<web_sys::WebGlShader, JsValue> {
    let shader = gl
        .create_shader(shader_type)
        .ok_or_else(|| JsValue::from_str("Failed to create shader"))?;
    gl.shader_source(&shader, source);
    gl.compile_shader(&shader);

    if !gl
        .get_shader_parameter(&shader, GL::COMPILE_STATUS)
        .as_bool()
        .unwrap_or(false)
    {
        let info = gl.get_shader_info_log(&shader).unwrap_or_default();
        gl.delete_shader(Some(&shader));
        return Err(JsValue::from_str(&format!("Shader compile error: {info}")));
    }
    Ok(shader)
}

// ─── ShaderProgram ───────────────────────────────────────────────────────

/// A compiled+linked WebGL program with cached uniform locations.
pub struct ShaderProgram {
    program: web_sys::WebGlProgram,
    /// Uniform name → location (resolved once at creation).
    locations: HashMap<&'static str, Option<web_sys::WebGlUniformLocation>>,
}

impl ShaderProgram {
    /// Compile, link, and pre-resolve a list of uniform names.
    pub fn new(
        gl: &GL,
        vert_src: &str,
        frag_src: &str,
        uniform_names: &[&'static str],
    ) -> Result<Self, JsValue> {
        let vert = compile_shader(gl, GL::VERTEX_SHADER, vert_src)?;
        let frag = compile_shader(gl, GL::FRAGMENT_SHADER, frag_src)?;

        let program = gl
            .create_program()
            .ok_or_else(|| JsValue::from_str("Failed to create program"))?;
        gl.attach_shader(&program, &vert);
        gl.attach_shader(&program, &frag);
        gl.link_program(&program);

        if !gl
            .get_program_parameter(&program, GL::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            let info = gl.get_program_info_log(&program).unwrap_or_default();
            return Err(JsValue::from_str(&format!("Program link error: {info}")));
        }

        gl.delete_shader(Some(&vert));
        gl.delete_shader(Some(&frag));

        // Pre-resolve all uniform locations
        let mut locations = HashMap::with_capacity(uniform_names.len());
        for &name in uniform_names {
            let loc = gl.get_uniform_location(&program, name);
            locations.insert(name, loc);
        }

        Ok(Self { program, locations })
    }

    /// Bind this program for use.
    pub fn activate(&self, gl: &GL) {
        gl.use_program(Some(&self.program));
    }

    /// Get a cached uniform location (returns `None` for inactive/optimized-out uniforms).
    fn loc(&self, name: &str) -> Option<&web_sys::WebGlUniformLocation> {
        self.locations.get(name).and_then(|o| o.as_ref())
    }

    // ── Typed uniform setters ──

    pub fn set_mat4(&self, gl: &GL, name: &str, mat: &Mat4) {
        if let Some(loc) = self.loc(name) {
            gl.uniform_matrix4fv_with_f32_array(Some(loc), false, &mat.to_cols_array());
        }
    }

    pub fn set_vec3(&self, gl: &GL, name: &str, v: &[f32; 3]) {
        if let Some(loc) = self.loc(name) {
            gl.uniform3f(Some(loc), v[0], v[1], v[2]);
        }
    }

    pub fn set_float(&self, gl: &GL, name: &str, val: f32) {
        if let Some(loc) = self.loc(name) {
            gl.uniform1f(Some(loc), val);
        }
    }

    pub fn set_bool(&self, gl: &GL, name: &str, val: bool) {
        if let Some(loc) = self.loc(name) {
            gl.uniform1i(Some(loc), val as i32);
        }
    }

    pub fn set_int(&self, gl: &GL, name: &str, val: i32) {
        if let Some(loc) = self.loc(name) {
            gl.uniform1i(Some(loc), val);
        }
    }
}
