use std::rc::Rc;

use nalgebra::{Transform2, Point2};

use crate::engine::render::{render_object::{RenderType, VertexAttrib, ShaderDataTypes, RenderObject, UniformAttrib, UniformRole, AttributeRole}, renderer::{Renderer, MappedRenderObject, UniformBlock, Uniform, MappedTexture}};

thread_local! {
    static IMAGE_RENDER_TYPE: Rc<RenderType> = Rc::new(RenderType {
        name:String::from("Rect"),
        instanced:None,
        vertex_shader:String::from(
            "#version 300 es
    
            in vec2 position;
            in vec2 texCoord;

            out vec2 vTexCoord;
    
            void main() {
                gl_Position = vec4(position, 0.0, 1.0);
                vTexCoord = texCoord;
            }"
        ),
        fragment_shader:String::from(
            "#version 300 es

            precision mediump float;

            in vec2 vTexCoord;

            out vec4 fragColor;

            uniform sampler2D texture0;

            void main() {
                fragColor = texture(texture0, vTexCoord);
            }"
        ),
        vertex_attribs:vec![
            VertexAttrib { 
                name: String::from("position"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT_VEC2, 
            }, 
            VertexAttrib { 
                name: String::from("texCoord"), 
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FLOAT_VEC2, 
            }
        ],
        uniform_attribs:vec![
            UniformAttrib {
                name:String::from("texture0"),
                role:UniformRole::Texture
            }
        ],
        instance_attribs:Vec::new(),
        blank_vertex:vec![0.0, 0.0, 0.0, 0.0, 0.0, 0.0],
        vertex_size:4,
        verticies_chunk_min_size:20,
        verticies_chunk_grow_factor:1.1,
        verticies_chunk_max_size:2000,
        indicies_chunk_min_size:1000,
        indicies_chunk_grow_factor:1.1, 
        indicies_chunk_max_size:2000,
    })
}

pub struct Image {
    obj:MappedRenderObject,
    img:MappedTexture,
    img_valid:bool
}

impl Image {
    pub fn from_url(renderer:&mut Renderer, transform:Transform2<f32>, url:String) -> Self {
        let img = renderer.upload_image_from_url(url);

        let (minx,miny) = img.get_texcoord(&renderer,0f32, 0f32);
        let (maxx, maxy) = img.get_texcoord(&renderer,1.0, 1.0);

        let v0:Point2<f32> = transform * Point2::<f32>::new(1.0, 1.0);
        let v1:Point2<f32> = transform * Point2::<f32>::new(-1.0, 1.0);
        let v2:Point2<f32> = transform * Point2::<f32>::new(-1.0, -1.0);
        let v3:Point2<f32> = transform * Point2::<f32>::new(1.0, -1.0);

        let verticies = vec![
            v0.x,v0.y, maxx, maxy,
            v1.x,v1.y, minx,maxy,
            v2.x,v2.y, minx,miny,
            v3.x,v3.y, maxx,miny
        ];

        let indicies:Vec<u16> = vec![0,1,2, 0,2,3];

        let render_object = RenderObject {
            type_id:IMAGE_RENDER_TYPE.with(|f| f.clone()),
            uniforms:UniformBlock::new(vec![Uniform::new("texture0", crate::engine::render::renderer::UnifromType::Texture(img.clone()))]),
            verticies:verticies,
            indicies:indicies
        };

        let mapped = MappedRenderObject::new(renderer, render_object);

        let valid = img.valid();

        Self { obj:mapped, img:img, img_valid:valid }
    }

    pub fn from_mapped(renderer:&mut Renderer, transform:Transform2<f32>, mapped:MappedTexture) -> Self {

        let (minx,miny) = mapped.get_texcoord(&renderer, 0f32, 0f32);
        let (maxx, maxy) = mapped.get_texcoord(&renderer,1.0, 1.0);

        let v0:Point2<f32> = transform * Point2::<f32>::new(1.0, 1.0);
        let v1:Point2<f32> = transform * Point2::<f32>::new(-1.0, 1.0);
        let v2:Point2<f32> = transform * Point2::<f32>::new(-1.0, -1.0);
        let v3:Point2<f32> = transform * Point2::<f32>::new(1.0, -1.0);

        let verticies = vec![
            v0.x,v0.y, maxx, maxy,
            v1.x,v1.y, minx,maxy,
            v2.x,v2.y, minx,miny,
            v3.x,v3.y, maxx,miny
        ];

        let indicies:Vec<u16> = vec![0,1,2, 0,2,3];

        let render_object = RenderObject {
            type_id:IMAGE_RENDER_TYPE.with(|f| f.clone()),
            uniforms:UniformBlock::new(vec![Uniform::new("texture0", crate::engine::render::renderer::UnifromType::Texture(mapped.clone()))]),
            verticies:verticies,
            indicies:indicies
        };

        let obj = MappedRenderObject::new(renderer, render_object);

        let valid = mapped.valid();

        Self { obj:obj, img:mapped, img_valid:valid }
    }

    fn update_render_object(&mut self, renderer:&mut Renderer, transform:Transform2<f32>) {
        let (minx,miny) = self.img.get_texcoord(&renderer, 0f32, 0f32);
        let (maxx, maxy) = self.img.get_texcoord(&renderer,1.0, 1.0);

        let v0:Point2<f32> = transform * Point2::<f32>::new(1.0, 1.0);
        let v1:Point2<f32> = transform * Point2::<f32>::new(-1.0, 1.0);
        let v2:Point2<f32> = transform * Point2::<f32>::new(-1.0, -1.0);
        let v3:Point2<f32> = transform * Point2::<f32>::new(1.0, -1.0);

        let verticies = vec![
            v0.x,v0.y, maxx, maxy,
            v1.x,v1.y, minx,maxy,
            v2.x,v2.y, minx,miny,
            v3.x,v3.y, maxx,miny
        ];

        let indicies:Vec<u16> = vec![0,1,2, 0,2,3];

        let new_render_object = RenderObject {
            type_id:IMAGE_RENDER_TYPE.with(|f| f.clone()),
            uniforms:UniformBlock::new(vec![Uniform::new("texture0", crate::engine::render::renderer::UnifromType::Texture(self.img.clone()))]),
            verticies:verticies,
            indicies:indicies
        };

        self.obj.update(renderer, new_render_object)
    }

    pub fn render(&mut self, renderer:&mut Renderer, transform:Transform2<f32>) {
        if !self.img_valid && self.img.valid() {
            self.update_render_object(renderer, transform);
        }
        self.update_render_object(renderer, transform);
        self.img_valid = self.img.valid();
    }
}