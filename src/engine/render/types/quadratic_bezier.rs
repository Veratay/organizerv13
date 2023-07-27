use std::{rc::Rc,f32::consts::{FRAC_PI_4,FRAC_PI_2, TAU, SQRT_2,PI}};

use nalgebra::{Point2, Vector2};

use crate::{engine::render::{render_object::{RenderType, VertexAttrib, ShaderDataTypes, RenderObject, UniformAttrib, UniformRole, AttributeRole, InstancedData}, renderer::{Renderer, MappedRenderObject, UniformBlock, Uniform, MappedTexture}}, log_str, log_i32};

thread_local! {
    static QUADRATIC_BEZIER_RENDER_TYPE: Rc<RenderType> = Rc::new(RenderType {
        name:String::from("Quadratic_bezier"),
        instanced:None,
        vertex_shader:String::from(
            "#version 300 es
            
            in vec2 pos;
            in vec4 vColor;
            in float vThickness;
            in float vSmooth;
            in vec2 points1;
            in vec2 points2;
            in vec2 points3;

            out vec2 posf;
            out vec2 af;
            out vec2 controlf;
            out vec2 cf;
            out float thicknessf;
            out vec4 color;
            out float fsmooth;
    
            void main() {
                gl_Position = vec4(pos,0.0,1.0);
                posf = gl_Position.xy;
                af = points1;
                controlf = points2;
                cf =points3;
                thicknessf = vThickness;
                color = vColor;
                fsmooth = vSmooth;
            }"
        ),
        fragment_shader:String::from(
            "# version 300 es
            precision highp float;
            // start point
            in vec2 af;
            // control point
            in vec2 controlf;
            // end point
            in vec2 cf;
            // current position in fragment shader
            in vec2 posf;
            // thickness of the curve
            in float thicknessf;
            in float fsmooth;

            in vec4 color;
    
            out vec4 FragColor;
    
            float dot2( in vec2 v ) { return dot(v,v); }
    
            // borrowed from here https://www.iquilezles.org/www/articles/distfunctions2d/distfunctions2d.htm
            float sdBezier( in vec2 pos, in vec2 A, in vec2 B, in vec2 C )
            {
                vec2 a = B - A;
                vec2 b = A - 2.0*B + C;
                vec2 c = a * 2.0;
                vec2 d = A - pos;
                float kk = 1.0/dot(b,b);
                float kx = kk * dot(a,b);
                float ky = kk * (2.0*dot(a,a)+dot(d,b)) / 3.0;
                float kz = kk * dot(d,a);
                float res = 0.0;
                float p = ky - kx*kx;
                float p3 = p*p*p;
                float q = kx*(2.0*kx*kx-3.0*ky) + kz;
                float h = q*q + 4.0*p3;
                if( h >= 0.0)
                {
                    h = sqrt(h);
                    vec2 x = (vec2(h,-h)-q)/2.0;
                    vec2 uv = sign(x)*pow(abs(x), vec2(1.0/3.0));
                    float t = clamp( uv.x+uv.y-kx, 0.0, 1.0 );
                    res = dot2(d + (c + b*t)*t);
                }
                else
                {
                    float z = sqrt(-p);
                    float v = acos( q/(p*z*2.0) ) / 3.0;
                    float m = cos(v);
                    float n = sin(v)*1.732050808;
                    vec3  t = clamp(vec3(m+m,-n-m,n-m)*z-kx,0.0,1.0);
                    res = min(  dot2(d+(c+b*t.x)*t.x),
                                dot2(d+(c+b*t.y)*t.y) );
                    // the third root cannot be the closest
                    // res = min(res,dot2(d+(c+b*t.z)*t.z));
                }
                return sqrt( res );
            }
    
            void main() {
                vec4 result = color;
                float d = sdBezier(posf, af, controlf, cf) - thicknessf;
                // bigger value -- more blury
                float s = smoothstep(0., fsmooth, -d);
                if (d < 0.) {
                    result.a = s;
                } else {
                    result.a = 0.5;
                    // discard;
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
                name: String::from("points3"),
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

pub struct QuadraticBezier {
    obj:MappedRenderObject,
}

impl QuadraticBezier {
    pub fn new(renderer:&mut Renderer, points:[Vector2<f32>; 3], color:[f32; 4], thickness:f32, smooth:f32) -> Self {
        let offset = thickness + smooth;
        
        let theta = (points[2].y-points[0].y).atan2(points[2].x-points[0].x);
        let m:Vector2<f32> = (points[2]+points[0])/2.0;
        let ctheta = (points[1].y-m.y).atan2(points[1].x-m.x);
        let copposite = f32::signum(f32::sin(ctheta));
        let c:f32 = copposite*((points[1]-m).magnitude()/2.0+offset*2.0);
        log_str(&theta.to_string());
        let t0 = if copposite == -1.0 { theta+FRAC_PI_2+FRAC_PI_4} else { theta+PI+FRAC_PI_4};
        let t1 = if copposite == -1.0 { theta+FRAC_PI_4 } else { theta+TAU-FRAC_PI_4};
        let (x0,y0) = (points[0].x+f32::cos(t0)*offset*SQRT_2,points[0].y+f32::sin(t0)*offset*SQRT_2);
        let (x1,y1) = (points[2].x+f32::cos(t1)*offset*SQRT_2,points[2].y+f32::sin(t1)*offset*SQRT_2);
        let (x2,y2) = (x0+f32::cos(theta+FRAC_PI_2)*c,y0+f32::sin(theta+FRAC_PI_2)*c);
        let (x3,y3) = (x1+f32::cos(theta+FRAC_PI_2)*c,y1+f32::sin(theta+FRAC_PI_2)*c);

        let verticies = vec![
            x0,y0, color[0],color[1],color[2],color[3], thickness, points[0].x,points[0].y,points[1].x,points[1].y,points[2].x,points[2].y, smooth,
            x1,y1, color[0],color[1],color[2],color[3], thickness, points[0].x,points[0].y,points[1].x,points[1].y,points[2].x,points[2].y, smooth,
            x2,y2, color[0],color[1],color[2],color[3], thickness, points[0].x,points[0].y,points[1].x,points[1].y,points[2].x,points[2].y, smooth,
            x3,y3, color[0],color[1],color[2],color[3], thickness, points[0].x,points[0].y,points[1].x,points[1].y,points[2].x,points[2].y, smooth,
        ];

        let indicies = vec![0,1,3, 3,2,0];

        let render_object = RenderObject {
            type_id:QUADRATIC_BEZIER_RENDER_TYPE.with(|f| f.clone()),
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