//! Shader compilation and uniform helper functions.
//!
//! All WebGL shader boilerplate lives here so the renderer stays focused
//! on the render pipeline rather than low-level GPU plumbing.

use glam::Mat4;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

// ─── Shader compilation ──────────────────────────────────────────────────

/// Compile a single GLSL shader (vertex or fragment).
pub fn compile_shader(gl: &GL, shader_type: u32, source: &str) -> Result<web_sys::WebGlShader, JsValue> {
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

/// Compile and link a vertex + fragment shader pair into a program.
pub fn compile_program(gl: &GL, vert_src: &str, frag_src: &str) -> Result<web_sys::WebGlProgram, JsValue> {
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

    // Individual shaders can be freed after linking.
    gl.delete_shader(Some(&vert));
    gl.delete_shader(Some(&frag));

    Ok(program)
}

// ─── Uniform setters ────────────────────────────────────────────────────

pub fn set_uniform_mat4(gl: &GL, program: &web_sys::WebGlProgram, name: &str, mat: &Mat4) {
    let loc = gl.get_uniform_location(program, name);
    gl.uniform_matrix4fv_with_f32_array(loc.as_ref(), false, &mat.to_cols_array());
}

pub fn set_uniform_vec3(gl: &GL, program: &web_sys::WebGlProgram, name: &str, v: &[f32; 3]) {
    let loc = gl.get_uniform_location(program, name);
    gl.uniform3f(loc.as_ref(), v[0], v[1], v[2]);
}

pub fn set_uniform_float(gl: &GL, program: &web_sys::WebGlProgram, name: &str, val: f32) {
    let loc = gl.get_uniform_location(program, name);
    gl.uniform1f(loc.as_ref(), val);
}

pub fn set_uniform_bool(gl: &GL, program: &web_sys::WebGlProgram, name: &str, val: bool) {
    let loc = gl.get_uniform_location(program, name);
    gl.uniform1i(loc.as_ref(), val as i32);
}

pub fn set_uniform_int(gl: &GL, program: &web_sys::WebGlProgram, name: &str, val: i32) {
    let loc = gl.get_uniform_location(program, name);
    gl.uniform1i(loc.as_ref(), val);
}
