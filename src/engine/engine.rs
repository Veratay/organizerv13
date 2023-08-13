use instant::{Instant, Duration};
use wasm_bindgen::JsCast;

use super::{input::input_collector::InputCollector, render::{renderer::Renderer, camera::CameraController}};

pub struct Engine {
    pub input:InputCollector,
    pub renderer:Renderer,
    pub camera_controller:Option<CameraController>,
    last_time:Instant,
    pub dt:Duration
}

impl Engine {
    pub fn new() -> Self {
        let document = web_sys::window().unwrap().document().unwrap();
        let canvas: web_sys::HtmlCanvasElement = document.get_element_by_id("rootCanvas").unwrap().dyn_into::<web_sys::HtmlCanvasElement>().unwrap();

        Self {
            input: InputCollector::new(), 
            renderer: Renderer::new(canvas),
            camera_controller:Some(CameraController::new(1.0, 75.0)),
            last_time:Instant::now(),
            dt:Duration::new(0, 0)
        }
    }
    pub fn run(&mut self) {
        let dt = self.last_time.elapsed();
        self.last_time = Instant::now();

        self.input.process();
        if let Some(controller) = &mut self.camera_controller {
            controller.process_input(&self.input);
            controller.update_camera(self.renderer.camera_mut(), dt);
        }
        self.renderer.render();
        self.dt = dt;
    }
}