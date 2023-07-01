// use std::{rc::Rc, collections::HashMap};

// use web_sys::{WebGlProgram, WebGlTexture };

// use crate::engine::render::{renderObject::RenderType, renderer::{MappedRenderObjectInternal, RenderChunk}};

// thread_local! {
//     static IMAGE_RENDER_TYPE:Rc<RenderType> = Rc::new(RenderType {
//         name:String::from("image"),
//         instanced:None,
//         vertex_shader:String::from(
//             "#version 300 es
    
//             in vec2 position;
    
//             out vec4 vColor;
        
//             void main() {
//                 gl_Position = vec4(position, 0.0, 1.0);
//                 vColor = color;
//             }"
//         ),
//         fragment_shader:String::from(
//             "#version 300 es

//             precision mediump float;
    
//             in vec2 vTextureCoord;
    
//             uniform sampler2D image;
    
//             void main() {
//                 fragColor = texture2D(image, vTextureCoord);
//             }"
//         ),
//         batcher:
//     })
// }

// struct ImageBatcher {
//     chunks:Vec<RenderChunk>,
//     mapped:HashMap<u32,MappedRenderObjectInternal>,
//     last_mapped_id:u32,
//     program:WebGlProgram
// }

// impl ImageBatcher {
//     fn get_metadata_ref() -> Self {
//         ImageBatcher { chunks: Vec::new(), mapped: HashMap::new(), last_mapped_id: (), program: () }
//     }
// }

// struct ImageChunk { texture:WebGlTexture }