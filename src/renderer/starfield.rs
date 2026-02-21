//! Procedural starfield (background sky).
//!
//! Generates randomly distributed point-stars on a large sphere.

use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

use crate::constants::*;

/// Create the starfield VAO and return `(vao, point_count)`.
pub fn create_starfield(gl: &GL) -> Result<(web_sys::WebGlVertexArrayObject, i32), JsValue> {
    create_starfield_custom(gl, STARFIELD_COUNT, STARFIELD_RADIUS)
}

/// Create a starfield with custom count and radius.
pub fn create_starfield_custom(
    gl: &GL,
    count: usize,
    radius: f32,
) -> Result<(web_sys::WebGlVertexArrayObject, i32), JsValue> {
    use rand::Rng;

    let mut rng = rand::rng();
    let mut data = Vec::with_capacity(count * 4);

    for _ in 0..count {
        let theta: f32 = rng.random::<f32>() * std::f32::consts::TAU;
        let phi: f32 = (rng.random::<f32>() * 2.0 - 1.0).acos();

        data.push(radius * phi.sin() * theta.cos()); // x
        data.push(radius * phi.cos());                // y
        data.push(radius * phi.sin() * theta.sin()); // z
        data.push(rng.random::<f32>() * 0.7 + 0.3);  // brightness
    }

    let vao = gl
        .create_vertex_array()
        .ok_or_else(|| JsValue::from_str("Failed to create VAO"))?;
    gl.bind_vertex_array(Some(&vao));

    let vbo = gl
        .create_buffer()
        .ok_or_else(|| JsValue::from_str("Failed to create VBO"))?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));

    unsafe {
        let array = js_sys::Float32Array::view(&data);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &array, GL::STATIC_DRAW);
    }

    let stride = 4 * 4; // 4 floats × 4 bytes
    gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
    gl.enable_vertex_attrib_array(0);
    gl.vertex_attrib_pointer_with_i32(1, 1, GL::FLOAT, false, stride, 3 * 4);
    gl.enable_vertex_attrib_array(1);

    gl.bind_vertex_array(None);
    Ok((vao, count as i32))
}
