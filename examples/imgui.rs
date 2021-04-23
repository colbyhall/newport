use newport::*;
use engine::{ Module, Engine, EngineBuilder, Window, WindowEvent };
use imgui::{DrawState, Panel, RawInput};
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

                let mut window = engine.window();

                let dpi = window.dpi();

                let backbuffer = device.acquire_backbuffer();

                let example = engine.module::<ImguiExample>().unwrap();

                let mut gui = example.context.lock().unwrap();
                let mesh = {
                    let mut input = {
                        let mut input = example.input.borrow_mut();
                        input.take()
                    }.unwrap_or_default();
                    
                    input.viewport = (0.0, 0.0, backbuffer.width() as f32 / dpi, backbuffer.height() as f32 / dpi).into();
                    input.dt = dt;
                    input.dpi = 1.0;

                    gui.begin_frame(input);

                    Panel::top("menu_bar", 38.0).build(&mut gui, |builder| {
                        builder.label("bar");
                        
                        if builder.button("soo").clicked() {
                            println!("soo");
                        }
                        
                        builder.label("car");
                        
                        if builder.button("dar").clicked() {
                            println!("dar");
                        }

                        let drag = builder.layout.available_rect();
                        window.set_custom_drag(drag);
                    });

                    gui.end_frame()
                };

                device.update_bindless();

                let mut gfx = device.create_graphics_context().unwrap();
                gfx.begin();
                {
                    gfx.begin_render_pass(&graphics.backbuffer_render_pass(), &[&backbuffer]);
                    gfx.clear(Color::BLACK);
                    example.draw_state.draw(mesh, &mut gfx, &gui);
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