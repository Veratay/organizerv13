use crate::{log_str, engine::{render::types::line::{Line,EndBehavior}, engine::Engine}};

use cgmath::{Vector4, Vector2};

extern crate wasm_bindgen;
use std::{ rc::Rc, cell::RefCell};

use wasm_bindgen::prelude::*;

extern crate console_error_panic_hook;
use std::panic;

#[wasm_bindgen]
pub fn line_test_func() -> JsValue {
    log_str("starting line test");
    panic::set_hook(Box::new(console_error_panic_hook::hook));
    let mut engine = Engine::new();

    let mut line =  Line::new(&mut engine.renderer,[
        Vector2::new(0.0, 0.0),Vector2::new(0.5, 0.0)
    ],Vector4::new(1.0,0.0,0.0,1.0), 0.01, 0.01, EndBehavior::Clipped);

    let mut line2 =  Line::new(&mut engine.renderer,[
        Vector2::new(0.5, 0.5),Vector2::new(0.0, 0.0)
    ],Vector4::new(1.0,0.0,0.0,1.0),0.01,0.01, EndBehavior::Clipped);

    let result = Closure::new(move || {
        
        line.render();
        line2.render();
        line2.update_points(&mut engine.renderer, Vector2::new(0.0,0.0), engine.input.mouse_pos);
        engine.run();

    });

    return result.into_js_value()
}