//! Mesh generation (sphere, ring) and GPU upload utilities.
//!
//! A [`Mesh`] holds CPU-side vertex + index data. Upload it with
//! [`create_mesh_vao`] to get a ready-to-draw VAO on the GPU.

use glam::Vec3;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

use crate::constants::*;

// ─── CPU-side mesh ──────────────────────────────────────────────────────

/// Interleaved vertex data (pos.xyz + norm.xyz + uv.xy) and u16 indices.
pub struct Mesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<u16>,
}

// ─── Sphere ──────────────────────────────────────────────────────────────

/// Generate a UV-sphere with the default segment/ring counts from [`constants`].
pub fn generate_sphere() -> Mesh {
    generate_sphere_custom(SPHERE_SEGMENTS, SPHERE_RINGS)
}

/// Generate a UV-sphere with custom resolution.
pub fn generate_sphere_custom(segments: u32, rings: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for y in 0..=rings {
        let v = y as f32 / rings as f32;
        let phi = v * std::f32::consts::PI;

        for x in 0..=segments {
            let u = x as f32 / segments as f32;
            let theta = u * std::f32::consts::TAU;

            let px = phi.sin() * theta.cos();
            let py = phi.cos();
            let pz = phi.sin() * theta.sin();

            // Position
            vertices.push(px);
            vertices.push(py);
            vertices.push(pz);
            // Normal (same as position on a unit sphere)
            vertices.push(px);
            vertices.push(py);
            vertices.push(pz);
            // UV (equirectangular)
            vertices.push(u);
            vertices.push(v);
        }
    }

    for y in 0..rings {
        for x in 0..segments {
            let a = y * (segments + 1) + x;
            let b = a + segments + 1;

            indices.push(a as u16);
            indices.push(b as u16);
            indices.push((a + 1) as u16);

            indices.push((a + 1) as u16);
            indices.push(b as u16);
            indices.push((b + 1) as u16);
        }
    }

    Mesh { vertices, indices }
}

// ─── Ring (annulus) ─────────────────────────────────────────────────────

/// Generate ring geometry with the default radii from [`constants`].
pub fn generate_ring() -> Mesh {
    generate_ring_custom(RING_INNER_RADIUS, RING_OUTER_RADIUS, RING_SEGMENTS)
}

/// Generate a flat annulus in the XZ plane with custom parameters.
pub fn generate_ring_custom(inner: f32, outer: f32, segments: u32) -> Mesh {
    let mut vertices = Vec::new();
    let mut indices = Vec::new();

    for i in 0..=segments {
        let angle = (i as f32 / segments as f32) * std::f32::consts::TAU;
        let cos_a = angle.cos();
        let sin_a = angle.sin();

        // Inner vertex (pos + normal up)
        vertices.push(inner * cos_a);
        vertices.push(0.0);
        vertices.push(inner * sin_a);
        vertices.push(0.0);
        vertices.push(1.0);
        vertices.push(0.0);

        // Outer vertex (pos + normal up)
        vertices.push(outer * cos_a);
        vertices.push(0.0);
        vertices.push(outer * sin_a);
        vertices.push(0.0);
        vertices.push(1.0);
        vertices.push(0.0);
    }

    for i in 0..segments {
        let base = i * 2;
        indices.push(base as u16);
        indices.push((base + 1) as u16);
        indices.push((base + 2) as u16);

        indices.push((base + 1) as u16);
        indices.push((base + 3) as u16);
        indices.push((base + 2) as u16);
    }

    Mesh { vertices, indices }
}

// ─── GPU upload ──────────────────────────────────────────────────────────

/// Upload a [`Mesh`] (interleaved pos+norm+uv) to a WebGL VAO.
pub fn create_mesh_vao(gl: &GL, mesh: &Mesh) -> Result<web_sys::WebGlVertexArrayObject, JsValue> {
    let vao = gl
        .create_vertex_array()
        .ok_or_else(|| JsValue::from_str("Failed to create VAO"))?;
    gl.bind_vertex_array(Some(&vao));

    // Vertex buffer
    let vbo = gl
        .create_buffer()
        .ok_or_else(|| JsValue::from_str("Failed to create VBO"))?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));

    unsafe {
        let vert_array = js_sys::Float32Array::view(&mesh.vertices);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &vert_array, GL::STATIC_DRAW);
    }

    let stride = 8 * 4; // 8 floats × 4 bytes (pos.xyz + norm.xyz + uv.xy)

    // location 0 — position
    gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, stride, 0);
    gl.enable_vertex_attrib_array(0);

    // location 1 — normal
    gl.vertex_attrib_pointer_with_i32(1, 3, GL::FLOAT, false, stride, 3 * 4);
    gl.enable_vertex_attrib_array(1);

    // location 2 — uv
    gl.vertex_attrib_pointer_with_i32(2, 2, GL::FLOAT, false, stride, 6 * 4);
    gl.enable_vertex_attrib_array(2);

    // Index buffer
    let ibo = gl
        .create_buffer()
        .ok_or_else(|| JsValue::from_str("Failed to create IBO"))?;
    gl.bind_buffer(GL::ELEMENT_ARRAY_BUFFER, Some(&ibo));

    unsafe {
        let idx_array = js_sys::Uint16Array::view(&mesh.indices);
        gl.buffer_data_with_array_buffer_view(GL::ELEMENT_ARRAY_BUFFER, &idx_array, GL::STATIC_DRAW);
    }

    gl.bind_vertex_array(None);
    Ok(vao)
}

/// Upload a line-strip (Vec3 positions) to a WebGL VAO.
pub fn create_line_vao(gl: &GL, points: &[Vec3]) -> Result<web_sys::WebGlVertexArrayObject, JsValue> {
    let vao = gl
        .create_vertex_array()
        .ok_or_else(|| JsValue::from_str("Failed to create VAO"))?;
    gl.bind_vertex_array(Some(&vao));

    let data: Vec<f32> = points.iter().flat_map(|p| [p.x, p.y, p.z]).collect();

    let vbo = gl
        .create_buffer()
        .ok_or_else(|| JsValue::from_str("Failed to create VBO"))?;
    gl.bind_buffer(GL::ARRAY_BUFFER, Some(&vbo));

    unsafe {
        let array = js_sys::Float32Array::view(&data);
        gl.buffer_data_with_array_buffer_view(GL::ARRAY_BUFFER, &array, GL::STATIC_DRAW);
    }

    gl.vertex_attrib_pointer_with_i32(0, 3, GL::FLOAT, false, 0, 0);
    gl.enable_vertex_attrib_array(0);

    gl.bind_vertex_array(None);
    Ok(vao)
}
