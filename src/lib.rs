mod engine;
mod tests;

mod lucas_game;

extern crate wasm_bindgen;

use wasm_bindgen::prelude::*;

use js_sys::{Float32Array, Uint16Array, ArrayBuffer, Uint8Array};

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_str(s: &str);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_f32_arr(s: Float32Array);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u8_arr(s: Uint8Array);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_u16_arr(s: Uint16Array);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_array_buffer(s: &ArrayBuffer);

    #[wasm_bindgen(js_namespace = console, js_name = log)]
    fn log_i32(s: i32);
}

#[wasm_bindgen(module="/js.js")]
extern "C" {
    fn log_json_string(s: &str);
    fn log_u8_as_f32_arr(x: Uint8Array);
    fn window_resize_listener(f: &Closure<dyn FnMut(i32,i32) -> ()>);
}