use std::{rc::Rc,f32::consts::{FRAC_PI_4,FRAC_PI_2, TAU, SQRT_2,PI}};

use cgmath::{Vector2, InnerSpace, Vector4};

use crate::engine::render::{render_object::{RenderType, VertexAttrib, ShaderDataTypes, RenderObject, AttributeRole}, renderer::{Renderer, RenderObjectAllocation, UniformBlock, VertexData}};

thread_local! {
    static QUADRATIC_BEZIER_RENDER_TYPE: Rc<RenderType> = Rc::new(RenderType::new_batched_growable(
        String::from(
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
        ),String::from(
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
                    discard;
                }
                FragColor = result;
            }"
        ), 
        vec![
            VertexAttrib { 
                name: String::from("pos"), 
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FloatVec2, 
            },
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
                name: String::from("points3"),
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
        2000,
        1000,
        2000, 
        2.0,
        2.0
    ));
}

pub struct QuadraticBezier {
    obj:RenderObject,
}

impl QuadraticBezier {
    pub fn new(renderer:&mut Renderer, points:[Vector2<f32>; 3], color:Vector4<f32>, thickness:f32, smooth:f32) -> Self {
        let offset = thickness + smooth;
        
        let theta = (points[2].y-points[0].y).atan2(points[2].x-points[0].x);
        let m:Vector2<f32> = (points[2]+points[0])/2.0;
        let ctheta = (points[1].y-m.y).atan2(points[1].x-m.x);
        let copposite = f32::signum(f32::sin(ctheta));
        let c:f32 = copposite*((points[1]-m).magnitude()/2.0+offset*2.0);
        let t0 = if copposite == -1.0 { theta+FRAC_PI_2+FRAC_PI_4} else { theta+PI+FRAC_PI_4};
        let t1 = if copposite == -1.0 { theta+FRAC_PI_4 } else { theta+TAU-FRAC_PI_4};
        let p0 = Vector2 {
            x: points[0].x+f32::cos(t0)*offset*SQRT_2,
            y: points[0].y+f32::sin(t0)*offset*SQRT_2
        };
        let p1 = Vector2 { 
            x: points[2].x+f32::cos(t1)*offset*SQRT_2,
            y: points[2].y+f32::sin(t1)*offset*SQRT_2
        };
        let p2 = p0 + Vector2 {x:f32::cos(theta+FRAC_PI_2),y:f32::sin(theta+FRAC_PI_2)}*c;
        let p3 = p1 + Vector2 {x:f32::cos(theta+FRAC_PI_2),y:f32::sin(theta+FRAC_PI_2)}*c;

        let mut render_object = RenderObject::new(QUADRATIC_BEZIER_RENDER_TYPE.with(|f| f.clone()));

        render_object.add_triangle([0,1,3]);
        render_object.add_triangle([3,2,0]);
        
        render_object.set_v_datas(0, "pos", vec![VertexData::FloatVec2(p0),VertexData::FloatVec2(p1),VertexData::FloatVec2(p2),VertexData::FloatVec2(p3)]);
        render_object.set_v_datas(0,"points1", vec![VertexData::FloatVec2(points[0]),VertexData::FloatVec2(points[0]),VertexData::FloatVec2(points[0]),VertexData::FloatVec2(points[0])]);
        render_object.set_v_datas(0,"points2", vec![VertexData::FloatVec2(points[1]),VertexData::FloatVec2(points[1]),VertexData::FloatVec2(points[1]),VertexData::FloatVec2(points[1])]);
        render_object.set_v_datas(0,"points3", vec![VertexData::FloatVec2(points[2]),VertexData::FloatVec2(points[2]),VertexData::FloatVec2(points[2]),VertexData::FloatVec2(points[2])]);
        render_object.set_v_datas(0, "vColor", vec![VertexData::FloatVec4(color.clone()),VertexData::FloatVec4(color.clone()),VertexData::FloatVec4(color.clone()),VertexData::FloatVec4(color.clone())]);
        render_object.set_v_datas(0, "vThickness", vec![VertexData::Float(thickness),VertexData::Float(thickness),VertexData::Float(thickness),VertexData::Float(thickness)]);
        render_object.set_v_datas(0, "vSmooth", vec![VertexData::Float(smooth),VertexData::Float(smooth),VertexData::Float(smooth),VertexData::Float(smooth)]);

        renderer.update(&mut render_object);

        Self { obj:render_object }
    }

    pub fn render(&mut self) {

    }
}