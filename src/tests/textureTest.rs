use crate::{log_str, engine::render::{types::image::Image, renderer::Renderer, texture::RawTextureSource}};

use nalgebra::{Transform2, Matrix3};

extern crate wasm_bindgen;
use std::{ rc::Rc, cell::RefCell};

use wasm_bindgen::prelude::*;

use js_sys::{Float32Array, Uint16Array, ArrayBuffer};
use wasm_bindgen::{JsCast};

#[wasm_bindgen]
pub fn texture_test() {
    log_str("starting texture test");
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("rootCanvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let renderer = Rc::new(RefCell::new(Renderer::new(canvas)));

    let mapped = renderer.borrow_mut().upload_texture(&RawTextureSource {
        data:&[0u8, 255u8, 0u8, 255u8],
        format:crate::engine::render::texture::TextureFormat::RGBA,
        width:1,
        height:1,
        unique:false
    });

    Image::from_mapped(Transform2::from_matrix_unchecked(Matrix3::new(
                0.5,0.0,0.5,
                0.0,0.5,0.0,
                0.0,0.0,1.0
            )), mapped, renderer.clone());
    renderer.borrow().render();

    // Here we want to call `requestAnimationFrame` in a loop, but only a fixed
    // number of times. After it's done we want all our resources cleaned up. To
    // achieve this we're using an `Rc`. The `Rc` will eventually store the
    // closure we want to execute on each frame, but to start out it contains
    // `None`.
    //
    // After the `Rc` is made we'll actually create the closure, and the closure
    // will reference one of the `Rc` instances. The other `Rc` reference is
    // used to store the closure, request the first frame, and then is dropped
    // by this function.
    //
    // Inside the closure we've got a persistent `Rc` reference, which we use
    // for all future iterations of the loop
    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    let mut i = 0;
    *g.borrow_mut() = Some(Closure::new(move || {

        // Set the body's text content to how many times this
        // requestAnimationFrame callback has fired.
        i += 1;
        
        renderer.borrow().render();

        // Schedule ourself for another requestAnimationFrame callback.
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}

fn window() -> web_sys::Window {
    web_sys::window().expect("no global `window` exists")
}