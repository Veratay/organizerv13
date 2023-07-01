mod engine;
use engine::render::{renderer::Renderer, types::rect::Rect};
use nalgebra::{Transform2, Matrix3};

extern crate wasm_bindgen;
use std::{ rc::Rc, cell::RefCell};

use wasm_bindgen::prelude::*;

use js_sys::{Float32Array, Uint16Array, ArrayBuffer};
use wasm_bindgen::{JsCast};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_str(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_f32_arr(s: Float32Array);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u16_arr(s: Uint16Array);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_array_buffer(s: &ArrayBuffer);
}

#[wasm_bindgen]
pub fn rust_hello_old() {
    log_str("Hey :)");
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("rootCanvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    let renderer = Rc::new(RefCell::new(Renderer::new(canvas)));
    Rect::new(Transform2::from_matrix_unchecked(Matrix3::new(
        0.5,0.0,0.5,
        0.0,0.5,0.0,
        0.0,0.0,1.0
    )), [0.0,0.0,1.0,1.0], renderer.clone());
    Rect::new(Transform2::from_matrix_unchecked(Matrix3::new(
        0.5,0.0,-0.5,
        0.0,0.5,0.0,
        0.0,0.0,1.0
    )), [0.0,0.5,0.5,1.0], renderer.clone());
    Rect::new(Transform2::from_matrix_unchecked(Matrix3::new(
        0.5,0.0,-0.5,
        0.0,0.5,0.5,
        0.0,0.0,1.0
    )), [1.0,0.0,0.0,1.0], renderer.clone());
    Rect::new(Transform2::from_matrix_unchecked(Matrix3::new(
        0.25,0.0,0.0,
        0.0,0.25,0.0,
        0.0,0.0,1.0
    )), [1.0,1.0,1.0,1.0], renderer.clone());
    renderer.borrow().render();
}