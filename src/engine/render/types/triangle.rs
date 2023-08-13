use std::rc::Rc;

use cgmath::{Vector2, Vector4};

use crate::engine::render::{render_object::{RenderType, VertexAttrib, ShaderDataTypes, RenderObject, AttributeRole}, renderer::{Renderer, RenderObjectAllocation, VertexData}};

thread_local! {
    static TRIANGLE_RENDER_TYPE:Rc<RenderType> = Rc::new(RenderType::new_batched_growable(
        String::from(
            "#version 300 es
            
            in vec2 pos;
            in vec4 vColor;

            out vec4 color;
    
            void main() {
                gl_Position = vec4(pos,0.0,1.0);
                color = vColor;
            }"
        ), String::from(
            "# version 300 es
            precision highp float;
            in vec4 color;
            out vec4 FragColor;
            void main() {
                FragColor = color;
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
            
        ], 
        Vec::new(), 
        Vec::new(), 
        20, 
        40, 
        20, 
        40, 
        2.0, 
        2.0
    ));
}

pub struct Triangle {
    obj:RenderObject,
}

impl Triangle {
    pub fn new(renderer:&mut Renderer, points:[Vector2<f32>; 3], color:Vector4<f32>) -> Self {

        let mut render_object = RenderObject::new(TRIANGLE_RENDER_TYPE.with(|f| f.clone()));
        render_object.add_triangle([0,1,2]);
        render_object.set_v_datas(0, "pos", vec![
            VertexData::FloatVec2(points[0]),
            VertexData::FloatVec2(points[1]),
            VertexData::FloatVec2(points[2])
        ]);
        render_object.set_v_datas(0, "vColor", vec![
            VertexData::FloatVec4(color),
            VertexData::FloatVec4(color),
            VertexData::FloatVec4(color)
        ]);

        render_object.update(renderer);
        Self { obj:render_object }
    }

    pub fn render(&mut self) {

    }
}