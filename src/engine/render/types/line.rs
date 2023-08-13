use std::{rc::Rc,f32::consts::{FRAC_PI_4,FRAC_PI_2, SQRT_2, PI}};

use cgmath::{Vector2, Vector4};

use crate::{engine::render::{render_object::{RenderType, VertexAttrib, ShaderDataTypes, RenderObject, AttributeRole}, renderer::{Renderer, VertexData}}, log_str};

thread_local! {
    static LINE_RENDER_TYPE: Rc<RenderType> = Rc::new(RenderType::new_batched_growable(
        String::from(
            "#version 300 es

            uniform mat4 view;
            uniform mat4 projection;
            
            in vec2 pos;
            in vec4 vColor;
            in float vThickness;
            in float vSmooth;
            in vec2 points1;
            in vec2 points2;

            out vec2 posf;
            out vec2 sf;
            out vec2 controlf;
            out vec2 ef;
            out float thicknessf;
            out vec4 color;
            out float fsmooth;
    
            void main() {
                gl_Position = vec4(pos,0.0,1.0);
                posf = gl_Position.xy;
                sf = points1;
                ef =points2;
                thicknessf = vThickness;
                color = vColor;
                fsmooth = vSmooth;
            }"
        ),
        String::from(
            "# version 300 es
            precision highp float;
            // start point
            in vec2 sf;
            // end point
            in vec2 ef;
            // current position in fragment shader
            in vec2 posf;
            // thickness of the curve
            in float thicknessf;
            in float fsmooth;

            in vec4 color;
    
            out vec4 FragColor;
    
            float dot2( in vec2 v ) { return dot(v,v); }
    
            // borrowed from here https://www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm
            float sdSegment( in vec2 p, in vec2 a, in vec2 b )
            {
                vec2 pa = p-a, ba = b-a;
                float h = clamp( dot(pa,ba)/dot(ba,ba), 0.0, 1.0 );
                return length( pa - ba*h );
            }
    
            void main() {
                vec4 result = color;
                float d = sdSegment(posf, sf, ef) - thicknessf;
                float s = smoothstep(0., fsmooth, -d);
                if (d < 0.) {
                    result.a = s;
                } else {
                    discard;
                }
                FragColor = result;
            }"
        ),
        vec![
            VertexAttrib { 
                name: String::from("pos"), 
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FloatVec3, 
            },
            VertexAttrib {
                name: String::from("vTexcoord"),
                role: AttributeRole::Custom,
                data_type: ShaderDataTypes::FloatVec2
            }
            VertexAttrib {
                name: String::from("vColor"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FloatVec4,
            },
            VertexAttrib {
                name:String::from("vThickness"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT,
            },
            VertexAttrib { 
                name: String::from("points1"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FloatVec2, 
            }, 
            VertexAttrib { 
                name: String::from("points2"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FloatVec2, 
            }, 
            VertexAttrib { 
                name: String::from("vSmooth"), 
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT, 
            },
        ], 
        Vec::new(), 
        Vec::new(),
        20, 
        1000, 
        30, 
        1500, 
        2.0, 
        2.0
    ));
}

pub struct Line {
    obj:RenderObject,
    end_behavior:EndBehavior,
    smooth:f32,
    thickness:f32
}

pub enum EndBehavior {
    Clipped,
    Rounded
}

impl Line {
    pub fn new(renderer:&mut Renderer, points:[Vector2<f32>; 2], color:Vector4<f32>, thickness:f32, smooth:f32, end_behavior:EndBehavior) -> Self {
        
        let theta = (points[0].y-points[1].y).atan2(points[0].x-points[1].x);

        let t0 = match end_behavior {
                EndBehavior::Rounded => theta+FRAC_PI_2+FRAC_PI_4,
                EndBehavior::Clipped => theta+FRAC_PI_2
            };
        let t1 = match end_behavior {
            EndBehavior::Rounded => theta+FRAC_PI_4,
            EndBehavior::Clipped => theta+FRAC_PI_2
        };
        let offset = thickness + smooth;
    
        let c = match end_behavior {
            EndBehavior::Clipped => offset,
            EndBehavior::Rounded => offset*SQRT_2
        };

        let p0 = points[0] + Vector2::new(f32::cos(t0)*c, f32::sin(t0)*c);
        let p1 = points[1] + Vector2::new(f32::cos(t1)*c, f32::sin(t1)*c);
        let p2 = points[1] + Vector2::new(f32::cos(t0+PI)*c, f32::sin(t0+PI)*c);
        let p3 = points[0] + Vector2::new(f32::cos(t1+PI)*c, f32::sin(t1+PI)*c);

        let mut render_object = RenderObject::new(LINE_RENDER_TYPE.with(|f| f.clone()));
        render_object.add_triangle([0,1,2]);
        render_object.add_triangle([2,3,0]);

        render_object.set_v_datas(0, "pos", vec![VertexData::FloatVec2(p0),VertexData::FloatVec2(p1),VertexData::FloatVec2(p2),VertexData::FloatVec2(p3)]);
        render_object.set_v_datas(0, "vColor", vec![VertexData::FloatVec4(color.clone()),VertexData::FloatVec4(color.clone()),VertexData::FloatVec4(color.clone()),VertexData::FloatVec4(color.clone())]);
        render_object.set_v_datas(0, "vThickness", vec![VertexData::Float(thickness),VertexData::Float(thickness),VertexData::Float(thickness),VertexData::Float(thickness)]);
        render_object.set_v_datas(0, "vSmooth", vec![VertexData::Float(smooth),VertexData::Float(smooth),VertexData::Float(smooth),VertexData::Float(smooth)]);
        render_object.set_v_datas(0, "points1", vec![VertexData::FloatVec2(points[0].clone()),VertexData::FloatVec2(points[0].clone()),VertexData::FloatVec2(points[0].clone()),VertexData::FloatVec2(points[0].clone())]);
        render_object.set_v_datas(0, "points2", vec![VertexData::FloatVec2(points[1].clone()),VertexData::FloatVec2(points[1].clone()),VertexData::FloatVec2(points[1].clone()),VertexData::FloatVec2(points[1].clone())]);

        render_object.update(renderer);
        Self { obj:render_object, end_behavior:end_behavior, smooth:smooth, thickness:thickness }
    }

    pub fn update_points(&mut self, renderer: &mut Renderer, p1:Vector2<f32>, p2:Vector2<f32>) {

        Self::set_bounding_box(&mut self.obj, &self.end_behavior, [p1,p2], self.thickness, self.smooth);
        
        self.obj.set_v_datas(0, "points1", vec![VertexData::FloatVec2(p1.clone()),VertexData::FloatVec2(p1.clone()),VertexData::FloatVec2(p1.clone()),VertexData::FloatVec2(p1.clone())]);
        self.obj.set_v_datas(0, "points2", vec![VertexData::FloatVec2(p2.clone()),VertexData::FloatVec2(p2.clone()),VertexData::FloatVec2(p2.clone()),VertexData::FloatVec2(p2.clone())]);
        
        self.obj.update(renderer);
    }

    fn set_bounding_box(render_obj:&mut RenderObject, end_behavior:&EndBehavior, points:[Vector2<f32>; 2], thickness:f32, smooth:f32) {
        let theta = (points[0].y-points[1].y).atan2(points[0].x-points[1].x);

        let t0 = match end_behavior {
                EndBehavior::Rounded => theta+FRAC_PI_2+FRAC_PI_4,
                EndBehavior::Clipped => theta+FRAC_PI_2
            };
        let t1 = match end_behavior {
            EndBehavior::Rounded => theta+FRAC_PI_4,
            EndBehavior::Clipped => theta+FRAC_PI_2
        };
        let offset = thickness + smooth;
    
        let c = match end_behavior {
            EndBehavior::Clipped => offset,
            EndBehavior::Rounded => offset*SQRT_2
        };

        let p0 = points[0] + Vector2::new(f32::cos(t0)*c, f32::sin(t0)*c);
        let p1 = points[1] + Vector2::new(f32::cos(t1)*c, f32::sin(t1)*c);
        let p2 = points[1] + Vector2::new(f32::cos(t0+PI)*c, f32::sin(t0+PI)*c);
        let p3 = points[0] + Vector2::new(f32::cos(t1+PI)*c, f32::sin(t1+PI)*c);

        render_obj.set_v_datas(0, "pos", vec![VertexData::FloatVec2(p0),VertexData::FloatVec2(p1),VertexData::FloatVec2(p2),VertexData::FloatVec2(p3)]);
    }

    pub fn render(&mut self) {

    }
}