use std::{collections::HashMap, rc::{Rc, Weak}, mem, cell::RefCell, thread, time::Duration};

use js_sys::Promise;
use wasm_bindgen::{JsCast, JsValue, UnwrapThrowExt, prelude::Closure};
use web_sys::{WebGl2RenderingContext, WebGlProgram, HtmlCanvasElement, WebGlTexture, WebGlUniformLocation, HtmlImageElement, Event};

use crate::log_str;

use super::{renderObject::{GlBuffers, RenderType, RenderObject, TextureLayoutNumber}, texture::{TextureBatcher, BatchedTexture, TextureBatcherInstance, BatchableTextureSource, ImageTextureSource, TempBlankTextureSource}};

//TODO- maybe find a better way to approach fragmentation when large number of textures are being used in one rendertype, 
//because if textures are inserted whereever there is space and the texture batcher instances are getting full, 
//then basically a new renderchunk will have to be created every time a new object is allocated.

const BATCH_TEXTURE_SIZE:i32 = 4096;

pub struct Renderer {
    gl:WebGl2RenderingContext,
    canvas:HtmlCanvasElement,
    render_batchers:HashMap<String,RenderBatcher>,
    texture_batcher:Rc<RefCell<TextureBatcher>>,
    loaded_images:HashMap<String, Weak<RefCell<BatchedTexture>>>
}

impl PartialEq for Renderer {
    fn eq(&self, other: &Self) -> bool {
        self.canvas == other.canvas && self.gl == other.gl
    }
}

impl Renderer {
    pub fn new(canvas:HtmlCanvasElement) -> Self {
        let gl = canvas.get_context("webgl2").unwrap().unwrap().dyn_into::<WebGl2RenderingContext>().unwrap();
        Self { 
            gl: gl.clone(),
            canvas: canvas, 
            render_batchers: HashMap::new(),
            texture_batcher:Rc::new(RefCell::new(TextureBatcher::new(gl, BATCH_TEXTURE_SIZE, BATCH_TEXTURE_SIZE))),
            loaded_images:HashMap::new()
        }
    }

    pub fn add(&mut self, object:RenderObject) -> MappedRenderObject  {
        if let Some(data) = self.render_batchers.get_mut(&object.type_id.name.clone().to_owned()) {
            let mapped = data.map_render_object(&self.gl, object);
            return mapped;
        } else {
            let k = object.type_id.name.clone().to_owned();
            let (data, mapped) = RenderBatcher::map_render_object_into_new(&self.gl, object);
            self.render_batchers.insert(k, data);
            return mapped;
        }
    }

    pub fn upload_image_from_url(&mut self, url:String) -> Result<MappedTexture,()> {
        if let Some(rc) = self.loaded_images.get(&url).and_then(|weak| weak.upgrade()) {
            return Ok(MappedTexture { batched_texture: rc, valid:true });
        }

        let batched_texture = Rc::new(RefCell::new(BatchedTexture::new(self.texture_batcher.clone(), &TempBlankTextureSource::new(true, 1, 1, super::texture::TextureFormat::RGBA))));
        
        let mapped_texture = MappedTexture { batched_texture:batched_texture, valid: false };
        let mut mapped_clone = mapped_texture.clone();
        let url_clone = url.clone();
        let img = HtmlImageElement::new().expect_throw("Error creating HtmlImageELement while uploading image from url");
        let img_clone = img.clone();
        img.set_cross_origin(Some(""));
        let onerr_callback = Closure::wrap(Box::new(move |_| {
            log_str(&("Warning: Image ".to_owned() + &url_clone + " failed to load"))
        }) as Box<dyn FnMut(Event)>);
        let onload_callback = Closure::wrap(Box::new(move |_| {
            mapped_clone.update( &ImageTextureSource::new(img_clone.clone(), true));
            mapped_clone.valid = true;
        }) as Box<dyn FnMut(Event)>);
        img.set_onerror(Some(onerr_callback.as_ref().unchecked_ref()));
        onerr_callback.forget();
        img.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
        onload_callback.forget();
        img.set_src(&url);

        self.loaded_images.insert(url, Rc::downgrade(&mapped_texture.batched_texture));
        Ok(mapped_texture)
    }

    pub fn upload_texture(&mut self, src:&dyn BatchableTextureSource) -> MappedTexture {
        MappedTexture {
            batched_texture:Rc::new(RefCell::new(BatchedTexture::new(self.texture_batcher.clone(), src))),
            valid:true
        }
    }

    pub fn render(&self) {

        self.resize_canvas();
        self.gl.viewport(0, 0, self.canvas.width() as i32, self.canvas.height() as i32);

        for data in self.render_batchers.values() {
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

pub struct RenderBatcher {
    chunks:Vec<RenderChunk>,
    mapped:HashMap<u32,RenderChunkIndex>,
    last_mapped_id:u32,
    program:WebGlProgram,
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
        for chunk in self.chunks.iter() {
            chunk.render(gl,&self.program);
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

pub struct RenderChunk {
    render_type:Rc<RenderType>,
    gl_buffers:GlBuffers,
    uniforms:UniformBlock,
    verticies:Vec<f32>,
    verticies_free_areas:Vec<SlicePointer>,
    indicies:Vec<u16>,
    indicies_free_areas:Vec<SlicePointer>,
    indicies_count:usize,
    verticies_count:usize
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
                uniforms:object.uniforms.clone(),
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
        if !self.uniforms.batchable_with(&object.uniforms) { return None; }
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

    fn render(&self, gl:&WebGl2RenderingContext, program:&WebGlProgram) {
        self.uniforms.setup_uniforms_and_textures(gl, program);
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

pub struct MappedRenderObject { 
    render_type:Rc<RenderType>, 
    id:u32 
}

#[derive(Clone, PartialEq)]
pub struct MappedTexture {
    batched_texture:Rc<RefCell<BatchedTexture>>,
    valid:bool
}

impl MappedTexture {
    pub fn update(&self, src:&dyn BatchableTextureSource) {
        if self.valid { self.batched_texture.borrow_mut().update(src); }
    }

    pub fn get_texcoord(&self, x:f32, y:f32) -> (f32,f32) {
        self.batched_texture.borrow().get_texcoord(x, y)
    }

    pub fn valid(&self) -> bool {
        self.valid
    }

    fn bind(&self) {
        self.batched_texture.borrow().bind();
    }
}

#[derive(PartialEq)]
pub struct RenderChunkIndex {
    v_slice:SlicePointer,
    i_slice:SlicePointer
}

#[derive(PartialEq)]
struct SlicePointer {
    start:usize,
    size:usize
}

#[derive(PartialEq, Clone)]
pub enum UnifromType {
    Texture(MappedTexture),
    Float(f32),
}

#[derive(PartialEq, Clone)]
pub struct Uniform {
    name:String,
    data:UnifromType,
}

impl Uniform {
    pub fn new(name:&str, data:UnifromType) -> Self {
        Self { name: String::from(name), data: data }
    }
    fn batchable_with(&self, other:&Self) -> bool {
        self.name == other.name && match (&self.data,&other.data) {
            (UnifromType::Texture(tex1),UnifromType::Texture(tex2)) => tex1.batched_texture.borrow().same_instance(&tex2.batched_texture.borrow()),
            (x,y) => x==y
        }
    }
}

#[derive(Clone)]
pub struct UniformBlock {
    uniforms:Vec<Uniform>,
    cached_uniform_locations:RefCell<Vec<WebGlUniformLocation>>
}

impl UniformBlock {
    pub fn new(uniforms:Vec<Uniform>) -> Self {
        Self { uniforms: uniforms, cached_uniform_locations: RefCell::new(Vec::new()) }
    }

    fn batchable_with(&self, other:&Self) -> bool {
        self.uniforms.iter().zip(other.uniforms.iter()).find(|(a,b)| !a.batchable_with(b)).is_none()
    }

    fn get_uniform_locations(&self,gl:&WebGl2RenderingContext, program:&WebGlProgram) {
        *self.cached_uniform_locations.borrow_mut() = Vec::new();
        for uniform in self.uniforms.iter() {
            self.cached_uniform_locations.borrow_mut().push(gl.get_uniform_location(program, &uniform.name).expect_throw("Get unifrom location failed"));
        }
    }

    fn setup_uniforms_and_textures(&self, gl:&WebGl2RenderingContext, program:&WebGlProgram) {
        let mut texture_count = 0;
        if self.cached_uniform_locations.borrow().is_empty() { self.get_uniform_locations(gl, program); }
        for (uniform,location) in self.uniforms.iter().zip(self.cached_uniform_locations.borrow().iter()) {
            match &uniform.data {
                UnifromType::Float(x) => { gl.uniform1f(Some(&location), *x); },
                UnifromType::Texture(mapped) => {
                    let active = WebGl2RenderingContext::TEXTURE0 + texture_count;
                    gl.active_texture(active);
                    mapped.bind();
                    gl.uniform1i(Some(&location), texture_count as i32);
                    gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
                    texture_count += 1;
                }
            }
        }
    }
}