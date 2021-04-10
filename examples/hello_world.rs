use newport::*;
use math::*;
use engine::*;
use gpu::*;
use graphics::*;
use imgui::{ Painter, PainterVertex };
use asset::*;

use std::mem::size_of;
use std::cell::RefCell;

struct RenderState {
    render_pass: RenderPass,
    pipeline:    Pipeline,

    #[allow(dead_code)]
    font: AssetRef<font::FontCollection>,
}

struct HelloWorld {
    render_state: RefCell<Option<RenderState>>,
}

#[derive(Copy, Clone, Default)]
#[allow(dead_code)]
struct Constants {
    view:  Matrix4,
}

impl ModuleCompileTime for HelloWorld {
    fn new() -> Self {
        Self{ render_state: RefCell::new(None) }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Graphics>()
    }
}

impl ModuleRuntime for HelloWorld {
    fn post_init(&mut self, engine: &mut Engine) {
        let asset_manager = engine.module_mut::<AssetManager>().unwrap();
        asset_manager
            .register_collection("assets/".into());
    }

    fn on_startup(&'static mut self) {
        let graphics = Engine::as_ref().module::<Graphics>().unwrap();
        let device   = graphics.device();
        
        let render_pass = device.create_render_pass(vec![Format::BGR_U8_SRGB], None).unwrap();

        let shader = "
            #define NULL 0

            ByteAddressBuffer all_buffers[]  : register(t0);
            Texture2D         all_textures[] : register(t1);
            SamplerState      all_samplers[] : register(s2);

            struct Constants {
                float4x4 view;
            };
            [[vk::push_constant]] Constants constants;

            #define VARIANT_SOLID 0
            #define VARIANT_IMAGE 1
            #define VARIANT_FONT  2

            struct Vertex {
                float3 position : POSITION;
                float2 uv       : TEXCOORD0;
                float4 color    : COLOR;
                
                int variant : INT1;
                int texture : INT2;
            };

            struct Vertex_Out {
                float2 uv       : TEXCOORD0;
                float4 color    : COLOR;

                int variant : INT1;
                int texture : INT2;

                float4 position : SV_POSITION;
            };

            Vertex_Out main_vs( Vertex IN ){
                Vertex_Out OUT;

                OUT.uv      = IN.uv;
                OUT.color   = IN.color;
                OUT.variant = IN.variant;
                OUT.texture = IN.texture;

                OUT.position = mul(constants.view, float4(IN.position, 1.0));

                return OUT;
            }

            struct Pixel_In {
                float2 uv    : TEXCOORD0;
                float4 color : COLOR;

                int variant : INT1;
                int texture : INT2;
            };

            float4 main_ps( Pixel_In IN) : SV_TARGET {
                switch(IN.variant) {
                    case VARIANT_SOLID:
                        return IN.color;
                    case VARIANT_FONT:
                        Texture2D    my_texture = all_textures[IN.texture];
                        SamplerState my_sampler = all_samplers[IN.texture];

                        float4 color = my_texture.Sample(my_sampler, IN.uv, 0);
                        return lerp(float4(0, 0, 0, 0), IN.color, color.r);
                }

                return float4(0, 0, 0, 0);
            }
        ";

        let vertex_main = "main_vs".to_string();
        let pixel_main  = "main_ps".to_string();

        let vertex_bin = shaders::compile("vertex.hlsl", shader, &vertex_main, ShaderVariant::Vertex).unwrap();
        let pixel_bin  = shaders::compile("pixel.hlsl", shader, &pixel_main, ShaderVariant::Pixel).unwrap();

        let vertex_shader = device.create_shader(&vertex_bin[..], ShaderVariant::Vertex, vertex_main).unwrap();
        let pixel_shader  = device.create_shader(&pixel_bin[..], ShaderVariant::Pixel, pixel_main).unwrap();

        let pipeline_desc = PipelineBuilder::new_graphics(render_pass.clone())
            .shaders(vec![vertex_shader, pixel_shader])
            .vertex::<PainterVertex>()
            .push_constant_size::<Constants>()
            .enable_blend()
            .src_alpha_blend(BlendFactor::OneMinusSrcAlpha)
            .build();

        let pipeline = device.create_pipeline(pipeline_desc).unwrap();

        let asset_manager = Engine::as_ref().module::<AssetManager>().unwrap();
        let font = asset_manager.find("assets/fonts/menlo_regular.ttf").unwrap();

        let mut hello_world = self.render_state.borrow_mut();
        *hello_world = Some(RenderState{
            render_pass: render_pass,
            pipeline: pipeline,

            font: font,
        });
    }

    fn on_tick(&self, _dt: f32) {
        let render_state_borrow = self.render_state.borrow();
        let render_state = render_state_borrow.as_ref().unwrap();

        let device = Engine::as_ref().module::<Graphics>().unwrap().device();
        let asset_manager = Engine::as_ref().module::<AssetManager>().unwrap();

        let backbuffer = device.acquire_backbuffer();

        let font = asset_manager.find("assets/fonts/menlo_regular.ttf").unwrap();

        let mut painter = Painter::new();
        painter.rect((Vector2::new(-100.0, -100.0), Vector2::new(100.0, 100.0)).into()).color(Color::MAGENTA);
        painter.text("Hello World".into(), font, 15, Vector2::new(100.0, 100.0)).color(Color::CYAN);
        let painter_vertices = painter.tesselate();

        let buffer = device.create_buffer(
            BufferUsage::VERTEX, 
            MemoryType::HostVisible, 
            painter_vertices.len() * size_of::<PainterVertex>()
        ).unwrap();
        buffer.copy_to(&painter_vertices[..]);

        device.update_bindless();

        let mut graphics = device.create_graphics_context().unwrap();
        {
            graphics.begin();
            {
                graphics.begin_render_pass(&render_state.render_pass, &[&backbuffer]);
                graphics.clear(Color::new(0.01, 0.01, 0.01, 1.0));
                graphics.bind_pipeline(&render_state.pipeline);
                graphics.bind_vertex_buffer(&buffer);
                
                let size = Vector2::new(backbuffer.width() as f32, backbuffer.height() as f32);

                let proj = Matrix4::ortho(size.x, size.y, 1000.0, 0.1);
                let view = Matrix4::translate((-size / 2.0, 0.0).into());
                let constants = Constants { 
                    view:  proj * view,
                };
                
                graphics.push_constants(constants);
                graphics.draw(painter_vertices.len(), 0);
    
                graphics.end_render_pass();
            }
            graphics.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present);
            graphics.end();
        }
        
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