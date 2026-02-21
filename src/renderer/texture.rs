//! Asynchronous texture loading from image URLs.
//!
//! Each planet texture is loaded via an `HtmlImageElement`. The image
//! reference is captured directly in the onload closure — no hidden DOM
//! elements or `get_element_by_id` hacks needed.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext as GL;

use crate::simulation::body::CelestialBody;

/// Shared handle to the texture map so multiple closures can insert into it.
pub type TextureMap = Rc<RefCell<HashMap<String, web_sys::WebGlTexture>>>;

/// Shared counter for tracking how many textures have finished loading.
type LoadedCounter = Rc<RefCell<usize>>;

/// Load a single texture from `url` and store it under `body_name`.
pub fn load_texture_async(
    gl: &GL,
    textures: &TextureMap,
    body_name: &str,
    url: &str,
    loaded_count: LoadedCounter,
    total: usize,
) {
    let image = Rc::new(web_sys::HtmlImageElement::new().unwrap());
    image.set_cross_origin(Some("anonymous"));

    let gl_clone = gl.clone();
    let textures_clone = Rc::clone(textures);
    let name = body_name.to_string();
    let image_ref = Rc::clone(&image);
    let loaded_ok = Rc::clone(&loaded_count);

    let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
        let gl = &gl_clone;
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));

        gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            GL::RGBA,
            GL::UNSIGNED_BYTE,
            &image_ref,
        )
        .unwrap();

        gl.generate_mipmap(GL::TEXTURE_2D);

        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_S, GL::REPEAT as i32);
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_WRAP_T, GL::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(
            GL::TEXTURE_2D,
            GL::TEXTURE_MIN_FILTER,
            GL::LINEAR_MIPMAP_LINEAR as i32,
        );
        gl.tex_parameteri(GL::TEXTURE_2D, GL::TEXTURE_MAG_FILTER, GL::LINEAR as i32);

        gl.bind_texture(GL::TEXTURE_2D, None);

        textures_clone.borrow_mut().insert(name.clone(), texture);
        log::info!("🌍 Texture loaded: {}", name);

        // ── Splash progress ──
        crate::splash::update_step(&format!("tex-{}", name), "done");
        let mut count = loaded_ok.borrow_mut();
        *count += 1;
        if *count >= total {
            crate::splash::hide_splash();
        }
    }) as Box<dyn FnMut(web_sys::Event)>);

    image.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    // Handle load errors so the splash still completes
    let name_err = body_name.to_string();
    let loaded_err = Rc::clone(&loaded_count);
    let onerror = Closure::wrap(Box::new(move |_: web_sys::Event| {
        log::warn!("⚠️ Failed to load texture: {}", name_err);
        crate::splash::update_step(&format!("tex-{}", name_err), "done");
        let mut count = loaded_err.borrow_mut();
        *count += 1;
        if *count >= total {
            crate::splash::hide_splash();
        }
    }) as Box<dyn FnMut(web_sys::Event)>);
    image.set_onerror(Some(onerror.as_ref().unchecked_ref()));
    onerror.forget();

    image.set_src(url);
}

/// Kick off asynchronous texture loading for every body that has a texture file.
pub fn start_loading_textures(gl: &GL, textures: &TextureMap, bodies: &[CelestialBody]) {
    let total = bodies.iter().filter(|b| b.texture_file.is_some()).count();
    let loaded_count: LoadedCounter = Rc::new(RefCell::new(0));

    for body in bodies {
        if let Some(file) = body.texture_file {
            crate::splash::update_step(&format!("tex-{}", body.name), "loading");
            let url = format!("textures/{file}");
            load_texture_async(gl, textures, body.name, &url, Rc::clone(&loaded_count), total);
        }
    }
}
