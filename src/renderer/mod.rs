//! WebGL2 renderer — orchestrates the draw pipeline.
//!
//! Sub-modules handle the individual concerns:
//! - [`camera`]    — orbital camera controller
//! - [`shader`]    — GLSL compilation & uniform helpers
//! - [`mesh`]      — CPU mesh generation & GPU upload
//! - [`starfield`] — procedural background stars
//! - [`texture`]   — async image → GPU texture loading

pub mod camera;
pub mod mesh;
pub mod shader;
pub mod starfield;
pub mod texture;

use camera::Camera;
use glam::{Mat4, Vec3};
use mesh::{create_line_vao, create_mesh_vao};
use shader::*;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use texture::TextureMap;
use wasm_bindgen::JsValue;
use web_sys::WebGl2RenderingContext as GL;

use crate::simulation::body::CelestialBody;
use crate::simulation::orbit;

// ─── Shader sources (loaded from files at compile time) ──────────────────

const PLANET_VERT: &str = include_str!("../../shaders/planet.vert");
const PLANET_FRAG: &str = include_str!("../../shaders/planet.frag");
const ORBIT_VERT: &str = include_str!("../../shaders/orbit.vert");
const ORBIT_FRAG: &str = include_str!("../../shaders/orbit.frag");
const STAR_VERT: &str = include_str!("../../shaders/star.vert");
const STAR_FRAG: &str = include_str!("../../shaders/star.frag");
const RING_VERT: &str = include_str!("../../shaders/ring.vert");
const RING_FRAG: &str = include_str!("../../shaders/ring.frag");

// ─── Renderer ────────────────────────────────────────────────────────────

pub struct Renderer {
    gl: GL,
    pub camera: Camera,

    // Shader programs
    planet_program: web_sys::WebGlProgram,
    orbit_program: web_sys::WebGlProgram,
    star_program: web_sys::WebGlProgram,
    ring_program: web_sys::WebGlProgram,

    // Geometry
    planet_vao: web_sys::WebGlVertexArrayObject,
    planet_index_count: i32,
    ring_vao: web_sys::WebGlVertexArrayObject,
    ring_index_count: i32,
    star_vao: web_sys::WebGlVertexArrayObject,
    star_count: i32,
    orbit_vaos: Vec<(web_sys::WebGlVertexArrayObject, i32)>,

    // Textures (populated asynchronously)
    textures: TextureMap,

    // Accumulated time for shader animations
    render_time: f32,
}

impl Renderer {
    /// Create the renderer: compile shaders, upload meshes, configure GL state.
    pub fn new(
        gl: GL,
        canvas_width: u32,
        canvas_height: u32,
        bodies: &[CelestialBody],
    ) -> Result<Self, JsValue> {
        // Compile shader programs
        let planet_program = compile_program(&gl, PLANET_VERT, PLANET_FRAG)?;
        let orbit_program = compile_program(&gl, ORBIT_VERT, ORBIT_FRAG)?;
        let star_program = compile_program(&gl, STAR_VERT, STAR_FRAG)?;
        let ring_program = compile_program(&gl, RING_VERT, RING_FRAG)?;

        // Generate & upload meshes
        let sphere = mesh::generate_sphere();
        let planet_vao = create_mesh_vao(&gl, &sphere)?;
        let planet_index_count = sphere.indices.len() as i32;

        let ring_mesh = mesh::generate_ring();
        let ring_vao = create_mesh_vao(&gl, &ring_mesh)?;
        let ring_index_count = ring_mesh.indices.len() as i32;

        let (star_vao, star_count) = starfield::create_starfield(&gl)?;

        // Orbit line VAOs (one per non-star body)
        let mut orbit_vaos = Vec::new();
        for body in bodies.iter().filter(|b| !b.is_star) {
            let path = orbit::generate_orbit_path(body.semi_major_axis_au, body.inclination_rad);
            let vao = create_line_vao(&gl, &path)?;
            orbit_vaos.push((vao, path.len() as i32));
        }

        let aspect = canvas_width as f32 / canvas_height as f32;
        let camera = Camera::new(aspect);

        // Global GL state
        gl.enable(GL::DEPTH_TEST);
        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        gl.clear_color(0.04, 0.04, 0.1, 1.0);

        let textures = Rc::new(RefCell::new(HashMap::new()));

        Ok(Self {
            gl,
            camera,
            planet_program,
            orbit_program,
            star_program,
            ring_program,
            planet_vao,
            planet_index_count,
            ring_vao,
            ring_index_count,
            star_vao,
            star_count,
            orbit_vaos,
            textures,
            render_time: 0.0,
        })
    }

    // ── Public API ──

    /// Render one complete frame.
    pub fn render(&mut self, bodies: &[CelestialBody], dt: f32) {
        self.render_time += dt;
        let gl = &self.gl;

        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let view = self.camera.view_matrix();
        let proj = self.camera.projection_matrix();
        let eye_pos = self.camera.eye_position();

        self.draw_starfield(&view, &proj);
        self.draw_orbits(bodies, &view, &proj);

        for body in bodies {
            self.draw_planet(body, &view, &proj, eye_pos);
            if body.has_rings {
                self.draw_ring(body, &view, &proj);
            }
        }
    }

    /// Handle canvas resize.
    pub fn resize(&mut self, width: u32, height: u32) {
        self.gl.viewport(0, 0, width as i32, height as i32);
        self.camera.set_aspect(width as f32 / height as f32);
    }

    /// Clone of the GL context for external use (e.g. texture loading).
    pub fn gl_handle(&self) -> GL {
        self.gl.clone()
    }

    /// Shared handle to the texture map.
    pub fn textures_handle(&self) -> TextureMap {
        Rc::clone(&self.textures)
    }

    // ── Private draw passes ──

    fn draw_planet(&self, body: &CelestialBody, view: &Mat4, proj: &Mat4, eye_pos: Vec3) {
        let gl = &self.gl;
        gl.use_program(Some(&self.planet_program));

        let model = Mat4::from_translation(body.position)
            * Mat4::from_scale(Vec3::splat(body.display_radius));
        let normal_matrix = model.inverse().transpose();

        set_uniform_mat4(gl, &self.planet_program, "u_model", &model);
        set_uniform_mat4(gl, &self.planet_program, "u_view", view);
        set_uniform_mat4(gl, &self.planet_program, "u_projection", proj);
        set_uniform_mat4(gl, &self.planet_program, "u_normal_matrix", &normal_matrix);
        set_uniform_vec3(gl, &self.planet_program, "u_color", &body.color);
        set_uniform_vec3(gl, &self.planet_program, "u_light_pos", &[0.0, 0.0, 0.0]);
        set_uniform_vec3(
            gl,
            &self.planet_program,
            "u_view_pos",
            &[eye_pos.x, eye_pos.y, eye_pos.z],
        );
        set_uniform_bool(gl, &self.planet_program, "u_is_star", body.is_star);

        // Texture binding
        let textures = self.textures.borrow();
        let has_texture = textures.contains_key(body.name);
        set_uniform_bool(gl, &self.planet_program, "u_has_texture", has_texture);
        if has_texture {
            gl.active_texture(GL::TEXTURE0);
            gl.bind_texture(GL::TEXTURE_2D, textures.get(body.name));
            set_uniform_int(gl, &self.planet_program, "u_texture", 0);
        }

        gl.bind_vertex_array(Some(&self.planet_vao));
        gl.draw_elements_with_i32(GL::TRIANGLES, self.planet_index_count, GL::UNSIGNED_SHORT, 0);
        gl.bind_vertex_array(None);

        if has_texture {
            gl.bind_texture(GL::TEXTURE_2D, None);
        }
    }

    fn draw_ring(&self, body: &CelestialBody, view: &Mat4, proj: &Mat4) {
        let gl = &self.gl;
        gl.use_program(Some(&self.ring_program));

        let model = Mat4::from_translation(body.position)
            * Mat4::from_scale(Vec3::splat(body.display_radius));

        set_uniform_mat4(gl, &self.ring_program, "u_model", &model);
        set_uniform_mat4(gl, &self.ring_program, "u_view", view);
        set_uniform_mat4(gl, &self.ring_program, "u_projection", proj);
        set_uniform_vec3(gl, &self.ring_program, "u_color", &body.color);

        gl.bind_vertex_array(Some(&self.ring_vao));
        gl.draw_elements_with_i32(GL::TRIANGLES, self.ring_index_count, GL::UNSIGNED_SHORT, 0);
        gl.bind_vertex_array(None);
    }

    fn draw_orbits(&self, bodies: &[CelestialBody], view: &Mat4, proj: &Mat4) {
        let gl = &self.gl;
        gl.use_program(Some(&self.orbit_program));

        set_uniform_mat4(gl, &self.orbit_program, "u_view", view);
        set_uniform_mat4(gl, &self.orbit_program, "u_projection", proj);

        let planets: Vec<&CelestialBody> = bodies.iter().filter(|b| !b.is_star).collect();
        for (i, planet) in planets.iter().enumerate() {
            if let Some((vao, count)) = self.orbit_vaos.get(i) {
                set_uniform_vec3(gl, &self.orbit_program, "u_color", &planet.color);
                gl.bind_vertex_array(Some(vao));
                gl.draw_arrays(GL::LINE_STRIP, 0, *count);
                gl.bind_vertex_array(None);
            }
        }
    }

    fn draw_starfield(&self, view: &Mat4, proj: &Mat4) {
        let gl = &self.gl;
        gl.use_program(Some(&self.star_program));

        // Skybox-style: remove translation from the view matrix
        let mut sky_view = *view;
        sky_view.w_axis.x = 0.0;
        sky_view.w_axis.y = 0.0;
        sky_view.w_axis.z = 0.0;

        set_uniform_mat4(gl, &self.star_program, "u_view", &sky_view);
        set_uniform_mat4(gl, &self.star_program, "u_projection", proj);
        set_uniform_float(gl, &self.star_program, "u_time", self.render_time);

        gl.bind_vertex_array(Some(&self.star_vao));
        gl.draw_arrays(GL::POINTS, 0, self.star_count);
        gl.bind_vertex_array(None);
    }
}
