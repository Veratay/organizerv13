use std::{collections::HashSet, rc::Rc, sync::Mutex};

use cgmath::Vector2;
use gloo_events::EventListener;
use gloo_utils::window;
use wasm_bindgen::{JsCast, UnwrapThrowExt, prelude::Closure};
use web_sys::{Event, KeyboardEvent, MouseEvent};

use crate::{log_str, window_resize_listener};

#[derive(Debug)]
pub struct InputCollector {
    listener_output: Rc<Mutex<ListenerOutput>>,
    pub keys_pressed: HashSet<String>,
    pub mouse_buttons_pressed:Vec<i16>,
    pub mouse_pos_delta: Vector2<f32>,
    pub mouse_pos: Vector2<f32>,
    pub screen_size: Vector2<i32>,
    _listeners: Vec<EventListener>, //needs to be stored so that they drop properly
    _screen_listener: Closure<dyn FnMut(i32,i32)>
}

#[derive(Debug)]
struct ListenerOutput {
    focused: bool,
    keys_down: Vec<String>,
    keys_up: Vec<String>,
    mouse_pos: Vector2<i32>,
    mouse_buttons_down: Vec<i16>,
    mouse_buttons_up: Vec<i16>,
    screen_size: Vector2<i32>
}

impl ListenerOutput {
    fn new(screen_size:Vector2<i32>) -> Self {
        Self {
            focused: true,
            keys_down: Vec::new(),
            keys_up: Vec::new(),
            mouse_pos: Vector2 { x: 0, y: 0 },
            mouse_buttons_down: Vec::new(),
            mouse_buttons_up: Vec::new(),
            screen_size:screen_size
        }
    }
}

impl InputCollector {
    pub fn new() -> Self {
        let screen_size = Vector2 { x: window().inner_width().unwrap().as_f64().unwrap() as i32, y: window().inner_height().unwrap().as_f64().unwrap() as i32 };
        let listener_output = Rc::new(Mutex::new(ListenerOutput::new(screen_size)));

        let listener_output_clone = listener_output.clone();
        let key_down_callback = move |e: &Event| {
            let e: &KeyboardEvent = e.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
            listener_output_clone.lock().unwrap().keys_down.push(e.code());
        };

        let listener_output_clone = listener_output.clone();
        let key_up_callback = move |e: &Event| {
            let e: &KeyboardEvent = e.dyn_ref::<web_sys::KeyboardEvent>().unwrap_throw();
            listener_output_clone.lock().unwrap().keys_up.push(e.code());
        };

        let listener_output_clone = listener_output.clone();
        let focus_callback = move |_: &Event| {
            listener_output_clone.lock().unwrap().focused = true;
        };

        let listener_output_clone = listener_output.clone();
        let blur_callback = move |_: &Event| {
            listener_output_clone.lock().unwrap().focused = false;
        };

        let listener_output_clone = listener_output.clone();
        let mouse_move_callback = move |e: &Event| {
            let e: &MouseEvent = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
            listener_output_clone.lock().unwrap().mouse_pos = Vector2 { x: e.x(), y: e.y() };
        };

        let listener_output_clone = listener_output.clone();
        let mouse_up_callback = move |e: &Event| {
            let e: &MouseEvent = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
            listener_output_clone.lock().unwrap().mouse_buttons_up.push(e.button());
        };

        let listener_output_clone = listener_output.clone();
        let mouse_down_callback = move |e: &Event| {
            let e: &MouseEvent = e.dyn_ref::<web_sys::MouseEvent>().unwrap_throw();
            listener_output_clone.lock().unwrap().mouse_buttons_down.push(e.button());
        };

        let listener_output_clone = listener_output.clone();
        let resize_callback  = move |w:i32, h:i32| {
            listener_output_clone.lock().unwrap().screen_size = Vector2::new(w, h);
        };

        let window = window();
        window.focus().unwrap_throw();

        let key_down_listener = EventListener::new(&window, "keydown", key_down_callback);
        let key_up_listener = EventListener::new(&window, "keyup", key_up_callback);
        let focus_listener = EventListener::new(&window, "focus", focus_callback);
        let blur_listener = EventListener::new(&window, "blur", blur_callback);
        let mouse_move_listener = EventListener::new(&window, "mousemove", mouse_move_callback);
        let mouse_up_listener = EventListener::new(&window, "mouseup", mouse_up_callback);
        let mouse_down_listener: EventListener = EventListener::new(&window, "mousedown", mouse_down_callback);
        
        let resize_closure =   Closure::new(resize_callback);
        window_resize_listener(&resize_closure);

        Self {
            listener_output: listener_output,
            keys_pressed: HashSet::new(),
            mouse_buttons_pressed: Vec::new(),
            mouse_pos_delta:Vector2 { x: 0.0, y: 0.0 },
            mouse_pos: Vector2 { x: 0.0, y: 0.0 },
            screen_size: screen_size.cast().unwrap(),
            _listeners: vec![
                key_down_listener,
                key_up_listener,
                focus_listener,
                blur_listener,
                mouse_move_listener,
                mouse_up_listener,
                mouse_down_listener
            ],
            _screen_listener: resize_closure
        }
    }

    pub fn process(&mut self) {
        let mut output = self.listener_output.lock().unwrap();
        if !output.focused {
            self.keys_pressed.clear();
            return;
        }

        for down in output.keys_down.iter() {
            self.keys_pressed.insert(down.clone());
        }

        for up in output.keys_up.iter() {
            self.keys_pressed.remove(up);
        }

        for down in output.mouse_buttons_down.iter() {
            self.mouse_buttons_pressed.push(*down);
        }

        for up in output.mouse_buttons_up.iter() {
            self.mouse_buttons_pressed.retain(|x| x!= up);
        }

        //normalizes x and y to be between -1.0 and 1.0, with a cartesian orientation
        let new:Vector2<f32> =  Vector2 { x: (output.mouse_pos.x as f32 / self.screen_size.x as f32 * 2.0) -1.0, y: 1.0 - (output.mouse_pos.y as f32 / self.screen_size.y as f32 * 2.0)};
        self.mouse_pos_delta = new - self.mouse_pos;
        self.mouse_pos = new;

        self.screen_size = output.screen_size;

       // let mut listener_output = self.listener_output.lock().unwrap();

       //log_str(&format!("processed inputs: {:?}, mouse_d:{:?}, mouse_pos:{:?}, buttons pressed:{:?}",&self.keys_pressed,self.mouse_pos_delta,self.mouse_pos,self.mouse_buttons_pressed));

        output.keys_down.clear();
        output.keys_up.clear();
        output.mouse_buttons_down.clear();
        output.mouse_buttons_up.clear();

    }
}
