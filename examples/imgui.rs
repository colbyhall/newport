use newport::*;
use engine::{ Module, Engine, EngineBuilder, Window, WindowEvent };
use imgui::{ DrawState, Painter, Mesh, RawInput };
use math::Color;
use graphics::*;

use std::sync::Mutex;
use std::cell::RefCell;

struct ImguiExample {
    draw_state: DrawState,
    context:    Mutex<imgui::Context>,
    input:      RefCell<Option<RawInput>>,

}

impl Module for ImguiExample {
    fn new() -> Self {
        Self{
            draw_state: DrawState::new(),
            context:    Mutex::new(imgui::Context::new()),
            input:      RefCell::new(None),
        }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<graphics::Graphics>()
            .process_input(|engine: &Engine, _window: &Window, event: &WindowEvent| {
                let example = engine.module::<ImguiExample>().unwrap();
                let mut input = example.input.borrow_mut();
                if input.is_none() {
                    *input = Some(RawInput::default());
                }
                input.as_mut().unwrap().events.push_back(event.clone());
            })
            .tick(|engine: &Engine, dt: f32| {
                let graphics = engine.module::<Graphics>().unwrap();
                let device = graphics.device();

                let backbuffer = device.acquire_backbuffer();

                let example = engine.module::<ImguiExample>().unwrap();

                let mesh = {
                    let mut input = {
                        let mut input = example.input.borrow_mut();
                        input.take()
                    }.unwrap_or_default();
                    
                    input.viewport = (0.0, 0.0, backbuffer.width() as f32, backbuffer.height() as f32).into();
                    input.dt = dt;
                    input.dpi = 2.0;

                    let mut gui = example.context.lock().unwrap();
                    gui.begin_frame(input);
                    gui.end_frame()
                };

                device.update_bindless();

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