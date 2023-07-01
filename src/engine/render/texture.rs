use std::{rc::{Weak, Rc}, collections::HashMap, cell::RefCell};

use guillotiere::{AtlasAllocator, size2, Allocation};
use wasm_bindgen::UnwrapThrowExt;
use web_sys::{WebGlTexture, WebGl2RenderingContext};

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

pub struct TextureSource<'a> {
    data:&'a [f32],
    format:TextureFormat,
    width:i32,
    height:i32
}

pub struct BatchedTexture {
    batcher:Rc<RefCell<TextureBatcher>>,
    texture_id:u32,
    allocation:Allocation
}

impl Drop for BatchedTexture {
    fn drop(&mut self) {
        let batcher = &self.batcher;
        batcher.borrow_mut().remove(&self);
    }
}

pub struct TextureBatcher {
    instances:HashMap<u32,TextureBatcherInstance>,
    last_instance_id:u32,
    min_width:i32,
    min_height:i32
}

impl TextureBatcher {
    pub fn new(w:i32, h:i32) -> Self{
        Self { 
            instances: HashMap::new(), 
            last_instance_id: 
            0, 
            min_width: w,
            min_height: h
        }
    }

    pub fn add(rc:Rc<RefCell<Self>>, gl:&WebGl2RenderingContext, src:TextureSource) -> BatchedTexture {
        let instance_id = rc.borrow().last_instance_id;
        for (instance_id,instance) in rc.borrow_mut().instances.iter_mut() {
            if let Some(batchedTexture) = instance.add( gl, &rc, &src, *instance_id) {
                return batchedTexture;
            }
        }

        rc.borrow_mut().last_instance_id += 1;

        let mut new_instance = TextureBatcherInstance::new(
            gl, 
            src.format, 
            i32::max(rc.borrow().min_width, src.width),
            i32::max(rc.borrow().min_height, src.height) 
        );

        let result = new_instance.add(gl, &rc, &src, rc.borrow().last_instance_id).expect("Expected new texture batcher instance to succesfully allocate");

        rc.borrow_mut().instances.insert(instance_id+1, new_instance);
        
        result
    }

    fn remove(&mut self, batchedTexture:&BatchedTexture) {
        self.instances.get_mut(&batchedTexture.texture_id).expect_throw("Expected texture ID to be valid while removing").remove(batchedTexture.allocation);
    }
}

struct TextureBatcherInstance {
    atlas:AtlasAllocator,
    texture:WebGlTexture,
    width:i32,
    height:i32,
    format:TextureFormat,
}

impl TextureBatcherInstance {
    fn new(gl:&WebGl2RenderingContext,format:TextureFormat,width:i32,height:i32) -> Self {
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
            Some(&format.get_blank_pixel())
        );

        Self { 
            atlas: AtlasAllocator::new(size2(width, height)),
            texture: texture, 
            width: width,
            height: height,
            format: format,
        }
    }

    fn add(&mut self, gl:&WebGl2RenderingContext, rc:&Rc<RefCell<TextureBatcher>>, src:&TextureSource, instance_id:u32) -> Option<BatchedTexture> {
        if !(src.format == self.format && src.width <= self.width && src.height <= self.height) {return None;}

        if let Some(allocation) = self.atlas.allocate(size2(src.width, src.height)) {
            let rect = allocation.rectangle;

            gl.bind_texture(WebGl2RenderingContext::TEXTURE_2D, Some(&self.texture));
            unsafe {
                let buffer_view = js_sys::Float32Array::view(src.data);
                gl.tex_sub_image_2d_with_i32_and_i32_and_u32_and_type_and_opt_array_buffer_view(
                    WebGl2RenderingContext::TEXTURE_2D, 
                    0, 
                    rect.min.x, 
                    rect.min.y, 
                    rect.width(), 
                    rect.height(), 
                    self.format.get_format(), 
                    self.format.get_type(), 
                    Some(&buffer_view)
                );
            }

            return Some(BatchedTexture { 
                batcher: rc.clone(), 
                texture_id: instance_id,
                allocation: allocation 
            });
        } else {
            return None;
        }
    }

    fn remove(&mut self, allocation:Allocation) {
        self.atlas.deallocate(allocation.id);
    }
}