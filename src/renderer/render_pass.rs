//! Render pass trait and concrete implementations.
//!
//! Each visual layer (planets, orbits, starfield, rings) is a self-contained
//! [`RenderPass`].  To add a new visual layer, create a struct that implements
//! the trait and register it in `Renderer::new`.

use glam::{Mat4, Vec3};
use web_sys::WebGl2RenderingContext as GL;

use super::shader::ShaderProgram;
use super::texture::TextureMap;
use crate::simulation::body::CelestialBody;

// ─── Shared per-frame context ────────────────────────────────────────────

/// Read-only snapshot of everything a render pass needs for one frame.
pub struct FrameContext<'a> {
    pub gl: &'a GL,
    pub view: Mat4,
    pub projection: Mat4,
    pub eye_position: Vec3,
    pub time: f32,
}

/// A self-contained render pass.
///
/// Implement this trait to add new visual layers (e.g. asteroid belts,
/// labels, trails) without touching existing rendering code.
pub trait RenderPass {
    fn draw(&self, ctx: &FrameContext, bodies: &[CelestialBody]);
}

// ─── Planet pass ─────────────────────────────────────────────────────────

pub struct PlanetPass {
    pub shader: ShaderProgram,
    pub vao: web_sys::WebGlVertexArrayObject,
    pub index_count: i32,
    pub textures: TextureMap,
}

impl RenderPass for PlanetPass {
    fn draw(&self, ctx: &FrameContext, bodies: &[CelestialBody]) {
        let gl = ctx.gl;
        let s = &self.shader;
        s.activate(gl);

        let textures = self.textures.borrow();
        gl.bind_vertex_array(Some(&self.vao));

        // Frame-constant uniforms — set once outside the loop
        s.set_mat4(gl, "u_view", &ctx.view);
        s.set_mat4(gl, "u_projection", &ctx.projection);
        s.set_vec3(gl, "u_light_pos", &[0.0, 0.0, 0.0]);
        s.set_vec3(
            gl,
            "u_view_pos",
            &[ctx.eye_position.x, ctx.eye_position.y, ctx.eye_position.z],
        );

        for body in bodies {
            let model = Mat4::from_translation(body.position)
                * Mat4::from_scale(Vec3::splat(body.display_radius));
            let normal_matrix = model.inverse().transpose();

            s.set_mat4(gl, "u_model", &model);
            s.set_mat4(gl, "u_normal_matrix", &normal_matrix);
            s.set_vec3(gl, "u_color", &body.color);
            s.set_bool(gl, "u_is_star", body.is_star);

            // Texture binding
            let has_texture = textures.contains_key(body.name);
            s.set_bool(gl, "u_has_texture", has_texture);
            if has_texture {
                gl.active_texture(GL::TEXTURE0);
                gl.bind_texture(GL::TEXTURE_2D, textures.get(body.name));
                s.set_int(gl, "u_texture", 0);
            }

            gl.draw_elements_with_i32(GL::TRIANGLES, self.index_count, GL::UNSIGNED_SHORT, 0);
        }

        gl.bind_texture(GL::TEXTURE_2D, None);
        gl.bind_vertex_array(None);
    }
}

// ─── Ring pass ───────────────────────────────────────────────────────────

pub struct RingPass {
    pub shader: ShaderProgram,
    pub vao: web_sys::WebGlVertexArrayObject,
    pub index_count: i32,
}

impl RenderPass for RingPass {
    fn draw(&self, ctx: &FrameContext, bodies: &[CelestialBody]) {
        let gl = ctx.gl;
        let s = &self.shader;
        s.activate(gl);

        gl.bind_vertex_array(Some(&self.vao));

        // Frame-constant uniforms
        s.set_mat4(gl, "u_view", &ctx.view);
        s.set_mat4(gl, "u_projection", &ctx.projection);

        // Disable culling for rings (double-sided)
        gl.disable(GL::CULL_FACE);

        for body in bodies.iter().filter(|b| b.has_rings) {
            let model = Mat4::from_translation(body.position)
                * Mat4::from_scale(Vec3::splat(body.display_radius));

            s.set_mat4(gl, "u_model", &model);
            s.set_vec3(gl, "u_color", &body.color);

            gl.draw_elements_with_i32(GL::TRIANGLES, self.index_count, GL::UNSIGNED_SHORT, 0);
        }

        gl.enable(GL::CULL_FACE);
        gl.bind_vertex_array(None);
    }
}

// ─── Orbit pass ──────────────────────────────────────────────────────────

pub struct OrbitPass {
    pub shader: ShaderProgram,
    pub vaos: Vec<(web_sys::WebGlVertexArrayObject, i32)>,
}

impl RenderPass for OrbitPass {
    fn draw(&self, ctx: &FrameContext, bodies: &[CelestialBody]) {
        let gl = ctx.gl;
        let s = &self.shader;
        s.activate(gl);

        s.set_mat4(gl, "u_view", &ctx.view);
        s.set_mat4(gl, "u_projection", &ctx.projection);

        // Orbits are centred on the Sun — translate them by its current position
        // so they follow the galactic drift.
        let sun_pos = bodies
            .iter()
            .find(|b| b.is_star)
            .map(|b| b.position)
            .unwrap_or(Vec3::ZERO);
        let model = Mat4::from_translation(sun_pos);
        s.set_mat4(gl, "u_model", &model);

        let planets: Vec<&CelestialBody> = bodies.iter().filter(|b| !b.is_star).collect();
        for (i, planet) in planets.iter().enumerate() {
            if let Some((vao, count)) = self.vaos.get(i) {
                s.set_vec3(gl, "u_color", &planet.color);
                gl.bind_vertex_array(Some(vao));
                gl.draw_arrays(GL::LINE_STRIP, 0, *count);
                gl.bind_vertex_array(None);
            }
        }
    }
}

// ─── Starfield pass ──────────────────────────────────────────────────────

pub struct StarfieldPass {
    pub shader: ShaderProgram,
    pub vao: web_sys::WebGlVertexArrayObject,
    pub count: i32,
}

impl RenderPass for StarfieldPass {
    fn draw(&self, ctx: &FrameContext, bodies: &[CelestialBody]) {
        let _ = bodies; // starfield is independent of bodies
        let gl = ctx.gl;
        let s = &self.shader;
        s.activate(gl);

        // Don't write to depth buffer — stars are a backdrop
        gl.depth_mask(false);

        // Skybox-style: strip translation from the view matrix
        let mut sky_view = ctx.view;
        sky_view.w_axis.x = 0.0;
        sky_view.w_axis.y = 0.0;
        sky_view.w_axis.z = 0.0;

        s.set_mat4(gl, "u_view", &sky_view);
        s.set_mat4(gl, "u_projection", &ctx.projection);
        s.set_float(gl, "u_time", ctx.time);

        gl.bind_vertex_array(Some(&self.vao));
        gl.draw_arrays(GL::POINTS, 0, self.count);
        gl.bind_vertex_array(None);

        gl.depth_mask(true);
    }
}
