use std::{rc::{ Rc}, collections::HashMap, cell::RefCell};

use guillotiere::{AtlasAllocator, size2, Allocation};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{WebGlTexture, WebGl2RenderingContext, HtmlImageElement};

//TODO- make it so that the number of instances cannot grow larger than the max provided, by merging them.

#[derive(PartialEq, Eq, Clone, Copy)]
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

    fn get_size(&self) -> usize {
        match self {
            Self::RGB => 3,
            Self::RGBA => 4
        }
    }

    fn get_blank_pixel(&self) -> &[u8] {
        match self {
            Self::RGB => &[0u8; 3],
            Self::RGBA => &[0u8; 4],
        }
    }
}

pub trait BatchableTextureSource {
    fn tex_sub_image_2d(&self, gl:&WebGl2RenderingContext, x:i32, y:i32);
    fn width(&self) -> i32;
    fn height(&self) -> i32;
    fn format(&self) -> TextureFormat;
    fn unique_texture(&self) -> bool;
}

#[derive(PartialEq, Eq)]
pub struct RawTextureSource<'a> {
    pub data:&'a [u8],
    pub format:TextureFormat,
    pub width:i32,
    pub height:i32,
    pub unique:bool
}

impl<'a> BatchableTextureSource for RawTextureSource<'a> {
    fn height(&self) -> i32 { self.height }
    fn width(&self) -> i32 { self.width }
    fn format(&self) -> TextureFormat { self.format }
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
    format:TextureFormat
}

impl TempBlankTextureSource {
    pub fn new(unique:bool, width:i32, height:i32, format:TextureFormat) -> Self {
        Self { unique: unique, width: width, height: height, format: format }
    }
}

impl BatchableTextureSource for TempBlankTextureSource {
    fn format(&self) -> TextureFormat {
        self.format
    }
    fn height(&self) -> i32 {
        self.height
    }
    fn unique_texture(&self) -> bool {
        self.unique
    }
    fn width(&self) -> i32 {
        self.width
    }
    fn tex_sub_image_2d(&self, gl:&WebGl2RenderingContext, x:i32, y:i32) {
        // let arr = [0u8, 1, 1, 1];
        // let mut v = Vec::new();
        // for i in 0..=x*y {
        //     v.extend_from_slice(&arr);
        // }
        // gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_u8_array(WebGl2RenderingContext::TEXTURE_2D, 0, x, y, self.width, self.height, self.format.get_format(), self.format.get_type(), Some(v.as_slice()));
    }
}

pub struct ImageTextureSource {
    image:HtmlImageElement,
    unique:bool
}

impl ImageTextureSource {
    pub fn new(image:HtmlImageElement, unique:bool) -> Self {
        Self { image: image, unique: unique }
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
    batcher:Rc<RefCell<TextureBatcher>>,
    texture_id:u32,
    allocation:Allocation
}

impl PartialEq for BatchedTexture {
    fn eq(&self, other: &Self) -> bool {
        self.texture_id == other.texture_id && self.allocation == other.allocation && Rc::ptr_eq(&self.batcher, &other.batcher)
    }
}

impl Drop for BatchedTexture {
    fn drop(&mut self) {
        let batcher = &self.batcher;
        batcher.borrow_mut().remove(&self);
    }
}

impl BatchedTexture {
    pub fn new(batcher:Rc<RefCell<TextureBatcher>>, src:&dyn BatchableTextureSource) -> Self {
        TextureBatcher::add(batcher, src)
    }

    pub fn update(&mut self, src:&dyn BatchableTextureSource) {
        Rc::clone(&self.batcher).borrow().update(self, src);
    }

    pub fn get_texcoord(&self, x:f32, y:f32) -> (f32,f32) {
        self.batcher.borrow().get_texcoord(self, x, y)
    }

    pub fn get_instance_id(&self) -> u32 {
        return self.texture_id;
    }

    pub fn same_instance(&self, other:&Self) -> bool {
        Rc::ptr_eq(&self.batcher, &other.batcher) && self.texture_id == other.texture_id
    }

    pub fn bind(&self) {
        self.batcher.borrow().bind(self);
    }
}

pub struct TextureBatcher {
    instances:HashMap<u32,TextureBatcherInstance>,
    gl:WebGl2RenderingContext,
    last_instance_id:u32,
    min_width:i32,
    min_height:i32,
}

impl TextureBatcher {
    pub fn new(gl:WebGl2RenderingContext, w:i32, h:i32 ) -> Self {
        Self { 
            instances: HashMap::new(), 
            gl:gl,
            last_instance_id: 0, 
            min_width: w,
            min_height: h,
        }
    }

    pub fn get_texture_count(&self) -> usize {
        self.instances.len()
    }

    fn add(rc:Rc<RefCell<Self>>, src:&dyn BatchableTextureSource) -> BatchedTexture {
        let gl = rc.borrow().gl.clone();
        let unique = src.unique_texture();

        //try to add into existing instance
        if !unique {
            for (instance_id,instance) in rc.borrow_mut().instances.iter_mut() {
                if let Some(batched_texture) = instance.add(&gl, &rc, src, *instance_id) {
                    return batched_texture;
                }
            }
        }

        //create new instance
        rc.borrow_mut().last_instance_id += 1;

        let mut new_instance = TextureBatcherInstance::new(
            &rc,
            rc.borrow().last_instance_id,
            &gl, 
            src.format(), 
            if unique { src.width() } else { i32::max(rc.borrow().min_width, src.width()) },
            if unique { src.height() } else { i32::max(rc.borrow().min_height, src.height()) }
        );

        let result = new_instance.add(&gl, &rc, src, rc.borrow().last_instance_id).expect("Expected new texture batcher instance to succesfully allocate");
        
        let k = rc.borrow().last_instance_id;
        rc.borrow_mut().instances.insert(k, new_instance);
        
        result
    }

    pub fn bind(&self, batched_texture:&BatchedTexture) {
        self.instances.get(&batched_texture.texture_id).expect_throw("Expected texture ID to be valid while binding").bind(&self.gl.clone(),WebGl2RenderingContext::TEXTURE_2D)
    }

    fn get_texcoord(&self, batched_texture:&BatchedTexture, x:f32, y:f32) -> (f32,f32) {
        self.instances.get(&batched_texture.texture_id).expect_throw("Expected texture ID to be valid while adjusting texcoord")
        .adjust_texture_coord(&batched_texture.allocation, x, y)
    }

    fn update(&self, batched_texture:&mut BatchedTexture, src:&dyn BatchableTextureSource) {
        let id = batched_texture.texture_id;
        let gl = self.gl.clone();
        let instance = self.instances.get(&id).expect_throw("Expected texture ID to be valid while updating");
        
        if src.width() != batched_texture.allocation.rectangle.width() ||
           src.height() != batched_texture.allocation.rectangle.height() ||
           src.format() != instance.format {
            *batched_texture = Self::add(batched_texture.batcher.clone(), src);
            return;
        }
        
        instance.update_batched(&gl, &batched_texture.allocation, src);
    }

    fn remove(&mut self, batched_texture:&BatchedTexture) {
        self.instances.get_mut(&batched_texture.texture_id).expect_throw("Expected texture ID to be valid while removing").remove(batched_texture.allocation);
    }
}

impl Drop for TextureBatcher {
    fn drop(&mut self) {
        for instance in self.instances.values() {
            instance.delete_texture(self.gl.clone())
        }
    }
}

pub struct TextureBatcherInstance {
    batcher:Rc<RefCell<TextureBatcher>>,
    instance_id:u32,
    atlas:AtlasAllocator,
    texture:WebGlTexture,
    width:i32,
    height:i32,
    format:TextureFormat,
}

impl TextureBatcherInstance { 
    fn new(rc:&Rc<RefCell<TextureBatcher>>, instance_id:u32, gl:&WebGl2RenderingContext,format:TextureFormat,width:i32,height:i32) -> Self {
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
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

        Self { 
            batcher: rc.clone(),
            instance_id: instance_id,
            atlas: AtlasAllocator::new(size2(width, height)),
            texture: texture, 
            width: width,
            height: height,
            format: format,
        }
    }

    fn add(&mut self, gl:&WebGl2RenderingContext, rc:&Rc<RefCell<TextureBatcher>>, src:&dyn BatchableTextureSource, instance_id:u32) -> Option<BatchedTexture> {
        let (height,width,format) = (src.height(),src.width(),src.format());
        if !(format == self.format && width <= self.width && height <= self.height) {return None;}

        if let Some(allocation) = self.atlas.allocate(size2(width, height)) {
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
            let p = allocation.rectangle.min;
            src.tex_sub_image_2d(gl, p.x, p.y);
            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);

            return Some(BatchedTexture { 
                batcher: rc.clone(), 
                texture_id: instance_id,
                allocation: allocation 
            });
        } else {
            return None;
        }
    }

    fn update_batched(&self, gl:&WebGl2RenderingContext, allocation:&Allocation, src:&dyn BatchableTextureSource) {
        let p = allocation.rectangle.min;
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
        src.tex_sub_image_2d(gl, p.x, p.y);
        gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, None);
    }

    fn bind(&self, gl:&WebGl2RenderingContext, target:u32) {
        gl.bind_texture(target, Some(&self.texture));
    }

    fn adjust_texture_coord(&self, allocation:&Allocation, x:f32, y:f32) -> (f32,f32)  {
        let min = allocation.rectangle.min;
        (
            (min.x as f32 + x*allocation.rectangle.width() as f32)/self.width as f32, 
            (min.x as f32 + y*allocation.rectangle.height() as f32)/self.height as f32
        )
    }

    fn remove(&mut self, allocation:Allocation) {
        self.atlas.deallocate(allocation.id);
        if self.atlas.is_empty() {
            self.batcher.clone().borrow_mut().instances.remove(&self.instance_id);
        }
    }

    fn delete_texture(&self, gl:WebGl2RenderingContext) {
        gl.delete_texture(Some(&self.texture));
    }
}