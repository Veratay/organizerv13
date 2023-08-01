mod engine;
mod tests;
use engine::render::{renderer::Renderer};
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

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_i32(s: i32);
}