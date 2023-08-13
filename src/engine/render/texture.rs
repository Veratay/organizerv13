use std::{collections::HashMap, cell::RefCell, rc::Rc, fmt::Debug};

use guillotiere::{AtlasAllocator, size2, Allocation};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{WebGlTexture, WebGl2RenderingContext, HtmlImageElement};

//TODO- make it so that the number of instances cannot grow larger than the max provided, by merging them.

#[derive(PartialEq, Eq, Debug, Clone, Copy)]
#[allow(unused)]
pub enum TextureFormat {
    RGB,
    RGBA,
    // LUMINANCE_ALPHA,
    // LUMINANCE,
    // ALPHA,
    // R8,
    // R16F,
    // R32F,
    // R8UI,
    // RG8,
    // RG16F,
    // RG32F,
    // RG8UI,
    // RGB8,
    // SRGB8,
    // RGB565,
    // R11F_G11F_B10F,
    // RGB9_E5,
    // RGB16F,
    // RGB32F,
    // RGB8UI,
    // RGBA8,
    // SRGB8_ALPHA8,
    // RGB5_A1,
    // RGB10_A2,
    // RGBA4,
    // RGBA16F,
    // RGBA32F,
    // RGBA32F,
    // RGBA8UI
} 

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum TextureFilter {
    Linear,
    Nearest
}

impl TextureFilter {
    fn to_wgl(&self) -> i32  {
        match self {
            Self::Linear => WebGl2RenderingContext::LINEAR as i32,
            Self::Nearest => WebGl2RenderingContext::NEAREST as i32
        }
    }
}

impl TextureFormat {

    fn get_internal_format(&self) -> i32 {
        match self {
            Self::RGB => WebGl2RenderingContext::RGB as i32,
            Self::RGBA => WebGl2RenderingContext::RGBA as i32
        }
    }

    fn get_format(&self) -> u32 {
        match self {
            Self::RGB => WebGl2RenderingContext::RGB,
            Self::RGBA => WebGl2RenderingContext::RGBA
        }
    }

    fn get_type(&self) -> u32 {
        match self {
            Self::RGB | Self::RGBA => WebGl2RenderingContext::UNSIGNED_BYTE
        }
    }   
}

pub trait BatchableTextureSource {
    fn tex_sub_image_2d(&self, gl:&WebGl2RenderingContext, x:i32, y:i32);
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn format(&self) -> TextureFormat;
    fn min_filter(&self) -> TextureFilter;
    fn mag_filter(&self) -> TextureFilter;
    fn unique_texture(&self) -> bool;
    fn valid(&self) -> bool {true}
}

#[derive(PartialEq, Eq)]
pub struct RawTextureSource<'a> {
    pub data:&'a [u8],
    pub format:TextureFormat,
    pub min_filter:TextureFilter,
    pub mag_filter:TextureFilter,
    pub width:i32,
    pub height:i32,
    pub unique:bool
}

impl<'a> BatchableTextureSource for RawTextureSource<'a> {
    fn height(&self) -> i32 { self.height }
    fn width(&self) -> i32 { self.width }
    fn format(&self) -> TextureFormat { self.format }
    fn min_filter(&self) -> TextureFilter { self.min_filter }
    fn mag_filter(&self) -> TextureFilter { self.mag_filter }
    fn unique_texture(&self) -> bool { self.unique }
    fn tex_sub_image_2d(&self, gl:&WebGl2RenderingContext, x:i32, y:i32) {
        unsafe {
            let buffer_view = js_sys::Uint8Array::view(&self.data);
            gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                WebGl2RenderingContext::TEXTURE_2D, 
                0, 
                x, 
                y, 
                self.width, 
                self.height, 
                self.format.get_format(), 
                self.format.get_type(), 
                Some(&buffer_view)
            ).expect_throw("Texture batcher upload failed.");
        }
    }
}

pub struct TempBlankTextureSource {
    unique:bool,
    width:i32,
    height:i32,
    format:TextureFormat,
    min_filter:TextureFilter,
    mag_filter:TextureFilter
}

impl TempBlankTextureSource {
    pub fn new(unique:bool, width:i32, height:i32, format:TextureFormat, min_filter:TextureFilter, mag_filter:TextureFilter) -> Self {
        Self { unique: unique, width: width, height: height, format: format, min_filter:min_filter, mag_filter:mag_filter }
    }
}

impl BatchableTextureSource for TempBlankTextureSource {
    fn format(&self) -> TextureFormat {
        self.format
    }
    fn min_filter(&self) -> TextureFilter { self.min_filter }
    fn mag_filter(&self) -> TextureFilter { self.mag_filter }
    fn height(&self) -> i32 {
        self.height
    }
    fn unique_texture(&self) -> bool {
        self.unique
    }
    fn width(&self) -> i32 {
        self.width
    }
    fn tex_sub_image_2d(&self, _:&WebGl2RenderingContext, _:i32, _:i32) {}
    fn valid(&self) -> bool {
        false
    }
}

pub struct ImageTextureSource {
    image:HtmlImageElement,
    unique:bool,
    min_filter:TextureFilter,
    mag_filter:TextureFilter
}

impl ImageTextureSource {
    pub fn new(image:HtmlImageElement, unique:bool, min_filter:TextureFilter, mag_filter:TextureFilter) -> Self {
        Self { image: image, unique: unique, min_filter:min_filter, mag_filter:mag_filter }
    }
}

impl BatchableTextureSource for ImageTextureSource {
    fn format(&self) -> TextureFormat { TextureFormat::RGBA }
    fn height(&self) -> i32 {
        self.image.height() as i32
    }
    fn width(&self) -> i32 {
        self.image.width() as i32
    }
    fn min_filter(&self) -> TextureFilter { self.min_filter }
    fn mag_filter(&self) -> TextureFilter { self.mag_filter }
    fn tex_sub_image_2d(&self, gl:&WebGl2RenderingContext, x:i32, y:i32) {
        gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_html_image_element(
            WebGl2RenderingContext::TEXTURE_2D, 
            0, 
            x, 
            y, 
            self.image.width() as i32, 
            self.image.height() as i32, 
            WebGl2RenderingContext::RGBA, 
            WebGl2RenderingContext::UNSIGNED_BYTE, 
            &self.image
        ).expect_throw("Error uploading HtmlImageElement to texture");
    }
    fn unique_texture(&self) -> bool {
        self.unique
    }
}

pub struct BatchedTexture {
    remove_cache:Rc<RefCell<RemoveCache>>,
    texture_id:u32,
    allocation:Allocation,
    loaded:bool,
    updating:bool
}

impl Debug for BatchedTexture {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BatchedTexture")
        .field("remove_cache", &"..")
        .field("texture_id", &self.texture_id)
        .field("allocation", &self.allocation)
        .field("loaded", &self.loaded)
        .field("updating", &self.updating)
        .finish()
    }
}

#[derive(Debug)]
struct FutureRemovedBatchedTexture {
    texture_id:u32,
    allocation:Allocation
}

impl PartialEq for BatchedTexture {
    fn eq(&self, other: &Self) -> bool {
        self.texture_id == other.texture_id && self.allocation == other.allocation && Rc::ptr_eq(&other.remove_cache, &self.remove_cache)
    }
}

impl Drop for BatchedTexture {
    fn drop(&mut self) {
        if self.updating { return; }
        self.remove_cache.borrow_mut().remove(FutureRemovedBatchedTexture { texture_id: self.texture_id, allocation: self.allocation });
    }
}

impl BatchedTexture {
    pub fn new(batcher:&mut TextureBatcher, src:&dyn BatchableTextureSource) -> Self {
        batcher.add(src)
    }

    pub fn update(&mut self, batcher:&mut TextureBatcher, src:&dyn BatchableTextureSource) {
        batcher.update_batched(self, src); 
    }

    pub fn get_texcoord(&self, batcher:&TextureBatcher, x:f32, y:f32) -> (f32,f32) {
        return batcher.get_texcoord(self, x, y);
    }

    pub fn same_instance(&self, other:&Self) -> bool {
        self.texture_id == other.texture_id
    }

    pub fn bind(&self, batcher:&TextureBatcher) {
        batcher.bind(self);
    }

    pub fn loaded(&self) -> bool {
        self.loaded
    }
}


pub struct UpdateCache {
    inner:Vec<(Rc<RefCell<BatchedTexture>>,Box<dyn BatchableTextureSource>)>
}

impl Debug for UpdateCache {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner:Vec<_> = self.inner.iter().map(|x| &x.0).collect();
        f.debug_struct("UpdateCache")
        .field("inner", &inner)
        .finish()
    }
}

impl UpdateCache {
    fn new() -> Self {
        Self { inner:Vec::new() }
    }

    pub fn cache_update(&mut self, texture:Rc<RefCell<BatchedTexture>>,src:Box<dyn BatchableTextureSource>) {
        self.inner.push((texture,src));
    }

    fn process(&mut self, batcher:&mut TextureBatcher) {
        for (texture,src) in self.inner.iter() {
            batcher.update_batched(&mut texture.borrow_mut(), &**src);
        }
        self.inner.clear();
    }
}

#[derive(Debug)]
pub struct RemoveCache {
    inner:Vec<FutureRemovedBatchedTexture>
}

impl RemoveCache {
    fn new() -> Self {
        Self { inner:Vec::new() }
    }

    fn remove(&mut self, texture:FutureRemovedBatchedTexture) {
        self.inner.push(texture);
    }

    fn process(&mut self, batcher:&mut TextureBatcher) {
        for texture in self.inner.iter() {
            batcher.remove(texture.texture_id,texture.allocation);
        }
        self.inner.clear();
    }
}

#[derive(Debug)]
pub struct TextureBatcher {
    instances:HashMap<u32,TextureBatcherInstance>,
    texture_remove_cache:Rc<RefCell<RemoveCache>>, //needs to be owned by every batched texture to allow adding to queue on drop
    update_cache:Rc<RefCell<UpdateCache>>, //for asynchronous update operations, such as image loading
    gl:WebGl2RenderingContext,
    last_instance_id:u32,
    min_width:i32,
    min_height:i32,
}

impl TextureBatcher {    
    pub fn new(gl:WebGl2RenderingContext, w:i32, h:i32 ) -> Self {
        Self { 
            instances: HashMap::new(), 
            texture_remove_cache:Rc::new(RefCell::new(RemoveCache::new())),
            update_cache:Rc::new(RefCell::new(UpdateCache::new())),
            gl:gl,
            last_instance_id: 0, 
            min_width: w,
            min_height: h,
        }
    }

    pub fn get_update_cache(&self) -> Rc<RefCell<UpdateCache>> {
        self.update_cache.clone()
    }

    pub fn get_remove_cache(&self) -> Rc<RefCell<RemoveCache>> {
        self.texture_remove_cache.clone()
    }

    pub fn cleanup(&mut self) {
        let cache = Rc::clone(&self.texture_remove_cache);
        cache.borrow_mut().process(self);
    }

    pub fn update(&mut self) {
        let cache = Rc::clone(&self.update_cache);
        cache.borrow_mut().process(self);
    }

    fn add(&mut self, src:&dyn BatchableTextureSource) -> BatchedTexture {
        self.cleanup();
        let gl = self.gl.clone();
        let unique = src.unique_texture();

        //try to add into existing instance
        if !unique {
            for (instance_id,instance) in self.instances.iter_mut() {
                if let Some(batched_texture) = instance.add(self.texture_remove_cache.clone(), &gl, src, *instance_id) {
                    return batched_texture;
                }
            }
        }

        //create new instance
        self.last_instance_id += 1;

        let mut new_instance = TextureBatcherInstance::new(
            &gl, 
            src.format(), 
            src.min_filter(),
            src.mag_filter(),
            if unique { src.width() } else { i32::max(self.min_width, src.width()) },
            if unique { src.height() } else { i32::max(self.min_height, src.height()) }
        );

        let result = new_instance.add(self.texture_remove_cache.clone(), &gl, src, self.last_instance_id).expect("Expected new texture batcher instance to succesfully allocate");

        self.instances.insert(self.last_instance_id, new_instance);
        
        result
    }

    fn bind(&self, batched_texture:&BatchedTexture) {
        self.instances.get(&batched_texture.texture_id).expect_throw("Expected texture ID to be valid while binding").bind(&self.gl.clone(),WebGl2RenderingContext::TEXTURE_2D)
    }

    fn get_texcoord(&self, batched_texture:&BatchedTexture, x:f32, y:f32) -> (f32,f32) {
        self.instances.get(&batched_texture.texture_id).expect_throw("Expected texture ID to be valid while adjusting texcoord")
        .adjust_texture_coord(&batched_texture.allocation, x, y)
    }

    fn update_batched(&mut self, batched_texture:&mut BatchedTexture, src:&dyn BatchableTextureSource) {
        batched_texture.updating = true;

        let id = batched_texture.texture_id;
        let gl = self.gl.clone();
        let instance = self.instances.get(&id).expect_throw("Expected texture ID to be valid while updating");
        
        if src.width() != batched_texture.allocation.rectangle.width() ||
            src.height() != batched_texture.allocation.rectangle.height() ||
            src.format() != instance.format {
                //called to save space for differently sized texture
                self.cleanup();
                //do it this way to remove old texture before adding new one, cannot assign by dereference because that would cause a drop,
                //which would try to remove the allocation twice, which might panic(untested)
                self.remove(batched_texture.texture_id, batched_texture.allocation);
                *batched_texture = self.add(src);
        } else {
            instance.update_batched(&gl, &batched_texture.allocation, src);
            batched_texture.loaded = src.valid();
        }

        batched_texture.updating = false;
    }

    fn remove(&mut self, id:u32, allocation:Allocation) {
        self.instances.get_mut(&id).expect_throw("Expected texture ID to be valid while removing").remove(allocation);
    }
}

struct TextureBatcherInstance {
    atlas:AtlasAllocator,
    texture:WebGlTexture,
    width:i32,
    height:i32,
    format:TextureFormat,
}

impl Debug for TextureBatcherInstance {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("TextureBatcherInstance")
        .field("atlas", &"..")
        .field("texture", &self.texture)
        .field("width", &self.width)
        .field("height", &self.height)
        .field("format", &self.format)
        .finish()
    }
}

impl TextureBatcherInstance { 
    fn new(gl:&WebGl2RenderingContext,format:TextureFormat, min_filter:TextureFilter, mag_filter:TextureFilter, width:i32,height:i32) -> Self {
        let texture = gl.create_texture().expect_throw("Render Error: Unable to create instance of texture batcher");
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&texture));
        gl.tex_image_2d_with_i32_and_i32_and_i32_and_format_and_type_and_opt_u8_array(
            WebGl2RenderingContext::TEXTURE_2D, 
            0, 
            format.get_internal_format(), 
            width, 
            height, 
            0, 
            format.get_format(),
            format.get_type(), 
            None
        ).expect_throw("Error uploading initial data to TextureBatcher GlTexture");
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MIN_FILTER, min_filter.to_wgl());
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_MAG_FILTER, mag_filter.to_wgl());
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_S, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);
        gl.tex_parameteri(WebGl2RenderingContext::TEXTURE_2D, WebGl2RenderingContext::TEXTURE_WRAP_T, WebGl2RenderingContext::CLAMP_TO_EDGE as i32);

        Self { 
            atlas: AtlasAllocator::new(size2(width, height)),
            texture: texture, 
            width: width,
            height: height,
            format: format,
        }
    }

    fn add(&mut self, remove_cache:Rc<RefCell<RemoveCache>>, gl:&WebGl2RenderingContext, src:&dyn BatchableTextureSource, instance_id:u32) -> Option<BatchedTexture> {
        let (height,width,format) = (src.height(),src.width(),src.format());
        if !(format == self.format && width <= self.width && height <= self.height) {return None;}

        if let Some(allocation) = self.atlas.allocate(size2(width, height)) {
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
            let p = allocation.rectangle.min;
            src.tex_sub_image_2d(gl, p.x, p.y);

            let result = BatchedTexture { 
                remove_cache: remove_cache,
                texture_id: instance_id,
                allocation: allocation,
                loaded:src.valid(),
                updating:false
            };

            return Some(result);
        } else {
            return None;
        }
    }

    fn update_batched(&self, gl:&WebGl2RenderingContext, allocation:&Allocation, src:&dyn BatchableTextureSource) {
        let p = allocation.rectangle.min;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        src.tex_sub_image_2d(gl, p.x, p.y);
    }

    fn bind(&self, gl:&WebGl2RenderingContext, target:u32) {
        gl.bind_texture(target, Some(&self.texture));
    }

    fn adjust_texture_coord(&self, allocation:&Allocation, x:f32, y:f32) -> (f32,f32)  {
        let min = allocation.rectangle.min;

        (
            (min.x as f32 + x*allocation.rectangle.width() as f32) / self.width as f32, 
            (min.y as f32 + y*allocation.rectangle.height() as f32) / self.height as f32
        )
    }

    fn remove(&mut self, allocation:Allocation) {
        self.atlas.deallocate(allocation.id);
    }

    fn empty(&self) -> bool {
        self.atlas.is_empty()
    }
}