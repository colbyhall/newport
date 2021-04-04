use newport::*;
use gpu::*;
use math::*;

use std::sync::Arc;
use std::mem::size_of;

struct RenderState {
    device:   Arc<Device>,

    render_pass: Arc<RenderPass>,
    pipeline:    Arc<Pipeline>,

    vertex_buffer: Arc<Buffer>,
}

struct HelloWorld {
    render_state: Option<RenderState>,
}

#[allow(dead_code)]
struct HelloWorldVertex {
    position: Vector3,
    color:    Color,
}

impl Vertex for HelloWorldVertex {
    fn attributes() -> Vec<VertexAttribute> {
        vec![VertexAttribute::Vector3, VertexAttribute::Color]
    }
}

impl engine::ModuleCompileTime for HelloWorld {
    fn new() -> Self {
        Self{ render_state: None }
    }

    fn depends_on(builder: engine::EngineBuilder) -> engine::EngineBuilder {
        builder.module::<asset::AssetManager>()
    }
}

impl engine::ModuleRuntime for HelloWorld {
    fn post_init(&mut self, engine: &mut engine::Engine) {
        let instance = Instance::new().unwrap();
        let device   = Device::new(instance, Some(engine.window().handle())).unwrap();

        let render_pass = RenderPass::new(device.clone(), vec![Format::BGR_U8_SRGB], None).unwrap();

        let shader = "
            ByteAddressBuffer all_buffers[]  : register(t0);
            Texture2D         all_textures[] : register(t1);
            SamplerState      all_samplers[] : register(s2);

            struct Vertex {
                float3 position : POSITION;
                float4 color    : COLOR;
            };

            struct Vertex_Out {
                float4 color : COLOR;
                float4 position: SV_POSITION;
            };

            Vertex_Out main_vs( Vertex IN ){
                Vertex_Out OUT;

                OUT.color = IN.color;
                OUT.position = float4(IN.position, 1.0);

                return OUT;
            }

            float4 main_ps( float4 IN : COLOR) : SV_TARGET {
                return IN;
            }
        ";

        let vertex_main = "main_vs".to_string();
        let pixel_main = "main_ps".to_string();

        let vertex_bin = shaders::compile("vertex.hlsl", shader, &vertex_main, ShaderVariant::Vertex).unwrap();
        let pixel_bin = shaders::compile("pixel.hlsl", shader, &pixel_main, ShaderVariant::Pixel).unwrap();

        let vertex_shader = Shader::new(device.clone(), vertex_bin, ShaderVariant::Vertex, vertex_main).unwrap();
        let pixel_shader = Shader::new(device.clone(), pixel_bin, ShaderVariant::Pixel, pixel_main).unwrap();

        let pipeline = PipelineBuilder::new_graphics(render_pass.clone())
            .shaders(vec![vertex_shader, pixel_shader])
            .vertex::<HelloWorldVertex>()
            .build().unwrap();

        let vertices = vec![
            HelloWorldVertex{
                position: Vector3::new(-0.5, -0.5, 0.0),
                color:    Color::RED,
            },
            HelloWorldVertex{
                position: Vector3::new(0.0, 0.5, 0.0),
                color:    Color::GREEN,
            },
            HelloWorldVertex{
                position: Vector3::new(0.5, -0.5, 0.0),
                color:    Color::BLUE,
            }
        ];

        let vertex_buffer = Buffer::new(
            device.clone(), 
            BufferUsage::VERTEX, 
            MemoryType::HostVisible, 
            vertices.len() * size_of::<HelloWorldVertex>()
        ).unwrap();

        vertex_buffer.copy_to(vertices);

        self.render_state = Some(RenderState{
            device: device,

            render_pass: render_pass,
            pipeline: pipeline,

            vertex_buffer: vertex_buffer,
        })
    }

    fn on_tick(&self, _dt: f32) {
        let render_state = self.render_state.as_ref().unwrap();
        let device = &render_state.device;

        let backbuffer = device.acquire_backbuffer();

        let mut graphics = GraphicsContext::new(device.clone()).unwrap();
        graphics.begin();
        {
            graphics.begin_render_pass(render_state.render_pass.clone(), &[backbuffer.clone()]);
            graphics.bind_pipeline(render_state.pipeline.clone());
            graphics.bind_vertex_buffer(render_state.vertex_buffer.clone());
            graphics.draw(3, 0);
            graphics.end_render_pass();
        }
        graphics.resource_barrier_texture(backbuffer, Layout::ColorAttachment, Layout::Present);
        graphics.end();

        device.update_bindless();
        
        let receipt = device.submit_graphics(vec![graphics], &[]);
        device.display(&[receipt]);
    }
}

fn main() {
    let builder = engine::EngineBuilder::new()
        .module::<HelloWorld>()
        .name("Hello World".to_string());
    engine::Engine::run(builder).unwrap();
}