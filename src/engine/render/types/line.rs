use std::{rc::Rc,f32::consts::{FRAC_PI_4,FRAC_PI_2, TAU, SQRT_2,PI}};

use nalgebra::{Point2, Vector2};

use crate::{engine::render::{render_object::{RenderType, VertexAttrib, ShaderDataTypes, RenderObject, UniformAttrib, UniformRole, AttributeRole, InstancedData}, renderer::{Renderer, MappedRenderObject, UniformBlock, Uniform, MappedTexture}}, log_str, log_i32};

thread_local! {
    static LINE_RENDER_TYPE: Rc<RenderType> = Rc::new(RenderType {
        name:String::from("Line"),
        instanced:None,
        vertex_shader:String::from(
            "#version 300 es
            
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
        fragment_shader:String::from(
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
        vertex_attribs:vec![
            VertexAttrib { 
                name: String::from("pos"), 
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT_VEC2, 
            },
            VertexAttrib {
                name: String::from("vColor"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT_VEC4,
            },
            VertexAttrib {
                name:String::from("vThickness"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT,
            },
            VertexAttrib { 
                name: String::from("points1"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT_VEC2, 
            }, 
            VertexAttrib { 
                name: String::from("points2"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT_VEC2, 
            }, 
            VertexAttrib { 
                name: String::from("vSmooth"), 
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT, 
            },
        ],
        instance_attribs:Vec::new(),
        uniform_attribs:Vec::new(),
        blank_vertex:vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vertex_size:14,
        verticies_chunk_min_size:20,
        verticies_chunk_grow_factor:1.1,
        verticies_chunk_max_size:2000,
        indicies_chunk_min_size:1000,
        indicies_chunk_grow_factor:1.1, 
        indicies_chunk_max_size:2000,
    })
}

pub struct Line {
    obj:MappedRenderObject,
}

pub enum EndBehavior {
    Clipped,
    Rounded
}

impl Line {
    pub fn new(renderer:&mut Renderer, points:[Vector2<f32>; 2], color:[f32; 4], thickness:f32, smooth:f32, end_behavior:EndBehavior) -> Self {
        let offset = thickness + smooth;
        
        let theta = (points[1].y-points[0].y).atan2(points[1].x-points[0].x) + FRAC_PI_2;
        let m:Vector2<f32> = (points[1]+points[0])/2.0;

        let t0 = match end_behavior {
                EndBehavior::Rounded => theta+FRAC_PI_2+FRAC_PI_4,
                EndBehavior::Clipped => theta+FRAC_PI_2
            };
        let t1 = match end_behavior {
            EndBehavior::Rounded => theta-FRAC_PI_2-FRAC_PI_4,
            EndBehavior::Clipped => theta-FRAC_PI_2
        };
        let c: f32 = offset*SQRT_2*2;
        let (x0,y0) = (points[0].x+f32::cos(t0)*offset*SQRT_2,points[0].y+f32::sin(t0)*offset*SQRT_2);
        let (x1,y1) = (points[1].x+f32::cos(t1)*offset*SQRT_2,points[1].y+f32::sin(t1)*offset*SQRT_2);
        let (x2,y2) = (x0+f32::cos(theta+FRAC_PI_2)*c,y0+f32::sin(theta+FRAC_PI_2)*c);
        let (x3,y3) = (x1+f32::cos(theta+FRAC_PI_2)*c,y1+f32::sin(theta+FRAC_PI_2)*c);

        let verticies = vec![
            x0,y0, color[0],color[1],color[2],color[3], thickness, points[0].x,points[0].y,points[1].x,points[1].y, smooth,
            x1,y1, color[0],color[1],color[2],color[3], thickness, points[0].x,points[0].y,points[1].x,points[1].y, smooth,
            x2,y2, color[0],color[1],color[2],color[3], thickness, points[0].x,points[0].y,points[1].x,points[1].y, smooth,
            x3,y3, color[0],color[1],color[2],color[3], thickness, points[0].x,points[0].y,points[1].x,points[1].y, smooth,
        ];

        let indicies = vec![0,1,3, 3,2,0];

        let render_object = RenderObject {
            type_id:LINE_RENDER_TYPE.with(|f| f.clone()),
            uniforms:UniformBlock::default(),
            verticies:verticies,
            indicies:indicies
        };

        let obj = MappedRenderObject::new(renderer, render_object);

        Self { obj:obj }
    }

    pub fn render(&mut self) {

    }
}