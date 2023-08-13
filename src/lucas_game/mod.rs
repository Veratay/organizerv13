use cgmath::{Vector4, Vector2, Vector3, Matrix4, Deg, Point3};
use instant::Duration;
use wasm_bindgen::JsValue;

use crate::{engine::{render::{types::image::Image, texture::TextureFilter, renderer::Renderer}, engine::Engine, input::input_collector::InputCollector}, log_str};

use wasm_bindgen::prelude::*;

use js_sys::Math::random;

#[wasm_bindgen]
pub fn lucas_game() -> JsValue {
    log_str("starting texture test");

    let mountain_counts = vec![4,3,2,1];
    let mountians_pos: Vec<Vector3<f32>> = vec![Vector3::new(0.0,-5.0,-8.0),Vector3::new(0.0,-5.0,-16.0),Vector3::new(0.0,-5.0,-32.0),Vector3::new(0.0,-0.0,-40.0)];
    let mountains_width = vec![2.0,6.0,15.0,25.0];

    let mut engine = Engine::new();
    engine.renderer.set_clear_color(Vector4::new(0.431, 0.647, 1.0, 1.0));
    engine.camera_controller = None;

    let mut mountains = Vec::new();
    for i in 0..mountain_counts.len() {
        for j in -mountain_counts[i]..mountain_counts[i]+1 {
            mountains.push(Image::from_url(&mut engine.renderer, 
                Matrix4::from_translation(Vector3::new(j as f32*2.0*mountains_width[i], mountians_pos[i].y, mountians_pos[i].z)) * 
                Matrix4::from_angle_y(Deg(if j%2 == 0 { 180.0 } else {0.0})) * 
                Matrix4::from_scale(mountains_width[i])
                
            , String::from("./assets/lucas_background.png"),TextureFilter::Linear,TextureFilter::Linear));
        }
    }

    let mut platform_gen = PlatformGenerator::new();

    platform_gen.gen_layer(&mut engine.renderer);

    let mut player = Player::new(Vector2::new(platform_gen.layers.last().unwrap().1.last().unwrap().pos.x, -0.5),&mut engine.renderer);

    let mut cloud_gen = CloudGenerator::new(player.pos.extend(0.0), &mut engine.renderer);


    //let mut platform = Platform::new(Vector2::new(0.0, 0.0), &mut engine.renderer);
    

    let result = Closure::new(move || {
        
        for mountain in mountains.iter_mut() {
            mountain.render_unchanged(&mut engine.renderer);
        };

        platform_gen.process(engine.renderer.camera_mut().position.y, &mut engine.renderer);
        platform_gen.render(&mut engine.renderer);

        cloud_gen.process(player.pos.extend(0.0),&engine.dt,&mut engine.renderer);
        cloud_gen.render(&mut engine.renderer);

        let pos = player.pos;

        if pos.y < -1.0 {
            platform_gen.reset();
            platform_gen.gen_layer(&mut engine.renderer);
            player.pos = platform_gen.layers.last().unwrap().1.last().unwrap().pos;
            player.vertical_vel = 0.0;
            
        }
        let b = (-1000.0,Vec::new());
        let closest_layer = platform_gen.layers.iter().fold(&b, |a,x| if f32::abs(a.0-pos.y) > f32::abs(x.0-pos.y) { x } else { a} );
        let closest_platform = closest_layer.1.iter().fold(None, |a: Option<&Platform>,x| 
            if let Some(a) = a {
                if f32::abs(a.pos.x-pos.x) > f32::abs(x.pos.x-pos.x) { Some(x) } else { Some(a) }
            } else {
                Some(x)
            }
        ).unwrap();

        let diff = (closest_platform.pos-pos);
        let x = f32::abs(diff.x) < 0.2 && f32::abs(diff.y) < 0.05;

        player.process(&engine.input,&engine.dt,x);
        player.render(&mut engine.renderer);

        if engine.input.keys_pressed.contains("KeyR") {
            log_str(&format!("renderer:{:#?}",&mut engine.renderer))
        }

        engine.renderer.camera_mut().position = Point3::new(player.pos.x, player.pos.y, 1.0);
        engine.run();

    });

    return result.into_js_value()
}
struct PlatformGenerator {
    next_layer_y:f32,
    layers:Vec<(f32,Vec<Platform>)>
}

const LAYER_PLATFORM_PROBABILITY:[f32; 3] = [0.5,0.4,0.1];
const LAYER_MAX_HEIGHT:f32 = 0.5;
const LAYER_HEIGHT_VARIATION:f32 = 0.05;
const PLATFORM_MIN_SAME_LAYER_DISTANCE:f32 = 0.3;
const PLATFORM_MAX_DIFF_LAYER_DISTANCE:f32 = 0.6;
const LAYER_GENERATION_DIST:f32 = 1.2;
const LAYER_DROP_DIST:f32 = -1.2;
const LAYER_START:f32 = -0.5;
impl PlatformGenerator {
    fn new() -> Self {
        Self { next_layer_y: LAYER_START, layers: Vec::new() }
    }

    fn process(&mut self, current_height:f32, renderer:&mut Renderer) {
        while self.layers.last().unwrap_or(&(0.0,Vec::new())).0-current_height < LAYER_GENERATION_DIST {
            self.gen_layer(renderer);
        }

        while self.layers.first().unwrap().0 - current_height < LAYER_DROP_DIST && self.layers.len() > 1 {
            self.layers.remove(0);
        }
    }

    fn reset(&mut self) {
        self.layers.clear();
        self.next_layer_y = LAYER_START
    }

    fn gen_layer(&mut self, renderer:&mut Renderer) {
        let y = self.next_layer_y;
        self.next_layer_y += LAYER_MAX_HEIGHT - LAYER_HEIGHT_VARIATION * random() as f32;

        let mut platforms: Vec<Platform> = Vec::new();
        let platform_count = get_weighted_index(random() as f32, &LAYER_PLATFORM_PROBABILITY) + 1;

        for _ in 0..platform_count {
            let mut pos = random() as f32 * 2.0 -1.0;
            while platforms.iter().find(|p| f32::abs( p.pos.x - pos) < PLATFORM_MIN_SAME_LAYER_DISTANCE).is_some() {
                pos = random() as f32 * 2.0 -1.0;
            }

            platforms.push(Platform::new(Vector2 { x: pos, y: y }, renderer));
        }

        if !self.layers.is_empty() {
            let i = platforms.last_mut().unwrap();
            let a = self.layers.last().unwrap().1.iter().find(|p| f32::abs( p.pos.x - i.pos.x) < PLATFORM_MAX_DIFF_LAYER_DISTANCE);
            if a.is_none() {
                i.pos.x = self.layers.last().unwrap().1.last().unwrap().pos.x;
            }
        }

        self.layers.push((y, platforms))
    }

    fn render(&mut self, renderer:&mut Renderer) {
        for p in self.layers.iter_mut().map(|x| &mut x.1).flatten() {
            p.render(renderer);
        }
    }
}

fn get_weighted_index(random_float: f32, probabilities: &[f32]) -> usize {
    let mut cumulative_probability = 0.0;

    for (index, &probability) in probabilities.iter().enumerate() {
        cumulative_probability += probability;
        if random_float <= cumulative_probability {
            return index;
        }
    }

    probabilities.len() - 1 // Fallback: return the last index if not found earlier
}

struct Platform {
    pos:Vector2<f32>,
    obj:Image
}

const PLATFORM_SCALE:Vector2<f32> = Vector2 { x:0.1, y:0.05};
impl Platform {
    fn new(pos:Vector2<f32>, renderer:&mut Renderer) -> Self {
        let obj = Image::from_url(renderer,
            Matrix4::from_translation(pos.extend(0.0)) * Matrix4::from_nonuniform_scale(PLATFORM_SCALE.x, PLATFORM_SCALE.y, 1.0),
            String::from("./assets/lucas_platform.png"),
            TextureFilter::Nearest,TextureFilter::Nearest
        );

        Self { pos: pos, obj: obj }
    }

    fn render(&mut self, renderer:&mut Renderer) {
        self.obj.render(renderer,Matrix4::from_translation(self.pos.extend(0.0)) * Matrix4::from_nonuniform_scale(PLATFORM_SCALE.x, PLATFORM_SCALE.y, 1.0));
    }
}

const PLAYER_SCALE:Vector2<f32> = Vector2 { x:0.1, y:0.05};
const PLAYER_SPEED:f32 = 0.0015;
const JUMP_VEL:f32 = 1.2;
const JUMP_DECAY:f32 = 1.1;
struct Player {
    pos:Vector2<f32>,
    obj:Image,
    vertical_vel:f32,
}

impl Player {
    fn new(pos:Vector2<f32>, renderer:&mut Renderer) -> Self {
        let obj = Image::from_url(renderer,
            Matrix4::from_translation(pos.extend(0.01)) * Matrix4::from_nonuniform_scale(PLAYER_SCALE.x, PLAYER_SCALE.y, 1.0),
            String::from("./assets/lucas_strawberry.png"),
            TextureFilter::Nearest,TextureFilter::Nearest
        );
        Self { pos: pos, obj: obj, vertical_vel:0.0 }
    }

    fn process(&mut self, input_collector:&InputCollector, dt:&Duration, colliding:bool) {
        for key in input_collector.keys_pressed.iter() {
            match key.as_str() {
                // "KeyW" | "ArrowUp" => {
                //     self.pos.y += PLAYER_SPEED * dt.as_millis() as f32;
                // }
                
                "KeyA" | "ArrowLeft" => {
                    self.pos.x -= PLAYER_SPEED * dt.as_millis() as f32;
                }
                "KeyD" | "ArrowRight" => {
                    self.pos.x += PLAYER_SPEED * dt.as_millis() as f32;
                }
                "Space" => {
                    if colliding && self.vertical_vel <= 0.0 { self.vertical_vel = JUMP_VEL; }
                }
                // "ShiftLeft" => {
                //     self.amount_down = amount;
                //     true
                // }
                _ => {
                    
                }
            }
        }
        self.vertical_vel -= JUMP_DECAY * dt.as_millis() as f32 / 1000.0;
        if colliding && self.vertical_vel < 0.0{
            self.vertical_vel = 0.0;
        } else { self.pos.y += self.vertical_vel * dt.as_millis() as f32 / 1000.0;}
    }

    fn render(&mut self, renderer:&mut Renderer) {
        self.obj.render(renderer, Matrix4::from_translation(self.pos.extend(0.01))  * Matrix4::from_nonuniform_scale(PLAYER_SCALE.x, PLAYER_SCALE.y, 1.0));
    }
}

const CLOUD_LIMIT:usize = 100;
const CLOUD_BOUNDING_SIZE:Vector3<f32> = Vector3 {x:20.0, y: 20.0, z:5.0};
const CLOUD_SPEED:f32 = 1.0;
const CLOUD_SPEED_VARIATION:f32 = 0.5;
const CLOUD_SCALE:Vector2<f32> = Vector2 {x:1.0, y:1.0};
const CLOUD_SCALE_VARIATION:f32 = 0.2;
const CLOUD_MIN_DIST:f32 = 3.0;
const CLOUD_X_START:f32 = -10.0;
struct CloudGenerator {
    clouds:Vec<Cloud>
}

impl CloudGenerator {
    fn new(pos:Vector3<f32>, renderer:&mut Renderer) -> Self {
        let mut result = Self {
            clouds:Vec::new()
        };
        while result.clouds.len() < CLOUD_LIMIT  {
            let pos = pos + Vector3 { x: CLOUD_X_START, y: CLOUD_BOUNDING_SIZE.y*(random()*2.0-1.0)as f32, z: -CLOUD_BOUNDING_SIZE.z*random() as f32 - CLOUD_MIN_DIST };
            let speed = CLOUD_SPEED + CLOUD_SPEED_VARIATION*(random()*2.0-1.0)as f32;
            let scale = CLOUD_SCALE * CLOUD_SCALE_VARIATION*random() as f32;
            result.clouds.push(Cloud::new(pos, speed, scale, renderer))
        }

        result
    }

    fn process(&mut self, pos:Vector3<f32>, dt:&Duration, renderer:&mut Renderer) {

        while self.clouds.len() < CLOUD_LIMIT  {
            let pos = pos + Vector3 { x: CLOUD_X_START, y: CLOUD_BOUNDING_SIZE.y*(random()*2.0-1.0)as f32, z: -CLOUD_BOUNDING_SIZE.z*random() as f32 - CLOUD_MIN_DIST };
            let speed = CLOUD_SPEED + CLOUD_SPEED_VARIATION*(random()*2.0-1.0)as f32;
            let scale = CLOUD_SCALE * (1.0-(CLOUD_SCALE_VARIATION*random() as f32));
            self.clouds.push(Cloud::new(pos, speed, scale, renderer))
        }

        self.clouds.retain_mut(|i| {
            i.process(dt);
            let diff = (i.pos - pos);

            if diff.x.abs() > CLOUD_BOUNDING_SIZE.x || diff.y.abs() > CLOUD_BOUNDING_SIZE.y || diff.z.abs() > CLOUD_BOUNDING_SIZE.z {
                return false;
            } else {
                return true;
            }
        });
    }

    fn render(&mut self, renderer:&mut Renderer) {
        for i in self.clouds.iter_mut() {
            i.render(renderer);
        }
    }
}

struct Cloud {
    obj:Image,
    speed:f32,
    scale:Vector2<f32>,
    pos:Vector3<f32>
}

impl Cloud {
    fn new(pos:Vector3<f32>, speed:f32, scale:Vector2<f32>, renderer:&mut Renderer) -> Self {
        let obj =Image::from_url(renderer,
            Matrix4::from_translation(pos) * Matrix4::from_nonuniform_scale(scale.x, scale.y, 1.0),
            String::from("./assets/lucas_cloud.png"),
            TextureFilter::Nearest,TextureFilter::Nearest
        );

        Self { obj: obj, speed: speed, scale:scale, pos: pos }
    }
    fn process(&mut self, dt:&Duration) {
        self.pos.x += self.speed * dt.as_millis() as f32/1000.0;
    }

    fn render(&mut self, renderer:&mut Renderer) {
        self.obj.render(renderer, Matrix4::from_translation(self.pos) * Matrix4::from_nonuniform_scale(self.scale.x, self.scale.y, 1.0));
    }
}