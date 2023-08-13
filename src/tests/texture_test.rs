use crate::{log_str, engine::{render::{types::image::Image, renderer::Renderer, texture::{RawTextureSource, TextureFilter}}, engine::Engine}};

extern crate wasm_bindgen;
use std::{ rc::Rc, cell::RefCell};

use cgmath::Matrix3;
use wasm_bindgen::{prelude::*, convert::IntoWasmAbi};

extern crate console_error_panic_hook;
use std::panic;

#[wasm_bindgen]
pub fn texture_test_func() -> JsValue {
    log_str("starting texture test");

    let mut engine = Engine::new();

    let mut img1 = Image::from_url( &mut engine.renderer, Matrix3::new(
        0.5,0.0,0.0,
        0.0,0.5,0.0,
        0.5,0.0,1.0
    ), String::from("./snout_stuff.png"),TextureFilter::Linear,TextureFilter::Linear);

    let mut img2 = Image::from_url( &mut engine.renderer, Matrix3::new(
                0.5,0.0,0.0,
                0.0,0.5,0.0,
                -0.5,0.0,1.0
            ), String::from("./sniff.jpeg"),TextureFilter::Linear,TextureFilter::Linear);

    let result = Closure::new(move || {
        
        img1.render(&mut engine.renderer, Matrix3::new(
            0.5,0.0,0.0,
            0.0,0.5,0.0,
            0.5,0.0,1.0
        ));
        img2.render(&mut engine.renderer,  Matrix3::new(
            0.5,0.0,0.0,    
            0.0,0.5,0.0,
            -0.5,0.0,1.0
        ));
        engine.run();

    });

    return result.into_js_value()
}