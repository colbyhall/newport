use newport::*;
use gpu::*;
use math::*;
use engine::*;
use graphics::*;

use std::sync::Arc;
use std::mem::size_of;
use std::fs;
use stb_image::image;

struct RenderState {
    render_pass: Arc<RenderPass>,
    pipeline:    Arc<Pipeline>,

    vertex_buffer: Arc<Buffer>,
    texture: Arc<Texture>,
}

struct HelloWorld {
    render_state: Option<RenderState>,
}

#[allow(dead_code)]
struct HelloWorldVertex {
    position: Vector3,
    uv:       Vector2,
}

#[derive(Copy, Clone, Default)]
#[allow(dead_code)]
struct Constants {
    view: Matrix4,
    tex: u32,
}

impl Vertex for HelloWorldVertex {
    fn attributes() -> Vec<VertexAttribute> {
        vec![VertexAttribute::Vector3, VertexAttribute::Vector2]
    }
}

impl ModuleCompileTime for HelloWorld {
    fn new() -> Self {
        Self{ render_state: None }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Graphics>()
            .post_init(|engine| {
                let graphics = engine.module::<Graphics>().unwrap();
                let device   = graphics.device();
                
                let render_pass = RenderPass::new(device.clone(), vec![Format::BGR_U8_SRGB], None).unwrap();
        
                let shader = "
                    ByteAddressBuffer all_buffers[]  : register(t0);
                    Texture2D         all_textures[] : register(t1);
                    SamplerState      all_samplers[] : register(s2);
        
                    struct Constants {
                        float4x4 view;
                        uint tex;
                    };
                    [[vk::push_constant]] Constants constants;
        
                    struct Vertex {
                        float3 position : POSITION;
                        float2 uv       : TEXCOORD0;
                    };
        
                    struct Vertex_Out {
                        float2 uv       : TEXCOORD0;
                        float4 position : SV_POSITION;
                    };
        
                    Vertex_Out main_vs( Vertex IN ){
                        Vertex_Out OUT;
        
                        OUT.uv = IN.uv;
                        OUT.position = mul(constants.view, float4(IN.position, 1.0));
        
                        return OUT;
                    }
        
                    struct Pixel_In {
                        float2 uv : TEXCOORD0;
                    };
        
                    float4 main_ps( Pixel_In IN) : SV_TARGET {
                        Texture2D    my_texture = all_textures[constants.tex];
                        SamplerState my_sampler = all_samplers[constants.tex];
        
                        float4 color = my_texture.Sample(my_sampler, IN.uv, 0);
                        return float4(IN.uv.x, IN.uv.y, color.r, 1.0);
                    }
                ";
        
                let vertex_main = "main_vs".to_string();
                let pixel_main  = "main_ps".to_string();
        
                let vertex_bin = shaders::compile("vertex.hlsl", shader, &vertex_main, ShaderVariant::Vertex).unwrap();
                let pixel_bin  = shaders::compile("pixel.hlsl", shader, &pixel_main, ShaderVariant::Pixel).unwrap();
        
                let vertex_shader = Shader::new(device.clone(), vertex_bin, ShaderVariant::Vertex, vertex_main).unwrap();
                let pixel_shader  = Shader::new(device.clone(), pixel_bin, ShaderVariant::Pixel, pixel_main).unwrap();
        
                let pipeline = PipelineBuilder::new_graphics(render_pass.clone())
                    .shaders(vec![vertex_shader, pixel_shader])
                    .vertex::<HelloWorldVertex>()
                    .push_constant_size::<Constants>()
                    .build().unwrap();
        
                let vert_z = 10.0;
                let size = 500.0;

                let vertices = vec![
                    HelloWorldVertex{
                        position: Vector3::new(-size, -size, vert_z),
                        uv:       Vector2::new(0.0, 1.0),
                    },
                    HelloWorldVertex{
                        position: Vector3::new(-size, size, vert_z),
                        uv:       Vector2::new(0.0, 0.0),
                    },
                    HelloWorldVertex{
                        position: Vector3::new(size, size, vert_z),
                        uv:       Vector2::new(1.0, 0.0),
                    },
                    HelloWorldVertex{
                        position: Vector3::new(-size, -size, vert_z),
                        uv:       Vector2::new(0.0, 1.0),
                    },
                    HelloWorldVertex{
                        position: Vector3::new(size, size, vert_z),
                        uv:       Vector2::new(1.0, 0.0),
                    },
                    HelloWorldVertex{
                        position: Vector3::new(size, -size, vert_z),
                        uv:       Vector2::new(0.0, 1.0),
                    },
                ];
        
                let vertex_buffer = Buffer::new(
                    device.clone(), 
                    BufferUsage::VERTEX, 
                    MemoryType::HostVisible, 
                    vertices.len() * size_of::<HelloWorldVertex>()
                ).unwrap();
        
                vertex_buffer.copy_to(vertices);

                let pixels = fs::read("assets/branding/logo_white.png").unwrap();

                let load_result = image::load_from_memory(&pixels[..]);

                let texture = match load_result {
                    image::LoadResult::ImageU8(image) => {
                        let width = image.width as u32;
                        let height = image.height as u32;

                        let pixels = image.data;

                        let buffer = Buffer::new(
                            device.clone(), 
                            BufferUsage::TRANSFER_SRC, 
                            MemoryType::HostVisible, 
                            pixels.len()
                        ).unwrap();
                        buffer.copy_to(pixels);

                        let tex = Texture::new(
                            device.clone(), 
                            MemoryType::DeviceLocal, 
                            TextureUsage::TRANSFER_DST | TextureUsage::SAMPLED,
                            Format::RGBA_U8,
                            width,
                            height,
                            1,
                            Wrap::Clamp,
                            Filter::Nearest,
                            Filter::Nearest
                        ).unwrap();

                        let mut gfx = GraphicsContext::new(device.clone()).unwrap();
                        gfx.begin();
                        {
                            gfx.resource_barrier_texture(tex.clone(), Layout::Undefined, Layout::TransferDst);
                            gfx.copy_buffer_to_texture(tex.clone(), buffer.clone());
                            gfx.resource_barrier_texture(tex.clone(), Layout::TransferDst, Layout::ShaderReadOnly);
                        }
                        gfx.end();
    
                        let receipt = device.submit_graphics(vec![gfx], &[]);
                        receipt.wait();

                        tex
                    },
                    _ => unimplemented!()
                };
        
                engine.module_mut::<HelloWorld>().unwrap().render_state = Some(RenderState{
                    render_pass: render_pass,
                    pipeline: pipeline,
        
                    vertex_buffer: vertex_buffer,
                    texture: texture
                })
            }
        )
    }
}

impl ModuleRuntime for HelloWorld {
    fn on_tick(&self, _dt: f32) {
        let render_state = self.render_state.as_ref().unwrap();
        let device = Engine::as_ref().module::<Graphics>().unwrap().device();

        let backbuffer = device.acquire_backbuffer();

        device.update_bindless();

        let mut graphics = GraphicsContext::new(device.clone()).unwrap();
        graphics.begin();
        {
            graphics.begin_render_pass(render_state.render_pass.clone(), &[backbuffer.clone()]);
            graphics.bind_pipeline(render_state.pipeline.clone());
            graphics.bind_vertex_buffer(render_state.vertex_buffer.clone());
            let index = graphics.bind_sampled_texture(render_state.texture.clone());

            let view = Matrix4::ortho(backbuffer.width() as f32, backbuffer.height() as f32, 1000.0, 0.1);

            let constants = Constants { 
                view: view,
                tex: index,
            };

            graphics.push_constants(constants);
            graphics.draw(6, 0);
            graphics.end_render_pass();
        }
        graphics.resource_barrier_texture(backbuffer, Layout::ColorAttachment, Layout::Present);
        graphics.end();
        
        let receipt = device.submit_graphics(vec![graphics], &[]);
        device.display(&[receipt]);
        device.wait_for_idle();
    }
}

fn main() {
    let builder = EngineBuilder::new()
        .module::<HelloWorld>()
        .name("Hello World".to_string());
    Engine::run(builder).unwrap();
}