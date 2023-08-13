use crate::{log_str, engine::render::{types::image::Image, renderer::Renderer}};

use nalgebra::{Transform2, Matrix3};

extern crate wasm_bindgen;
use std::{ rc::Rc, cell::RefCell};

use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;
use std::panic;

#[wasm_bindgen]
pub fn texture_update_test() {
    log_str("starting texture update test");
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("rootCanvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let mut renderer = Renderer::new(canvas);

    let mut images = Vec::new();

    let image_count = 26;
    let vertical_scale = 2.0;

    for i in 0..image_count {
        let scale = 1.0/image_count as f32;
        let pos = -1.0 + scale + scale*2.0*i as f32;
        images.push(Image::from_url(&mut renderer, Transform2::from_matrix_unchecked(Matrix3::new(
            scale,0.0,pos,
            0.0,scale*vertical_scale,0.0,
            0.0,0.0,1.0
        )), format!("./assets/sniff_{}.jpeg",i+1)))
    }

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

    *g.borrow_mut() = Some(Closure::new(move || {
        
        for (i,img) in images.iter_mut().enumerate() {
            let scale = 1.0/image_count as f32;
            let pos = -1.0 + scale + scale*2.0*i as f32;
            img.render(&mut renderer, Transform2::from_matrix_unchecked(Matrix3::new(
                scale,0.0,pos,
                0.0,scale*vertical_scale,0.0,
                0.0,0.0,1.0
            )));
        }
        renderer.render();

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