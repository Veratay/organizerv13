use wasm_bindgen::JsCast;

use crate::engine::render::renderer::Renderer;

// mod texture_test;
mod bezier_test;
//mod line_test;
// mod texture_update_test;
mod input_test;

fn make_renderer() -> Renderer {
    let document = web_sys::window().unwrap().document().unwrap();
    let canvas = document.get_element_by_id("rootCanvas").unwrap();
    let canvas: web_sys::HtmlCanvasElement = canvas.dyn_into::<web_sys::HtmlCanvasElement>().unwrap();
    Renderer::new(canvas)
}