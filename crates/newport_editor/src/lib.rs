use newport_engine::{ Module, Engine, EngineBuilder, WindowEvent };
use newport_graphics::Graphics;
use newport_gpu::*;
use newport_egui::Egui;

use std::cell::RefCell;


pub struct Editor {
    gui: RefCell<Egui>,
}

impl Module for Editor {
    fn new() -> Self {
        Self{
            gui: RefCell::new(Egui::new())
        }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Graphics>()
            .process_input(|engine: &Engine, event: &WindowEvent| {
                let editor = engine.module::<Editor>().unwrap();
                let mut gui = editor.gui.borrow_mut();
                gui.process_input(event);
            })
            .tick(|engine: &Engine, dt: f32| {
                let editor = engine.module::<Editor>().unwrap();
                let graphics = engine.module::<Graphics>().unwrap();
                let device = graphics.device();

                let mut gui = editor.gui.borrow_mut();

                gui.begin_frame(dt);
                let (_, clipped_meshes) = gui.end_frame();

                let backbuffer = device.acquire_backbuffer();
                let mut gfx = device.create_graphics_context().unwrap();
                gfx.begin();
                {
                    gfx.begin_render_pass(&graphics.backbuffer_render_pass(), &[&backbuffer]);
                    gfx.clear((0.01, 0.01, 0.01, 1.0));
                    gui.draw(clipped_meshes, &mut gfx);
                    gfx.end_render_pass();
                }
                gfx.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present);
                gfx.end();
                
                let receipt = device.submit_graphics(vec![gfx], &[]);
                device.display(&[receipt]);
                device.wait_for_idle();
            })
    }
}