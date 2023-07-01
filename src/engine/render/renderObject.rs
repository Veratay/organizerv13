use std::rc::Rc;

use js_sys::{Uint16Array, ArrayBuffer, Float32Array};
use nalgebra::{Transform3, Matrix4};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject, WebGlProgram, WebGlTexture};

use crate::{log_str, log_f32_arr, log_u16_arr};

use super::{program::{create_program_from_src}};

#[derive(Clone)]
pub struct RenderObject {
    pub type_id:Rc<RenderType>,
    //pub bounding_cuboid:Cuboid,
    pub verticies:Vec<f32>,
    pub indicies:Vec<u16>
}

//TODO: make shader registry
pub struct RenderType {
    pub name:String,
    pub vertex_shader:String,
    pub fragment_shader:String,
    //pub uniforms:Vec<Uniform>,
    pub vertex_attribs:Vec<VertexAttrib>,
    pub instanced:Option<RenderObject>,
    pub instance_attribs:Vec<VertexAttrib>,
    pub blank_vertex:Vec<f32>,
    pub vertex_size:usize,
    pub verticies_chunk_min_size:usize,
    pub verticies_chunk_grow_factor:f32,
    pub verticies_chunk_max_size:usize,
    pub indicies_chunk_min_size:usize,
    pub indicies_chunk_grow_factor:f32, 
    pub indicies_chunk_max_size:usize,
}

struct Unifrom {
    name:String,
    uniform:UniformType
}

enum UniformType {
    Texture(WebGlTexture),
    FLOAT(f32),
    MATRIX(Matrix4<f32>)
}

impl PartialEq<RenderType> for RenderType {
    fn eq(&self, other: &RenderType) -> bool {
        self.name == other.name &&
        self.vertex_shader == other.vertex_shader &&
        self.fragment_shader == other.fragment_shader
    }
}

impl RenderType {
    pub fn setup_program(&self, gl:&WebGl2RenderingContext) -> WebGlProgram {
        create_program_from_src(gl, &self.vertex_shader, &self.fragment_shader)
    }

    pub(super) fn setup_arrs(&self, gl:&WebGl2RenderingContext, verticies:&Vec<f32>, indicies:&Vec<u16>, program:&WebGlProgram) -> GlBuffers {
        let vao = gl.create_vertex_array().expect_throw("Error creating VAO");
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().expect_throw("Error Creating VBO");
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        unsafe {
            let buffer_view = match &self.instanced {
                None => js_sys::Float32Array::view(&verticies),
                Some(instanced_data) => js_sys::Float32Array::view(&instanced_data.verticies)
            };
            gl.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, &buffer_view, WebGl2RenderingContext::DYNAMIC_DRAW);
        }

        let mut offset = 0;
        let stride = self.vertex_attribs.iter().fold(0, |acc, a:&VertexAttrib| acc + a.data_type.get_size() * a.count );
        for a in self.vertex_attribs.iter() {
            let location = gl.get_attrib_location(program, &a.name) as u32;
            gl.enable_vertex_attrib_array(location);
            gl.vertex_attrib_pointer_with_i32(location, a.count, a.data_type.get_webgl_representation(), false, stride, offset);
            offset += a.data_type.get_size() * a.count;
        }

        let ibo = gl.create_buffer().expect_throw("Error Creating IBO");
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ibo));
        unsafe {
            let buffer_view = match &self.instanced {
                None => js_sys::Uint16Array::view(&indicies),
                Some(instanced_data) => js_sys::Uint16Array::view(&instanced_data.indicies)
            };
            gl.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, &buffer_view, WebGl2RenderingContext::DYNAMIC_DRAW);
        }

        let instance_buffer_object = match self.instanced {
            None => None,
            Some(_) => {
                let instanced_buffer_object = gl.create_buffer().expect_throw("unable to create Instance Buffer Object");
                gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&instanced_buffer_object));

                unsafe {
                    let buffer_view = js_sys::Float32Array::view(&verticies);
                    gl.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, &buffer_view, WebGl2RenderingContext::DYNAMIC_DRAW);
                }

                let stride = self.instance_attribs.iter().fold(0, |acc, a:&VertexAttrib| acc + a.data_type.get_size() * a.count );
                let mut offset = 0;
                for a in self.instance_attribs.iter() {
                    let location = gl.get_attrib_location(program, &a.name) as u32;
                    gl.enable_vertex_attrib_array(location);
                    gl.vertex_attrib_pointer_with_i32(location, a.count, a.data_type.get_webgl_representation(), false, stride, offset);
                    gl.vertex_attrib_divisor(location, 1);

                    offset += a.data_type.get_size() * a.count;
                }
                Some(instanced_buffer_object)
            }
        };

        gl.bind_vertex_array(None);

        GlBuffers { 
            vao:vao,
            vbo: vbo, 
            ibo: ibo, 
            instance: instance_buffer_object
        }
    }

    pub fn get_blank_vertex(&self) -> Vec<f32> {
        self.blank_vertex.clone()
    }
}

pub(super) struct GlBuffers {
    vao:WebGlVertexArrayObject,
    vbo:WebGlBuffer,
    ibo:WebGlBuffer,
    instance:Option<WebGlBuffer>
}

impl GlBuffers {
    pub fn buffer_sub_data(&self, gl:&WebGl2RenderingContext, verticies:&[f32],v_start:usize, indicies:&[u16], i_start:usize) {
        match &self.instance {
            None => gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vbo)),
            Some(instance_buffer) => gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(instance_buffer))
        };

        unsafe {
            log_f32_arr(js_sys::Float32Array::view(verticies));
            let buffer_view = js_sys::Float32Array::view(verticies);
            gl.buffer_sub_data_with_i32_and_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, v_start as i32, &buffer_view)
        }

        if self.instance.is_none() {
            gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.ibo));
            unsafe {
                log_u16_arr(js_sys::Uint16Array::view(indicies));
                let buffer_view = js_sys::Uint16Array::view(indicies);
                gl.buffer_sub_data_with_i32_and_array_buffer_view(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, i_start as i32, &buffer_view);
            }
        }
    }

    // pub fn update_instanced_data(&self, gl:&WebGl2RenderingContext, instance_data:&[f32]) -> Result<(),()> {
    //     if !self.instance.is_none() { return Err(()); }
    //     gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.instance.unwrap()));
    //     i32 size = gl.get_buffer_parameter(WebGl2RenderingContext::ARRAY_BUFFER, WebGl2RenderingContext::BUFFER_SIZE);
    //     Ok(())
    // }

    pub fn draw(&self, gl:&WebGl2RenderingContext, render_type:Rc<RenderType>, count:i32) {
        gl.bind_vertex_array(Some(&self.vao));
        if self.instance.is_some() {
            let l = &render_type.instanced.as_ref().expect_throw("Expected render type data to contain instance data").indicies.len();
            gl.draw_elements_instanced_with_i32(WebGl2RenderingContext::TRIANGLES, *l as i32, WebGl2RenderingContext::UNSIGNED_SHORT, 0, count);
        } else {
            gl.draw_elements_with_i32(WebGl2RenderingContext::TRIANGLES, count, WebGl2RenderingContext::UNSIGNED_SHORT, 0);
        }
        gl.bind_vertex_array(None);
    }

    pub fn is_instanced(&self) -> bool {
        self.instance.is_some()
    }

    pub fn log_data(&self, gl:&WebGl2RenderingContext, v_count:u32, i_count:u32) {
        //TODO: Check if this causes memory leak
        unsafe {
            let v_dst = Float32Array::new(&ArrayBuffer::new(v_count * 4));
            let i_dst = Uint16Array::new(&ArrayBuffer::new(i_count * 2));

            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER,Some(&self.vbo));
            gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,Some(&self.ibo));
            gl.get_buffer_sub_data_with_i32_and_array_buffer_view_and_dst_offset_and_length(WebGl2RenderingContext::ARRAY_BUFFER,0, &v_dst, 0, v_count);
            gl.get_buffer_sub_data_with_i32_and_array_buffer_view_and_dst_offset_and_length(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,0, &i_dst, 0, i_count);

            log_str("verticies");
            log_f32_arr(v_dst);
            log_str("indicies");
            log_u16_arr(i_dst);
        }
    }
}

#[derive(Clone)]
pub struct Vertex {
    pub vertex_data:Vec<f32>,
    pub instance_data:Vec<f32>,
    pub index_data:Vec<u16>
}

#[derive(Clone)]
pub struct VertexAttrib {
    pub name:String,
    pub data_type:ShaderDataTypes,
    pub count:i32
}


#[derive(Clone)]
pub enum ShaderDataTypes {
    FLOAT,
    // FLOAT_VEC2,
    // FLOAT_VEC3,
    // FLOAT_VEC4,
    // INT,
    // INT_VEC2,
    // INT_VEC3,
    // INT_VEC4,
    // BOOL,
    // BOOL_VEC2,
    // BOOL_VEC3,
    // BOOL_VEC4,
    // FLOAT_MAT2,
    // FLOAT_MAT3,
    // FLOAT_MAT4,
    // SAMPLER_2D,
    // SAMPLER_CUBE,
    // FLOAT_MAT2x3,
    // FLOAT_MAT2x4,
    // FLOAT_MAT3x2,
    // FLOAT_MAT3x4,
    // FLOAT_MAT4x2,
    // FLOAT_MAT4x3,
    // UNSIGNED_INT_VEC2,
    // UNSIGNED_INT_VEC3,
    // UNSIGNED_INT_VEC4,
    // UNSIGNED_NORMALIZED,
    // SIGNED_NORMALIZED
}

impl ShaderDataTypes {
    pub fn get_webgl_representation(&self) -> u32 {
        match self {
            ShaderDataTypes::FLOAT => WebGl2RenderingContext::FLOAT
        }
    }

    pub fn get_size(&self) -> i32 {
        match self {
            ShaderDataTypes::FLOAT => 4
        }
    }
}