//! WebGL2 renderer — orchestrates the draw pipeline.
//!
//! Sub-modules handle the individual concerns:
//! - [`camera`]      — orbital camera controller
//! - [`shader`]      — GLSL compilation & uniform helpers
//! - [`mesh`]        — CPU mesh generation & GPU upload
//! - [`starfield`]   — procedural background stars
//! - [`texture`]     — async image → GPU texture loading
//! - [`render_pass`] — `RenderPass` trait & concrete implementations

pub mod camera;
pub mod mesh;
pub mod render_pass;
pub mod shader;
pub mod starfield;
pub mod texture;

use camera::Camera;
use mesh::{create_line_vao, create_mesh_vao, create_trail_vao};
use render_pass::{
    FrameContext, OrbitPass, PlanetPass, RenderPass, RingPass, StarfieldPass,
    TrailPass, TrailBuffer,
};
use shader::ShaderProgram;
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
const TRAIL_VERT: &str = include_str!("../../shaders/trail.vert");
const TRAIL_FRAG: &str = include_str!("../../shaders/trail.frag");

// ─── Renderer ────────────────────────────────────────────────────────────

pub struct Renderer {
    gl: GL,
    pub camera: Camera,

    /// Ordered render passes — drawn front-to-back each frame.
    passes: Vec<Box<dyn RenderPass>>,

    /// Textures (populated asynchronously, shared via Rc).
    textures: TextureMap,

    /// Accumulated time for shader animations.
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
        // ── Compile shader programs ──

        let planet_shader = ShaderProgram::new(
            &gl,
            PLANET_VERT,
            PLANET_FRAG,
            &[
                "u_model",
                "u_view",
                "u_projection",
                "u_normal_matrix",
                "u_color",
                "u_light_pos",
                "u_view_pos",
                "u_is_star",
                "u_has_texture",
                "u_texture",
            ],
        )?;
        let orbit_shader = ShaderProgram::new(
            &gl,
            ORBIT_VERT,
            ORBIT_FRAG,
            &["u_view", "u_projection", "u_color"],
        )?;
        let star_shader = ShaderProgram::new(
            &gl,
            STAR_VERT,
            STAR_FRAG,
            &["u_view", "u_projection", "u_time"],
        )?;
        let ring_shader = ShaderProgram::new(
            &gl,
            RING_VERT,
            RING_FRAG,
            &["u_model", "u_view", "u_projection", "u_color"],
        )?;
        let trail_shader = ShaderProgram::new(
            &gl,
            TRAIL_VERT,
            TRAIL_FRAG,
            &["u_view", "u_projection", "u_color"],
        )?;

        // ── Generate & upload meshes ──

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

        // Trail VAOs (one per non-star body)
        let mut trail_buffers = Vec::new();
        for _body in bodies.iter().filter(|b| !b.is_star) {
            let (vao, vbo_pos, vbo_alpha) =
                create_trail_vao(&gl, crate::constants::TRAIL_MAX_POINTS)?;
            trail_buffers.push(TrailBuffer {
                positions: std::collections::VecDeque::new(),
                vao,
                vbo_pos,
                vbo_alpha,
            });
        }

        let aspect = canvas_width as f32 / canvas_height as f32;
        let camera = Camera::new(aspect);

        // Global GL state
        gl.enable(GL::DEPTH_TEST);
        gl.enable(GL::CULL_FACE);
        gl.cull_face(GL::BACK);
        gl.enable(GL::BLEND);
        gl.blend_func(GL::SRC_ALPHA, GL::ONE_MINUS_SRC_ALPHA);
        gl.clear_color(0.04, 0.04, 0.1, 1.0);

        let textures: TextureMap = Rc::new(RefCell::new(HashMap::new()));

        // ── Assemble render passes (order matters!) ──

        let passes: Vec<Box<dyn RenderPass>> = vec![
            Box::new(StarfieldPass {
                shader: star_shader,
                vao: star_vao,
                count: star_count,
            }),
            Box::new(OrbitPass {
                shader: orbit_shader,
                vaos: orbit_vaos,
            }),
            Box::new(TrailPass {
                shader: trail_shader,
                trails: trail_buffers,
            }),
            Box::new(PlanetPass {
                shader: planet_shader,
                vao: planet_vao,
                index_count: planet_index_count,
                textures: Rc::clone(&textures),
            }),
            Box::new(RingPass {
                shader: ring_shader,
                vao: ring_vao,
                index_count: ring_index_count,
            }),
        ];

        Ok(Self {
            gl,
            camera,
            passes,
            textures,
            render_time: 0.0,
        })
    }

    // ── Public API ──

    /// Render one complete frame by iterating over all registered passes.
    pub fn render(&mut self, bodies: &[CelestialBody], dt: f32) {
        self.render_time += dt;
        let gl = &self.gl;

        gl.clear(GL::COLOR_BUFFER_BIT | GL::DEPTH_BUFFER_BIT);

        let ctx = FrameContext {
            gl,
            view: self.camera.view_matrix(),
            projection: self.camera.projection_matrix(),
            eye_position: self.camera.eye_position(),
            time: self.render_time,
        };

        for pass in &mut self.passes {
            pass.draw(&ctx, bodies);
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
}
