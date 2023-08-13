use std::rc::Rc;

use cgmath::{Vector2, Matrix3, Vector3, Matrix4, Vector4};

use crate::{engine::render::{render_object::{RenderType, VertexAttrib, ShaderDataTypes, RenderObject, UniformAttrib, UniformRole, AttributeRole}, renderer::{Renderer, UniformData, MappedTexture, VertexData}, texture::{BatchableTextureSource, TextureFilter}}, log_str};

thread_local! {
    static IMAGE_RENDER_TYPE: Rc<RenderType> = Rc::new(RenderType::new_batched_growable(
        String::from(
            "#version 300 es

            uniform mat4 view;
            uniform mat4 projection;
    
            in vec3 position;
            in vec2 texCoord;

            out vec2 vTexCoord;
    
            void main() {
                gl_Position = projection * view * vec4(position, 1.0);
                vTexCoord = texCoord;
            }"
        ),
        String::from(
            "#version 300 es

            precision mediump float;

            in vec2 vTexCoord;

            out vec4 fragColor;

            uniform sampler2D texture0;

            void main() {
                fragColor = texture(texture0, vTexCoord);
                if(fragColor.w == 0.0) {
                    discard;
                }
            }"
        ),
        vec![
            VertexAttrib { 
                name: String::from("position"),
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FloatVec3, 
            }, 
            VertexAttrib { 
                name: String::from("texCoord"), 
                role:AttributeRole::Custom,
                data_type:ShaderDataTypes::FloatVec2, 
            }
        ],
        vec![
            UniformAttrib {
                name:String::from("texture0"),
                role:UniformRole::Custom
            },
            UniformAttrib {
                name:String::from("view"),
                role:UniformRole::View
            },
            UniformAttrib {
                name:String::from("projection"),
                role:UniformRole::Projection
            }
        ],
        Vec::new(),
        100,
        2000,
        400,
        2000,
        1.1,
        1.1
    ));
}

pub struct Image {
    obj:RenderObject,
    img:MappedTexture,
    pos:Matrix4<f32>,
    img_loaded:bool
}

impl Image {
    fn setup_object(renderer:&mut Renderer, transform:Matrix4<f32>, img:&MappedTexture) -> RenderObject {
        let (minx,miny) = img.get_texcoord(&renderer,0f32, 0f32);
        let (maxx, maxy) = img.get_texcoord(&renderer,1.0, 1.0);

        let v0:Vector3<f32> = (transform * Vector4::new(1.0, 1.0,0.0,1.0)).truncate();
        let v1:Vector3<f32> = (transform * Vector4::new(-1.0, 1.0,0.0,1.0)).truncate();
        let v2:Vector3<f32> = (transform * Vector4::new(-1.0, -1.0,0.0,1.0)).truncate();
        let v3:Vector3<f32> = (transform * Vector4::new(1.0, -1.0,0.0,1.0)).truncate();

        //  log_str(&format!("v0: {:?}, v0: {:?}, v0: {:?}, v0: {:?},",v0,v1,v2,v3));

        let mut render_object = RenderObject::new(IMAGE_RENDER_TYPE.with(|f| f.clone()));
        

        render_object.add_triangle([0,1,2]);
        render_object.add_triangle([0,2,3]);

        render_object.set_v_datas(0, "position", vec![VertexData::FloatVec3(v0),VertexData::FloatVec3(v1),VertexData::FloatVec3(v2),VertexData::FloatVec3(v3)]);
        render_object.set_v_datas(0, "texCoord", vec![VertexData::FloatVec2(Vector2 { x: maxx, y: maxy }),VertexData::FloatVec2(Vector2 { x: minx, y: maxy }),VertexData::FloatVec2(Vector2 { x: minx, y: miny }),VertexData::FloatVec2(Vector2 { x: maxx, y: miny })]);
        render_object
    }

    pub fn from_url(renderer:&mut Renderer, transform:Matrix4<f32>, url:String, min_filter:TextureFilter, mag_filter:TextureFilter) -> Self {
        let img = renderer.upload_image_from_url(url,min_filter,mag_filter);

        let mut render_object = Self::setup_object(renderer, transform,&img);

        render_object.set_uniform("texture0", UniformData::Texture(Some(img.clone())));
        render_object.set_uniform("projection", UniformData::Global);
        render_object.set_uniform("view", UniformData::Global);

        let loaded = img.loaded();

        if loaded { render_object.update(renderer); }

        Self { obj:render_object, img:img, img_loaded:loaded, pos:transform }
    }

    pub fn from_mapped(renderer:&mut Renderer, transform:Matrix4<f32>, img:MappedTexture) -> Self {

        let mut render_object = Self::setup_object(renderer, transform,&img);

        render_object.set_uniform("texture0", UniformData::Texture(Some(img.clone())));
        render_object.set_uniform("projection", UniformData::Global);
        render_object.set_uniform("view", UniformData::Global);

        let loaded = img.loaded();

        Self { obj:render_object, img:img, img_loaded:loaded, pos:transform }
    }

    pub fn update_texture_src(&mut self, renderer:&mut Renderer, src:&dyn BatchableTextureSource) {
        self.img.update(renderer, src);
    }

    pub fn update_texture_mapped(&mut self, mapped:MappedTexture) {
        self.img = mapped;
    }

    
    // fn update_render_object(&mut self, renderer:&mut Renderer, transform:Transform2<f32>) {
    //     let (minx,miny) = self.img.get_texcoord(&renderer, 0f32, 0f32);
    //     let (maxx, maxy) = self.img.get_texcoord(&renderer,1.0, 1.0);

    //     let v0:Point2<f32> = transform * Point2::<f32>::new(1.0, 1.0);
    //     let v1:Point2<f32> = transform * Point2::<f32>::new(-1.0, 1.0);
    //     let v2:Point2<f32> = transform * Point2::<f32>::new(-1.0, -1.0);
    //     let v3:Point2<f32> = transform * Point2::<f32>::new(1.0, -1.0);

    //     let verticies = vec![
    //         v0.x,v0.y, maxx, maxy,
    //         v1.x,v1.y, minx,maxy,
    //         v2.x,v2.y, minx,miny,
    //         v3.x,v3.y, maxx,miny
    //     ];

    //     let indicies:Vec<u16> = vec![0,1,2, 0,2,3];

    //     // let new_render_object = RenderObject {
    //     //     type_id:IMAGE_RENDER_TYPE.with(|f| f.clone()),
    //     //     uniforms:UniformBlock::new(vec![Uniform::new("texture0", crate::engine::render::renderer::UnifromData::Texture(self.img.clone()))]),
    //     //     verticies:verticies,
    //     //     indicies:indicies
    //     // };

    //     // self.allocation.update(renderer, &new_render_object)
    // }

    fn update_texcoords(&mut self, renderer:&mut Renderer) {
        let (minx,miny) = self.img.get_texcoord(&renderer, 0f32, 0f32);
        let (maxx, maxy) = self.img.get_texcoord(&renderer,1.0, 1.0);
        self.obj.set_v_datas(0, "texCoord", vec![VertexData::FloatVec2(Vector2 { x: maxx, y: maxy }),VertexData::FloatVec2(Vector2 { x: minx, y: maxy }),VertexData::FloatVec2(Vector2 { x: minx, y: miny }),VertexData::FloatVec2(Vector2 { x: maxx, y: miny })]);
    }

    fn update_pos(&mut self, transform:Matrix4<f32>) {
        let v0:Vector3<f32> = (transform * Vector4::new(1.0, 1.0,0.0,1.0)).truncate();
        let v1:Vector3<f32> = (transform * Vector4::new(-1.0, 1.0,0.0,1.0)).truncate();
        let v2:Vector3<f32> = (transform * Vector4::new(-1.0, -1.0,0.0,1.0)).truncate();
        let v3:Vector3<f32> = (transform * Vector4::new(1.0, -1.0,0.0,1.0)).truncate();

        self.obj.set_v_datas(0, "position", vec![VertexData::FloatVec3(v0),VertexData::FloatVec3(v1),VertexData::FloatVec3(v2),VertexData::FloatVec3(v3)]);
    }

    pub fn render(&mut self, renderer:&mut Renderer, transform:Matrix4<f32>) {
        if (
            if !self.img_loaded && self.img.loaded() {
                self.update_texcoords(renderer);
                self.img_loaded = true;
                true
            } else { false } ||
            if self.pos != transform {
                self.update_pos(transform);
                self.pos = transform;
                true
            } else { false } 
        ) {
            self.obj.update(renderer);
        }
    }

    pub fn render_unchanged(&mut self, renderer:&mut Renderer) {
        if !self.img_loaded && self.img.loaded() {
            self.update_texcoords(renderer);
            self.img_loaded = true;
            self.obj.update(renderer);
        }
    }
}