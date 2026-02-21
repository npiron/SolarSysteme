//! Asynchronous texture loading from image URLs.
//!
//! Each planet texture is loaded via a hidden `<img>` element in the DOM.
//! On load, the image data is uploaded to a WebGL texture with mipmaps.

use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::WebGl2RenderingContext as GL;

use crate::simulation::body::CelestialBody;

/// Shared handle to the texture map so multiple closures can insert into it.
pub type TextureMap = Rc<RefCell<HashMap<String, web_sys::WebGlTexture>>>;

/// Load a single texture from `url` and store it under `body_name`.
pub fn load_texture_async(gl: &GL, textures: &TextureMap, body_name: &str, url: &str) {
    let image = web_sys::HtmlImageElement::new().unwrap();
    image.set_cross_origin(Some("anonymous"));

    let gl_clone = gl.clone();
    let textures_clone = Rc::clone(textures);
    let name = body_name.to_string();

    let onload = Closure::wrap(Box::new(move |_: web_sys::Event| {
        let image: web_sys::HtmlImageElement = {
            let window = web_sys::window().unwrap();
            let document = window.document().unwrap();
            document
                .get_element_by_id(&format!("tex-{}", name))
                .unwrap()
                .dyn_into::<web_sys::HtmlImageElement>()
                .unwrap()
        };

        let gl = &gl_clone;
        let texture = gl.create_texture().unwrap();
        gl.bind_texture(GL::TEXTURE_2D, Some(&texture));

        gl.tex_image_2d_with_u32_and_u32_and_html_image_element(
            GL::TEXTURE_2D,
            0,
            GL::RGBA as i32,
            GL::RGBA,
            GL::UNSIGNED_BYTE,
            &image,
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

        // Clean up the hidden image element
        image.remove();
    }) as Box<dyn FnMut(web_sys::Event)>);

    // Temporary hidden <img> so the closure can find it by id
    let window = web_sys::window().unwrap();
    let document = window.document().unwrap();
    let body = document.body().unwrap();
    image.set_id(&format!("tex-{body_name}"));
    image.set_attribute("style", "display:none").unwrap();
    body.append_child(&image).unwrap();

    image.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();

    image.set_src(url);
}

/// Kick off asynchronous texture loading for every body that has a texture file.
pub fn start_loading_textures(gl: &GL, textures: &TextureMap, bodies: &[CelestialBody]) {
    for body in bodies {
        if let Some(file) = body.texture_file {
            let url = format!("textures/{file}");
            load_texture_async(gl, textures, body.name, &url);
        }
    }
}
