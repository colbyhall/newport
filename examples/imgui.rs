use newport::*;
use engine::{ Module, Engine, EngineBuilder, Window, WindowEvent };
use imgui::{ DrawState, Painter, Mesh };
use math::Color;
use graphics::*;

use std::sync::Mutex;

struct ImguiExample {
    draw_state: DrawState,
    context:    Mutex<imgui::Context>,
}

impl Module for ImguiExample {
    fn new() -> Self {
        Self{
            draw_state: DrawState::new(),
            context:    Mutex::new(imgui::Context::new())
        }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<graphics::Graphics>()
            .process_input(|engine: &Engine, _window: &Window, event: &WindowEvent| {
                let example = engine.module::<ImguiExample>().unwrap();
                let mut context = example.context.lock().unwrap();
            })
            .tick(|engine: &Engine, _dt: f32| {
                let graphics = engine.module::<Graphics>().unwrap();
                let device = graphics.device();

                let example = engine.module::<ImguiExample>().unwrap();

                let mut mesh = Mesh::default();
                let mut painter = Painter::new();
                painter.rect((0.0, 0.0, 100.0, 100.0));
                painter.tesselate(&mut mesh);

                device.update_bindless();

                let backbuffer = device.acquire_backbuffer();
                let mut gfx = device.create_graphics_context().unwrap();
                gfx.begin();
                {
                    gfx.begin_render_pass(&graphics.backbuffer_render_pass(), &[&backbuffer]);
                    gfx.clear(Color::BLACK);
                    example.draw_state.draw(mesh, &mut gfx);
                    gfx.end_render_pass();
                }
                gfx.resource_barrier_texture(&backbuffer, gpu::Layout::ColorAttachment, gpu::Layout::Present);
                gfx.end();
                
                let receipt = device.submit_graphics(vec![gfx], &[]);
                device.display(&[receipt]);
                device.wait_for_idle();
            })
    }
}

// Start the app runner
fn main() {
    let builder = EngineBuilder::new()
        .module::<ImguiExample>()
        .name("ImguiExample");
    Engine::run(builder);
}