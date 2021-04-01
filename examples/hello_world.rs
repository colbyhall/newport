use newport::*;
use gpu::*;
use math::*;

use std::sync::Arc;
use std::fs::read;
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

        let shaders = read("target\\spirv-builder\\spirv-unknown-unknown\\release\\newport_shaders.spv").unwrap();

        let vertex_shader = Shader::new(device.clone(), shaders.clone(), ShaderVariant::Vertex, "main_vs".to_string()).unwrap();
        let pixel_shader = Shader::new(device.clone(), shaders, ShaderVariant::Pixel, "main_fs".to_string()).unwrap();

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