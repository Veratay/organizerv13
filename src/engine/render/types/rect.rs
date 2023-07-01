use std::{ rc::Rc, cell::RefCell};

use nalgebra::{Transform3, Transform2, Point2};

use crate::engine::render::{renderObject::{RenderType, VertexAttrib, ShaderDataTypes, RenderObject}, renderer::{Renderer, MappedRenderObject}};

thread_local! {
    static RECT_RENDER_TYPE: Rc<RenderType> = Rc::new(RenderType {
        name:String::from("Rect"),
        instanced:None,
        vertex_shader:String::from(
            "#version 300 es
    
            in vec2 position;
            in vec4 color;

            out vec4 vColor;
    
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                vColor = color;
            }"
        ),
        fragment_shader:String::from(
            "#version 300 es

            precision mediump float;

            in vec4 vColor;  // Input color from vertex shader

            out vec4 fragColor;

            void main() {
                fragColor = vec4(vColor);  // Use input color for fragment color
            }"
        ),
        vertex_attribs:vec![
            VertexAttrib { name: String::from("position"), data_type:ShaderDataTypes::FLOAT, count:2}, 
            VertexAttrib { name: String::from("color"), data_type:ShaderDataTypes::FLOAT, count:4}
        ],
        instance_attribs:Vec::new(),
        blank_vertex:vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vertex_size:6,
        verticies_chunk_min_size:1000,
        verticies_chunk_grow_factor:1.1,
        verticies_chunk_max_size:2000,
        indicies_chunk_min_size:1000,
        indicies_chunk_grow_factor:1.1, 
        indicies_chunk_max_size:2000,
    })
}

pub struct Rect {
    pos:Transform2<f32>,
    color:[f32; 4],
    renderer:Rc<RefCell<Renderer>>,
    mapped_id:MappedRenderObject
}

impl Rect {
    pub fn new(pos:Transform2<f32>, color:[f32; 4], renderer:Rc<RefCell<Renderer>>) -> Self {
        let v0:Point2<f32> = pos * Point2::<f32>::new(1.0, 1.0);
        let v1:Point2<f32> = pos * Point2::<f32>::new(-1.0, 1.0);
        let v2:Point2<f32> = pos * Point2::<f32>::new(-1.0, -1.0);
        let v3:Point2<f32> = pos * Point2::<f32>::new(1.0, -1.0);

        let verticies = vec![
            v0.x,v0.y, color[0],color[1],color[2],color[3],
            v1.x,v1.y, color[0],color[1],color[2],color[3],
            v2.x,v2.y, color[0],color[1],color[2],color[3],
            v3.x,v3.y, color[0],color[1],color[2],color[3],
        ];

        let indicies:Vec<u16> = vec![0,1,2, 0,2,3];

        let render_object = RenderObject {
            type_id:RECT_RENDER_TYPE.with(|f| f.clone()),
            //pos: Transform3::identity(),
            verticies:verticies,
            indicies:indicies
        };

        let mapped = renderer.borrow_mut().map(render_object);

        Self { pos: pos, color: color, renderer: renderer.clone(), mapped_id: mapped }
    }
}