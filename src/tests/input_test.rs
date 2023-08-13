use std::{cell::RefCell, rc::Rc};
use wasm_bindgen::{prelude::{Closure, wasm_bindgen}, JsCast};

use gloo_utils::window;

use crate::engine::engine::Engine;

#[wasm_bindgen]
pub fn input_test() {
    let mut engine = Engine::new();

    let f = Rc::new(RefCell::new(None));
    let g = f.clone();

    *g.borrow_mut() = Some(Closure::new(move || {

        engine.run();
        
        request_animation_frame(f.borrow().as_ref().unwrap());
    }));

    request_animation_frame(g.borrow().as_ref().unwrap());
}

fn request_animation_frame(f: &Closure<dyn FnMut()>) {
    window()
        .request_animation_frame(f.as_ref().unchecked_ref())
        .expect("should register `requestAnimationFrame` OK");
}