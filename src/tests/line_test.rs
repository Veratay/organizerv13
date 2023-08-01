use crate::{log_str, engine::render::types::line::{Line,EndBehavior}};

use nalgebra::Vector2;

extern crate wasm_bindgen;
use std::{ rc::Rc, cell::RefCell};

use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;
use std::panic;

use super::make_renderer;

#[wasm_bindgen]
pub fn line_test() {
    log_str("starting texture test");
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut renderer = make_renderer();

    let mut line =  Line::new(&mut renderer,[
        Vector2::new(0.0, 0.0),Vector2::new(0.5, 0.0)
    ],[1.0,0.0,0.0,1.0],0.01,0.01, EndBehavior::Clipped);

    let mut line2 =  Line::new(&mut renderer,[
        Vector2::new(0.0, 0.0),Vector2::new(0.5, 0.0)
    ],[1.0,0.0,0.0,1.0],0.01,0.01, EndBehavior::Rounded);

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
        line.render();
        line2.render();
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