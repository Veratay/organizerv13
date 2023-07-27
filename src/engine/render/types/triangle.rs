use std::{rc::Rc,f32::consts::{FRAC_PI_4,FRAC_PI_2, TAU, SQRT_2,PI}};

use nalgebra::{Point2, Vector2};

use crate::{engine::render::{render_object::{RenderType, VertexAttrib, ShaderDataTypes, RenderObject, UniformAttrib, UniformRole, AttributeRole, InstancedData}, renderer::{Renderer, MappedRenderObject, UniformBlock, Uniform, MappedTexture}}, log_str, log_i32};

thread_local! {
    static TRIANGLE_RENDER_TYPE: Rc<RenderType> = Rc::new(RenderType {
        name:String::from("Triangle"),
        instanced:None,
        vertex_shader:String::from(
            "#version 300 es
            
            in vec2 pos;
            in vec4 vColor;

            out vec4 color;
    
            void main() {
                gl_Position = vec4(pos,0.0,1.0);
                color = vColor;
            }"
        ),
        fragment_shader:String::from(
            "# version 300 es
            precision highp float;
            in vec4 color;
            out vec4 FragColor;
            void main() {
                FragColor = color;
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
            
        ],
        instance_attribs:Vec::new(),
        uniform_attribs:Vec::new(),
        blank_vertex:vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vertex_size:6,
        verticies_chunk_min_size:20,
        verticies_chunk_grow_factor:1.1,
        verticies_chunk_max_size:2000,
        indicies_chunk_min_size:1000,
        indicies_chunk_grow_factor:1.1, 
        indicies_chunk_max_size:2000,
    })
}

pub struct Triangle {
    obj:MappedRenderObject,
}

impl Triangle {
    pub fn new(renderer:&mut Renderer, points:[Vector2<f32>; 3], color:[f32; 4]) -> Self {

        let verticies = vec![
            points[0].x,points[0].y, color[0],color[1],color[2],color[3],
            points[1].x,points[1].y, color[0],color[1],color[2],color[3],
            points[2].x,points[2].y, color[0],color[1],color[2],color[3]
        ];

        let indicies = vec![0,1,2];

        let render_object = RenderObject {
            type_id:TRIANGLE_RENDER_TYPE.with(|f| f.clone()),
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