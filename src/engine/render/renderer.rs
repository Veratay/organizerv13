use std::{collections::HashMap, rc::Rc, mem};

use wasm_bindgen::{JsCast, JsValue};
use web_sys::{WebGl2RenderingContext, WebGlProgram, HtmlCanvasElement, WebGlTexture};

use crate::log_str;

use super::renderObject::{GlBuffers, RenderType, RenderObject};

pub struct Renderer {
    gl:WebGl2RenderingContext,
    canvas:HtmlCanvasElement,
    data:HashMap<String,RenderBatcher>,
}

pub struct RenderBatcher {
    chunks:Vec<RenderChunk>,
    mapped:HashMap<u32,RenderChunkIndex>,
    last_mapped_id:u32,
    program:WebGlProgram
}

pub struct RenderChunk {
    render_type:Rc<RenderType>,
    gl_buffers:GlBuffers,
    verticies:Vec<f32>,
    verticies_free_areas:Vec<SlicePointer>,
    indicies:Vec<u16>,
    indicies_free_areas:Vec<SlicePointer>,
    indicies_count:usize,
    verticies_count:usize
}

pub struct MappedRenderObject { 
    render_type:Rc<RenderType>, 
    id:u32 
}

pub struct RenderChunkIndex {
    v_slice:SlicePointer,
    i_slice:SlicePointer
}

struct SlicePointer {
    start:usize,
    size:usize
}

enum Unifrom {
    Texture(WebGlTexture),
    Float
}

impl Renderer {
    pub fn new(canvas:HtmlCanvasElement) -> Self {
        let gl = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>().unwrap();
        Self { 
            gl: gl,
            canvas: canvas, 
            data: HashMap::new(),
        }
    }

    pub fn map(&mut self, object:RenderObject) -> MappedRenderObject  {
        if let Some(data) = self.data.get_mut(&object.type_id.name.clone().to_owned()) {
            let mapped = data.map_render_object(&self.gl, object);
            return mapped;
        } else {
            let k = object.type_id.name.clone().to_owned();
            let (data, mapped) = RenderBatcher::map_render_object_into_new(&self.gl, object);
            self.data.insert(k, data);
            return mapped;
        }
    }

    pub fn render(&self) {

        self.resize_canvas();
        self.gl.viewport(0, 0, self.canvas.width() as i32, self.canvas.height() as i32);

        for data in self.data.values() {
            data.render(&self.gl)
        }
    }

    fn resize_canvas(&self) {
        let display_width = self.canvas.client_width();
        let display_height = self.canvas.client_height();
        self.canvas.set_width(display_width as u32);
        self.canvas.set_height(display_height as u32);
    }
}

impl RenderBatcher {

    fn id_mapped_internal(&mut self, render_type:Rc<RenderType>, mapped:RenderChunkIndex) -> MappedRenderObject {
        batcher_id_mapped_internal_base(&mut self.mapped, &mut self.last_mapped_id, render_type, mapped)
    }
    fn map_render_object_into_new(gl:&WebGl2RenderingContext, object:RenderObject) -> (RenderBatcher,MappedRenderObject) {
        let mut result = Self {
            chunks:Vec::new(),
            program:object.type_id.setup_program(gl),
            mapped:HashMap::new(),
            last_mapped_id:0
        };

        let type_id = object.type_id.clone();

        let (chunk, mapped) = RenderChunk::map_render_object_into_new(gl, object, &result.program);

        result.chunks.push(chunk);

        let mapped = result.id_mapped_internal(type_id, mapped);

        (result,mapped)
    }

    fn map_render_object(&mut self, gl:&WebGl2RenderingContext, object:RenderObject) -> MappedRenderObject {
        let type_id = object.type_id.clone();

        match batcher_map_render_object_base(gl, &mut self.chunks, &object) {
            Some(mapped_internal) => {
                return self.id_mapped_internal(type_id, mapped_internal);
            },
            None => {
                let (chunk, mapped) = RenderChunk::map_render_object_into_new(gl, object, &self.program);
                self.chunks.push(chunk);

                return self.id_mapped_internal(type_id, mapped);
            }
        }
    }

    fn render(&self, gl:&WebGl2RenderingContext) {
        gl.use_program(Some(&self.program));
        log_str("rendering chunk");
        for chunk in self.chunks.iter() {
            chunk.render(gl);
        }
    }
}

pub fn batcher_id_mapped_internal_base(hash_map:&mut HashMap<u32,RenderChunkIndex>, last_mapped_id:&mut u32, render_type:Rc<RenderType>, mapped_internal:RenderChunkIndex) -> MappedRenderObject {
    let id = last_mapped_id.clone()+1;
    hash_map.insert(id, mapped_internal);
    *last_mapped_id = id;
    MappedRenderObject { render_type: render_type, id: id }
}

pub fn batcher_map_render_object_base(gl:&WebGl2RenderingContext, chunks:&mut Vec<RenderChunk>, object:&RenderObject) -> Option<RenderChunkIndex> {
    for (i, chunk) in chunks.iter_mut().enumerate() {
        if let Some(mut mapped) = chunk.map_render_object(gl, &object) {
            mapped.v_slice.start += i*object.type_id.verticies_chunk_min_size;
            mapped.i_slice.start += i*object.type_id.indicies_chunk_min_size;
            return Some(mapped);
        }
    };
    return None;
}

impl RenderChunk {
    fn map_render_object_into_new(gl:&WebGl2RenderingContext, object:RenderObject, program:&WebGlProgram) -> (Self,RenderChunkIndex) {
        let mut verticies = object.verticies;
        let verticies_len = verticies.len();
        verticies.resize(object.type_id.verticies_chunk_min_size, 0.0);

        let mut indicies = object.indicies;
        let indicies_len = indicies.len();
        indicies.resize(object.type_id.indicies_chunk_min_size, 0);

        let gl_buffers = object.type_id.setup_arrs(gl, &verticies, &indicies, program);

        return (
            RenderChunk {
                render_type: object.type_id.clone(),
                gl_buffers: gl_buffers,
                verticies,
                verticies_free_areas:vec![ SlicePointer { 
                    start:verticies_len, 
                    size: object.type_id.verticies_chunk_min_size-verticies_len 
                }],
                indicies: indicies,
                indicies_free_areas: vec![ SlicePointer {
                    start:indicies_len,
                    size:object.type_id.indicies_chunk_min_size-indicies_len
                }],
                indicies_count:indicies_len,
                verticies_count:verticies_len/object.type_id.vertex_size
            },

            RenderChunkIndex {
                v_slice:SlicePointer { start: 0, size: verticies_len },
                i_slice:SlicePointer { start: 0, size: indicies_len }
            }
        );
    }

    fn map_render_object(&mut self, gl:&WebGl2RenderingContext, object:&RenderObject) -> Option<RenderChunkIndex> {
        let verticies = &object.verticies;
        let verticies_len = verticies.len();

        let indicies = &object.indicies;
        let indicies_len = indicies.len();

        if let (Some(v_slice),Some(i_slice)) = 
            (
                self.verticies_free_areas.iter_mut().find(|s| s.size >= verticies_len),
                self.indicies_free_areas.iter_mut().find(|s| s.size >= indicies_len)
            )
        {
            log_str("indicies start");
            log_str(&i_slice.start.to_string());

            log_str("verticies start");
            log_str(&v_slice.start.to_string());

            log_str("verticies count");
            log_str(&self.verticies_count.to_string());

            unsafe {
                if verticies_len > 0 { 
                    std::ptr::copy_nonoverlapping::<f32>(verticies.as_ptr(), self.verticies.as_mut_ptr().offset( v_slice.start as isize), verticies_len); 
                }
                if indicies_len > 0 {
                    // self.indicies.splice(i_slice.start..i_slice.start+indicies_len, indicies.iter().map(|x| x + (v_slice.start/object.type_id.vertex_size) as u16));
                    std::ptr::copy_nonoverlapping(indicies.as_ptr(), self.indicies.as_mut_ptr().offset( i_slice.start as isize), indicies_len); 
                    for x in self.indicies[i_slice.start..i_slice.start+indicies_len].iter_mut() {
                        log_str("iter running");
                        *x += self.verticies_count as u16;
                    }
                }
            }

            self.gl_buffers.buffer_sub_data(gl, verticies, v_slice.start * mem::size_of::<f32>(), &self.indicies[i_slice.start..i_slice.start+indicies_len], i_slice.start * mem::size_of::<u16>());

            //TODO: COMPLETELY FLAWED WHEN THINGS START TO GET DELETED
            log_str("data after updating");
            self.gl_buffers.log_data(gl, (v_slice.start+v_slice.size) as u32, (i_slice.start+i_slice.size) as u32);

            let result = Some(RenderChunkIndex { 
                v_slice: SlicePointer { 
                    start: v_slice.start, 
                    size: verticies_len 
                }, 
                i_slice: SlicePointer { 
                    start: i_slice.start, 
                    size: indicies_len 
                }
            });

            v_slice.start += verticies_len;
            v_slice.size -= verticies_len;
            i_slice.start += indicies_len;
            i_slice.size -= indicies_len;
            
            self.verticies_count += verticies_len/object.type_id.vertex_size;
            self.indicies_count += indicies_len;

            return result;
        }
        return None;
    }

    fn render(&self, gl:&WebGl2RenderingContext) {
        let count = match self.gl_buffers.is_instanced() {
            true => {
                let mut count = 0;
                let mut end_used = false;
                for f in self.verticies_free_areas.iter() {
                    count = usize::max(count, f.start);
                    end_used = end_used ||(f.start+f.size == self.verticies.len());
                }
                match end_used {
                    true => count,
                    false => self.verticies.len()
                }
            },
        false => {
                let mut count = 0;
                let mut end_used = false;
                for f in self.indicies_free_areas.iter() {
                    count = usize::max(count, f.start);
                    end_used = end_used ||(f.start+f.size == self.indicies.len());
                }
                match end_used {
                    true => count,
                    false => self.indicies.len()
                }
            }
        };

        self.gl_buffers.draw(gl, self.render_type.clone(), count as i32);
    }
}
