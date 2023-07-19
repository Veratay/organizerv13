use std::{collections::HashMap, rc::{Rc, Weak}, mem, cell::RefCell};

use wasm_bindgen::{JsCast, UnwrapThrowExt, prelude::Closure};
use web_sys::{WebGl2RenderingContext, WebGlProgram, HtmlCanvasElement, WebGlUniformLocation, HtmlImageElement, Event};

use crate::{log_str, log_i32};

use super::{render_object::{GlBuffers, RenderType, RenderObject}, texture::{TextureBatcher, BatchedTexture, BatchableTextureSource, ImageTextureSource, TempBlankTextureSource, UpdateCache}, index_map::IndexMap};

//TODO- maybe find a better way to approach fragmentation when large number of textures are being used in one rendertype, 
//because if textures are inserted whereever there is space and the texture batcher instances are getting full, 
//then basically a new renderchunk will have to be created every time a new object is allocated.

const BATCH_TEXTURE_SIZE:i32 = 8192;

pub struct Renderer {
    gl:WebGl2RenderingContext,
    canvas:HtmlCanvasElement,
    render_batchers:HashMap<String,RenderBatcher>,
    texture_batcher:TextureBatcher,
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

        //config
        gl.pixel_storei(WebGl2RenderingContext::UNPACK_FLIP_Y_WEBGL, 1);

        Self { 
            gl: gl.clone(),
            canvas: canvas, 
            render_batchers: HashMap::new(),
            texture_batcher:TextureBatcher::new(gl, BATCH_TEXTURE_SIZE, BATCH_TEXTURE_SIZE),
            loaded_images:HashMap::new()
        }
    }

    fn add(&mut self, object:RenderObject) -> MappedRenderObject  {
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

    fn update(&mut self, mapped:&mut MappedRenderObject, object:RenderObject) {
        self.render_batchers.get_mut(&object.type_id.name.clone().to_owned()).expect_throw("Expected batcher to exist while updating render object").update(&self.gl, mapped, object)
    }

    pub fn upload_image_from_url(&mut self, url:String) -> MappedTexture {
        self.texture_batcher.cleanup();
        if let Some(rc) = self.loaded_images.get(&url).and_then(|weak| weak.upgrade()) {
            return MappedTexture { batched_texture: rc };
        }

        let batched_texture = Rc::new(RefCell::new(BatchedTexture::new(&mut self.texture_batcher, &TempBlankTextureSource::new(false, 1, 1, super::texture::TextureFormat::RGBA))));
        
        let mapped_texture = MappedTexture { batched_texture:batched_texture };
        let img = HtmlImageElement::new().expect_throw("Error creating HtmlImageELement while uploading image from url");
        
        let update_queue = self.texture_batcher.get_update_cache();
        let url_clone = url.clone();
        let img_clone = img.clone();
        let mapped_clone = mapped_texture.clone();

        let onerr_callback = Closure::wrap(Box::new(move |_| {
            log_str(&("Warning: Image ".to_owned() + &url_clone + " failed to load"))
        }) as Box<dyn FnMut(Event)>);
        
        let onload_callback = Closure::wrap(Box::new(move |_| {
            log_str("Image loading");
            mapped_clone.cached_update(&mut update_queue.borrow_mut(),Box::new( ImageTextureSource::new(img_clone.clone(), false)));
        }) as Box<dyn FnMut(Event)>);

        img.set_onerror(Some(onerr_callback.as_ref().unchecked_ref()));
        onerr_callback.forget();
        img.set_onload(Some(onload_callback.as_ref().unchecked_ref()));
        onload_callback.forget();

        img.set_cross_origin(Some(""));
        img.set_src(&url);

        self.loaded_images.insert(url, Rc::downgrade(&mapped_texture.batched_texture));
        mapped_texture
    }

    pub fn upload_texture(&mut self, src:&dyn BatchableTextureSource) -> MappedTexture {
        self.texture_batcher.cleanup();
        MappedTexture {
            batched_texture:Rc::new(RefCell::new(BatchedTexture::new(&mut self.texture_batcher, src))),
        }
    }

    pub fn render(&mut self) {
        self.texture_batcher.update();

        self.resize_canvas();
        self.gl.viewport(0, 0, self.canvas.width() as i32, self.canvas.height() as i32);

        for data in self.render_batchers.values() {
            data.render(self);
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
    chunks:IndexMap<RenderChunk>,
    mapped:IndexMap<RenderChunkIndex>,
    gl:WebGl2RenderingContext,
    program:WebGlProgram,
}

impl RenderBatcher {

    fn id_mapped_internal(&mut self, render_type:Rc<RenderType>, mapped:RenderChunkIndex) -> MappedRenderObject {
        MappedRenderObject { render_type: render_type, id: self.mapped.push(mapped)}
    }
    fn map_render_object_into_new(gl:&WebGl2RenderingContext, object:RenderObject) -> (RenderBatcher,MappedRenderObject) {
        let mut result = Self {
            gl:gl.clone(),
            chunks:IndexMap::new(),
            program:object.type_id.setup_program(gl),
            mapped:IndexMap::new(),
        };

        let type_id = object.type_id.clone();

        let (chunk, mut chunk_index) = RenderChunk::map_render_object_into_new(gl, object, &result.program);

        let chunk_id = result.chunks.push(chunk);
        chunk_index.chunk = chunk_id;

        let mapped = result.id_mapped_internal(type_id, chunk_index);

        (result,mapped)
    }

    fn map_render_object(&mut self, gl:&WebGl2RenderingContext, object:RenderObject) -> MappedRenderObject {
        let type_id = object.type_id.clone();

        for (i, chunk) in self.chunks.iter_mut() {
            if let Some(mut mapped) = chunk.map_render_object(gl, &object) {
                mapped.chunk = i.clone();
                return self.id_mapped_internal(type_id, mapped);
            }
        };

        let (chunk, mut mapped) = RenderChunk::map_render_object_into_new(gl, object, &self.program);
        mapped.chunk = self.chunks.push(chunk);

        return self.id_mapped_internal(type_id, mapped);
    }

    fn update(&mut self, gl:&WebGl2RenderingContext, mapped:&mut MappedRenderObject, object:RenderObject) {
        let chunk_index = &self.mapped[mapped.id];
        if self.chunks[chunk_index.chunk].update(gl, &object, &chunk_index).is_ok() { return; }

        self.remove(&mapped);
        *mapped = self.map_render_object(gl, object);
    }

    fn remove(&mut self, object:&MappedRenderObject) {
        log_str("aaaaa");
        log_i32(object.id as i32);
        // let chunk_index = &self.mapped[object.id];
        //self.chunks[chunk_index.chunk].remove(&self.gl, chunk_index, &object.render_type)
        self.mapped.remove(object.id);
        if true { log_str(&format!("{:?}",self.mapped)); }
    }

    fn render(&self, renderer:&Renderer) {
        let gl = &self.gl;
        gl.use_program(Some(&self.program));
        log_str(&format!("{:?}",self.mapped));
        for chunk in self.chunks.values() {
            chunk.render(renderer,&self.program);
        }
    }
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
        let verticies_chunk_size = usize::max(verticies_len,object.type_id.verticies_chunk_min_size*object.type_id.vertex_size);
        verticies.resize(verticies_chunk_size, 0.0);

        let mut indicies = object.indicies;
        let indicies_len = indicies.len();
        let indicied_chunk_size = usize::max(indicies_len,object.type_id.indicies_chunk_min_size*object.type_id.vertex_size);
        indicies.resize(indicied_chunk_size, 0);

        let gl_buffers = object.type_id.setup_arrs(gl, &verticies, &indicies, program);

        return (
            RenderChunk {
                render_type: object.type_id.clone(),
                gl_buffers: gl_buffers,
                verticies,
                verticies_free_areas:vec![ SlicePointer { 
                    start:verticies_len, 
                    size: verticies_chunk_size-verticies_len 
                }],
                uniforms:object.uniforms.clone(),
                indicies: indicies,
                indicies_free_areas: vec![ SlicePointer {
                    start:indicies_len,
                    size:indicied_chunk_size-indicies_len
                }],
                indicies_count:indicies_len,
                verticies_count:verticies_len/object.type_id.vertex_size//
            },

            RenderChunkIndex {
                chunk:0,
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

            let result = RenderChunkIndex { 
                chunk:0,
                v_slice: SlicePointer { 
                    start: v_slice.start, 
                    size: verticies_len 
                }, 
                i_slice: SlicePointer { 
                    start: i_slice.start, 
                    size: indicies_len 
                }
            };

            v_slice.start += verticies_len;
            v_slice.size -= verticies_len;
            i_slice.start += indicies_len;
            i_slice.size -= indicies_len;
            
            self.verticies_count += verticies_len/object.type_id.vertex_size;
            self.indicies_count += indicies_len;

            let log_v_len = (v_slice.start+v_slice.size) as u32;
            let log_i_len = (i_slice.start+i_slice.size) as u32;

            self.upload_at_slice(gl, object, &result);

            //TODO: COMPLETELY FLAWED WHEN THINGS START TO GET DELETED
            log_str("data after updating");
            self.gl_buffers.log_data(gl,log_v_len, log_i_len );

            return Some(result);
        }
        return None;
    }

    fn remove(&mut self, gl:&WebGl2RenderingContext, mapped:&RenderChunkIndex, type_id:&RenderType) {

        //this very heavily relies on the free areas array being in order and valid, if there is any corruption in it this will just make everything worse.
        let mut before = None;
        let mut i_to_remove:Option<usize> = None;
        let mut isolated = true;
        let mut last_under = 0;
        for (i,x) in self.verticies_free_areas.iter_mut().enumerate() {
            if x.start+x.size == mapped.v_slice.start {
                x.size += mapped.v_slice.size;
                before = Some(x);
                isolated = false;
            } else if x.start == mapped.v_slice.start + mapped.v_slice.size {
                isolated = false;
                match &mut before  {
                    None => { x.size += mapped.v_slice.size; },
                    Some(before) => { 
                        before.size += x.size;
                        i_to_remove = Some(i);
                    }
                }
            } else if isolated && x.start < mapped.v_slice.start {
                last_under = i;
            }
        }

        if let Some(i) = i_to_remove { self.verticies_free_areas.remove(i); }

        if isolated {
            self.verticies_free_areas.insert(last_under, mapped.v_slice.clone());
        }

        //indicies
        let mut before = None;
        let mut i_to_remove:Option<usize> = None;
        let mut isolated = true;
        let mut last_under = 0;
        for (i,x) in self.indicies_free_areas.iter_mut().enumerate() {
            if x.start+x.size == mapped.i_slice.start {
                x.size += mapped.i_slice.size;
                before = Some(x);
                isolated = false;
            } else if x.start == mapped.i_slice.start + mapped.i_slice.size {
                isolated = false;
                match &mut before  {
                    None => { x.size += mapped.i_slice.size; },
                    Some(before) => { 
                        before.size += x.size;
                        i_to_remove = Some(i);
                    }
                }
            } else if isolated && x.start < mapped.i_slice.start {
                last_under = i;
            }
        }

        if let Some(i) = i_to_remove { self.indicies_free_areas.remove(i); }

        if isolated {
            self.indicies_free_areas.insert(last_under, mapped.i_slice.clone());
        }

        let blank_vertex = &type_id.blank_vertex;

        let mut vec = Vec::new();
        let mut i = 0;
        let l = blank_vertex.len()-1;
        vec.resize_with(mapped.v_slice.size, || { i+=1; blank_vertex[(i-1) % l] });

        let mut indicies = Vec::new();
        indicies.resize(mapped.i_slice.size, 0u16);
        self.gl_buffers.buffer_sub_data(gl, &vec, mapped.v_slice.start, &indicies, mapped.i_slice.start);
    }

    fn update(&mut self, gl:&WebGl2RenderingContext, object:&RenderObject, mapped_chunk_index:&RenderChunkIndex) -> Result<(),()> {
        if !self.uniforms.batchable_with(&object.uniforms) { return Err(()); } 
        let verticies_len = object.verticies.len();
        let indicies_len = object.indicies.len();
        if verticies_len == mapped_chunk_index.v_slice.size && indicies_len == mapped_chunk_index.i_slice.size {
            self.upload_at_slice(gl, object, mapped_chunk_index);

            return Ok(());
        }
        Err(())
    }

    fn upload_at_slice(&mut self, gl:&WebGl2RenderingContext, object:&RenderObject, mapped_chunk_index:&RenderChunkIndex) {
        let verticies_len = object.verticies.len();
        let indicies_len = object.indicies.len();
        let v_slice = &mapped_chunk_index.v_slice;
        let i_slice = &mapped_chunk_index.i_slice;
        let verticies = &object.verticies;
        let indicies = &object.indicies;
        if verticies_len == mapped_chunk_index.v_slice.size && indicies_len == mapped_chunk_index.i_slice.size {
            unsafe {
                if verticies_len > 0 { 
                    std::ptr::copy_nonoverlapping::<f32>(verticies.as_ptr(), self.verticies.as_mut_ptr().offset( v_slice.start as isize), verticies_len); 
                }
                if indicies_len > 0 {
                    // self.indicies.splice(i_slice.start..i_slice.start+indicies_len, indicies.iter().map(|x| x + (v_slice.start/object.type_id.vertex_size) as u16));
                    std::ptr::copy_nonoverlapping(indicies.as_ptr(), self.indicies.as_mut_ptr().offset( i_slice.start as isize), indicies_len); 
                    for x in self.indicies[i_slice.start..i_slice.start+indicies_len].iter_mut() {
                        *x += (v_slice.start/object.type_id.vertex_size) as u16;
                    }
                }
            }

            self.gl_buffers.buffer_sub_data(gl, verticies, v_slice.start * mem::size_of::<f32>(), &self.indicies[i_slice.start..i_slice.start+indicies_len], i_slice.start * mem::size_of::<u16>());
        }
    }

    fn render(&self, renderer:&Renderer, program:&WebGlProgram) {
        self.uniforms.setup_uniforms_and_textures(renderer, program);
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

        self.gl_buffers.draw(&renderer.gl, self.render_type.clone(), count as i32);
    }
}

pub struct MappedRenderObject { 
    render_type:Rc<RenderType>, 
    id:usize,
}

impl MappedRenderObject {
    pub fn new(renderer:&mut Renderer, render_object:RenderObject) -> Self {
        renderer.add(render_object)
    }

    pub fn update(&mut self, renderer:&mut Renderer, render_object:RenderObject) {
        renderer.update(self, render_object);
    }
}

#[derive(Clone, PartialEq)]
pub struct MappedTexture {
    batched_texture:Rc<RefCell<BatchedTexture>>,
}

impl MappedTexture {
    pub fn update(&mut self, renderer:&mut Renderer, src:&dyn BatchableTextureSource) {
        self.batched_texture.borrow_mut().update(&mut renderer.texture_batcher, src);
    }

    pub fn get_texcoord(&self, renderer:&Renderer, x:f32, y:f32) -> (f32,f32) {
        self.batched_texture.borrow().get_texcoord(&renderer.texture_batcher, x, y)
    }

    pub fn valid(&self) -> bool {
        self.batched_texture.borrow().valid()
    }

    fn cached_update(&self, update_cache:&mut UpdateCache,  src:Box<dyn BatchableTextureSource>) {
        update_cache.cache_update(self.batched_texture.clone(), src);
    }

    fn bind(&self, renderer:&Renderer) {
        self.batched_texture.borrow().bind(&renderer.texture_batcher);
    }
}

#[derive(Clone, Debug)]
struct RenderChunkIndex {
    chunk:usize,
    v_slice:SlicePointer,
    i_slice:SlicePointer,
}

#[derive(Clone, Debug)]
struct SlicePointer {
    start:usize,
    size:usize
}

#[derive(PartialEq, Clone)]
#[allow(unused)]
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

    fn setup_uniforms_and_textures(&self, renderer:&Renderer, program:&WebGlProgram) {
        let gl = &renderer.gl;
        let mut texture_count = 0;
        if self.cached_uniform_locations.borrow().is_empty() { self.get_uniform_locations(gl, program); }
        for (uniform,location) in self.uniforms.iter().zip(self.cached_uniform_locations.borrow().iter()) {
            match &uniform.data {
                UnifromType::Float(x) => { gl.uniform1f(Some(&location), *x); },
                UnifromType::Texture(mapped) => {
                    let active = WebGl2RenderingContext::TEXTURE0 + texture_count;
                    gl.active_texture(active);
                    mapped.bind(renderer);
                    gl.uniform1i(Some(&location), texture_count as i32);
                    texture_count += 1;
                }
            }
        }
    }
}