pub use newport_imgui::*;

pub(crate) use newport_gpu as gpu;
pub(crate) use newport_os as os;
pub(crate) use newport_math as math;
pub(crate) use newport_engine as engine;
pub(crate) use newport_graphics as graphics;

use engine::{ Module, Engine, EngineBuilder, WindowEvent };
use graphics::{ Graphics };
use math::Color;

use std::sync::{ Mutex, MutexGuard };

pub use newport_codegen::Editable;

mod editable;
pub use editable::*;

pub trait Page {
    fn can_close(&self) -> bool {
        true
    }

    fn name(&self) -> &str;

    fn show(&mut self, ctx: &mut Context);
}

#[allow(dead_code)]
struct EditorInner {
    gui:    Context,
    input:  Option<RawInput>,
    draw_state: DrawState,

    pages: Vec<Box<dyn Page>>,
    selected_page: usize,
}

pub struct Editor(Mutex<EditorInner>);

impl Editor {
    pub fn push_page(&self, page: Box<dyn Page>) {
        let mut editor = self.lock();
        editor.pages.push(page);
    }

    fn lock(&self) -> MutexGuard<EditorInner> {
        self.0.lock().unwrap()
    }

    fn do_frame(&self, dt: f32) {
        let engine = Engine::as_ref();

        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let mut editor = self.lock();

        let backbuffer = device.acquire_backbuffer();

        let mesh = {
            let mut input = editor.input.take().unwrap_or_default();
            
            input.viewport = (0.0, 0.0, backbuffer.width() as f32, backbuffer.height() as f32).into();
            input.dt = dt;
            input.dpi = 1.0;

            let gui = &mut editor.gui;
            gui.begin_frame(input);

            gui.end_frame()
        };

        device.update_bindless();

        let mut gfx = device.create_graphics_context().unwrap();
        gfx.begin();
        {
            gfx.begin_render_pass(&graphics.backbuffer_render_pass(), &[&backbuffer]);
            gfx.clear(Color::BLACK);
            editor.draw_state.draw(mesh, &mut gfx, &editor.gui);
            gfx.end_render_pass();
        }
        gfx.resource_barrier_texture(&backbuffer, gpu::Layout::ColorAttachment, gpu::Layout::Present);
        gfx.end();
        
        let receipt = device.submit_graphics(vec![gfx], &[]);
        device.display(&[receipt]);
        device.wait_for_idle();
    }
}

impl Module for Editor {
    fn new() -> Self {
        Self(Mutex::new(EditorInner{
            gui:    Context::new(),
            input:  None,
            draw_state: DrawState::new(),

            pages: Vec::with_capacity(32),
            selected_page: 0,
        }))
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Graphics>()
            .process_input(|engine: &Engine, _window: &os::window::Window, event: &WindowEvent| {
                let mut editor = engine.module::<Editor>().unwrap().lock(); // SPEED: Maybe this will be too slow????

                if editor.input.is_none() {
                    editor.input = Some(RawInput::default());
                }
                editor.input.as_mut().unwrap().events.push_back(event.clone());
            })
            .tick(|engine: &Engine, dt: f32| {
                let editor = engine.module::<Editor>().unwrap();

                {
                    let window = engine.window();
                    if window.is_minimized() {
                        return;
                    }
                }
                editor.do_frame(dt);
            })
    }
}