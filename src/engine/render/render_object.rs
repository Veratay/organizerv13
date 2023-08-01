use std::{rc::Rc, mem};

use js_sys::{Uint16Array, ArrayBuffer, Float32Array};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlVertexArrayObject, WebGlProgram};

use crate::{log_str, log_f32_arr, log_u16_arr};

use super::{program::create_program_from_src, renderer::UniformBlock};

#[derive(Clone)]
pub struct RenderObject {
    pub type_id:Rc<RenderType>,
    pub uniforms:UniformBlock,
    pub verticies:Vec<f32>,
    pub indicies:Vec<u16>
}

pub struct InstancedData {
    pub verticies:Vec<f32>,
    pub indicies:Vec<u16>
}

//TODO: make shader registry
pub struct RenderType {
    pub name:String,
    pub vertex_shader:String,
    pub fragment_shader:String,
    pub instanced:Option<InstancedData>,
    pub blank_vertex:Vec<f32>,
    pub vertex_attribs:Vec<VertexAttrib>,
    pub instance_attribs:Vec<VertexAttrib>,
    pub uniform_attribs:Vec<UniformAttrib>,
    pub vertex_size:usize,
    pub verticies_chunk_min_size:usize,
    pub verticies_chunk_grow_factor:f32,
    pub verticies_chunk_max_size:usize,
    pub indicies_chunk_min_size:usize,
    pub indicies_chunk_grow_factor:f32, 
    pub indicies_chunk_max_size:usize,
}

impl PartialEq<RenderType> for RenderType {
    fn eq(&self, other: &RenderType) -> bool {
        self.name == other.name
    }
}

impl RenderType {
    pub fn setup_program(&self, gl:&WebGl2RenderingContext) -> WebGlProgram {
        create_program_from_src(gl, &self.vertex_shader, &self.fragment_shader)
    }

    pub(super) fn setup_arrs(&self, gl:&WebGl2RenderingContext, verticies:&Vec<f32>, indicies:&Vec<u16>, program:&WebGlProgram, verticies_size:usize, indicies_size:usize) -> GlBuffers {
        let vao = gl.create_vertex_array().expect_throw("Error creating VAO");
        gl.bind_vertex_array(Some(&vao));

        let vbo = gl.create_buffer().expect_throw("Error Creating VBO");
        gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&vbo));
        unsafe {
            let buffer_view = match &self.instanced {
                None => js_sys::Float32Array::view(&verticies),
                Some(instanced_data) => js_sys::Float32Array::view(&instanced_data.verticies)
            };
            if verticies_size == verticies.len() {
                gl.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, &buffer_view, WebGl2RenderingContext::DYNAMIC_DRAW);
            } else {
                gl.buffer_data_with_i32(WebGl2RenderingContext::ARRAY_BUFFER, (verticies_size*mem::size_of::<f32>()) as i32, WebGl2RenderingContext::DYNAMIC_DRAW);
                gl.buffer_sub_data_with_i32_and_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, 0, &buffer_view);
            }
        }

        let mut offset = 0;
        let stride = self.vertex_attribs.iter().fold(0, |acc, a:&VertexAttrib| acc + a.data_type.get_size() );
        for a in self.vertex_attribs.iter() {
            let location = gl.get_attrib_location(program, &a.name);
            if location == -1 { log_str(&format!("Could not find attribute location of {} attribute",a.name))}
            let location = location as u32;
            gl.enable_vertex_attrib_array(location);
            gl.vertex_attrib_pointer_with_i32(location, a.data_type.get_count(), a.data_type.get_webgl_representation(), false, stride, offset);
            offset += a.data_type.get_size();
        }

        let ibo = gl.create_buffer().expect_throw("Error Creating IBO");
        gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&ibo));
        unsafe {
            let buffer_view = match &self.instanced {
                None => js_sys::Uint16Array::view(&indicies),
                Some(instanced_data) => js_sys::Uint16Array::view(&instanced_data.indicies)
            };
            if indicies_size == indicies.len() {
                gl.buffer_data_with_array_buffer_view(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, &buffer_view, WebGl2RenderingContext::DYNAMIC_DRAW);
            } else {
                gl.buffer_data_with_i32(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, (indicies_size*mem::size_of::<u16>()) as i32, WebGl2RenderingContext::DYNAMIC_DRAW);
                gl.buffer_sub_data_with_i32_and_array_buffer_view(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, 0, &buffer_view);
            }
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

                let stride = self.instance_attribs.iter().fold(0, |acc, a:&VertexAttrib| acc + a.data_type.get_size() );
                let mut offset = 0;
                for a in self.instance_attribs.iter() {
                    let location = gl.get_attrib_location(program, &a.name) as u32;
                    gl.enable_vertex_attrib_array(location);
                    gl.vertex_attrib_pointer_with_i32(location, a.data_type.get_count(), a.data_type.get_webgl_representation(), false, stride, offset);
                    gl.vertex_attrib_divisor(location, 1);

                    offset += a.data_type.get_size();
                }
                Some(instanced_buffer_object)
            }
        };

        gl.bind_vertex_array(None);

        GlBuffers { 
            gl:gl.clone(),
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

#[derive(PartialEq)]
pub(super) struct GlBuffers {
    gl:WebGl2RenderingContext,
    vao:WebGlVertexArrayObject,
    vbo:WebGlBuffer,
    ibo:WebGlBuffer,
    instance:Option<WebGlBuffer>
}

impl Drop for GlBuffers {
    fn drop(&mut self) {
        self.gl.delete_buffer(Some(&self.vbo));
        self.gl.delete_buffer(Some(&self.ibo));
        if let Some(buffer) = &self.instance {
            self.gl.delete_buffer(Some(&buffer))
        }
        self.gl.delete_vertex_array(Some(&self.vao));
    }
}

impl GlBuffers {
    pub fn buffer_sub_data(&self, gl:&WebGl2RenderingContext, verticies:&[f32],v_start:usize, indicies:&[u16], i_start:usize) {
        match &self.instance {
            None => gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&self.vbo)),
            Some(instance_buffer) => gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(instance_buffer))
        };

        unsafe {
            let buffer_view = js_sys::Float32Array::view(verticies);
            gl.buffer_sub_data_with_i32_and_array_buffer_view(WebGl2RenderingContext::ARRAY_BUFFER, v_start as i32, &buffer_view)
        }

        if self.instance.is_none() {
            gl.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, Some(&self.ibo));
            unsafe {
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

    #[allow(unused)]
    pub fn log_data(&self, gl:&WebGl2RenderingContext, v_count:u32, i_count:u32) {
        //TODO: Check if this causes memory leak
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

#[derive(Clone, Copy)]
#[allow(unused)]
pub enum AttributeRole {
    Custom,
    TextureCoordinate
}

#[derive(Clone, Copy)]
#[allow(unused)]
pub enum UniformRole {
    Custom,
    Texture,
    Projection,
    View
}

#[derive(Clone)]
pub struct VertexAttrib {
    pub name:String,
    pub role:AttributeRole,
    pub data_type:ShaderDataTypes,
}

pub struct UniformAttrib {
    pub name:String,
    pub role:UniformRole
}


#[derive(Clone)]
#[allow(unused)]
pub enum ShaderDataTypes {
    FLOAT,
    FloatVec2,
    // FLOAT_VEC3,
    FloatVec4,
    INT,
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
            Self::FLOAT => WebGl2RenderingContext::FLOAT,
            Self::INT => WebGl2RenderingContext::INT,
            Self::FloatVec2 => WebGl2RenderingContext::FLOAT,
            Self::FloatVec4 => WebGl2RenderingContext::FLOAT
        }
    }

    pub fn get_size(&self) -> i32 {
        match self {
            Self::FLOAT => 4,
            Self::INT => 4,
            Self::FloatVec2 => 8,
            Self::FloatVec4 => 16
        }
    }

    pub fn get_count(&self) -> i32 {
        match self {
            Self::FLOAT | Self::INT => 1,
            Self::FloatVec2 => 2,
            Self::FloatVec4 => 4
        }
    }
}