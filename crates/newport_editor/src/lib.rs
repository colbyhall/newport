use newport_engine::{ Module, Engine, EngineBuilder, WindowEvent };
use newport_graphics::Graphics;
use newport_egui::*;
use newport_gpu as gpu;

use std::sync::{ Mutex, MutexGuard };

struct EditorInner {
    gui: Egui,
}

pub struct Editor(Mutex<EditorInner>);

impl Editor {
    fn lock(&self) -> MutexGuard<EditorInner> {
        self.0.lock().unwrap()
    }

    fn do_frame(&self, dt: f32) {
        let engine = Engine::as_ref();

        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let mut editor = self.lock();

        editor.gui.begin_frame(dt);

        let ctx = editor.gui.ctx().clone();

        let response = TopPanel::top("title").show(&ctx, |ui|{
            menu::bar(ui, |ui|{
                let original_width = ui.available_width();

                menu::menu(ui, "File", |ui| {
                    if ui.button("Open").clicked() {
                        println!("Hello World");
                    }
                });

                menu::menu(ui, "Edit", |ui| {
                    if ui.button("Open").clicked() {
                        println!("Hello World");
                    }
                });

                menu::menu(ui, "Selection", |ui| {
                    if ui.button("Open").clicked() {
                        println!("Hello World");
                    }
                });

                menu::menu(ui, "View", |ui| {
                    if ui.button("Open").clicked() {
                        println!("Hello World");
                    }
                });

                menu::menu(ui, "Help", |ui| {
                    if ui.button("Open").clicked() {
                        println!("Hello World");
                    }
                });

                // Take full width and fixed height:
                let height = ui.spacing().interact_size.y;
                let size = vec2(ui.available_width(), height);

                let used = original_width - size.x;

                ui.allocate_ui_with_layout(size, Layout::right_to_left(), |ui| {
                    if ui.button("🗙").clicked() {
                        engine.shutdown();
                    }

                    if ui.button("🗖").clicked() {
                        engine.shutdown();
                    }

                    if ui.button("🗕").clicked() {
                        engine.shutdown();
                    }

                    let right_used = size.x - ui.available_width();

                    ui.add_space(used - right_used);
                    ui.centered_and_justified(|ui| {
                        let label = format!("{} - Newport Editor", engine.name());
                        ui.label(label)
                    })
                });
            });
        });

        let (_, clipped_meshes) = editor.gui.end_frame();
        device.update_bindless();

        if !ctx.is_using_pointer() && response.response.hovered() {
            engine.ignore_drag();
        }

        let backbuffer = device.acquire_backbuffer();
        let mut gfx = device.create_graphics_context().unwrap();
        gfx.begin();
        {
            gfx.begin_render_pass(&graphics.backbuffer_render_pass(), &[&backbuffer]);
            gfx.clear((0.01, 0.01, 0.01, 1.0));
            editor.gui.draw(clipped_meshes, &mut gfx);
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
            gui: Egui::new(),
        }))
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Graphics>()
            .process_input(|engine: &Engine, event: &WindowEvent| {
                let mut editor = engine.module::<Editor>().unwrap().lock(); // SPEED: Maybe this will be too slow????
                editor.gui.process_input(event);
            })
            .tick(|engine: &Engine, dt: f32| {
                let editor = engine.module::<Editor>().unwrap();
                editor.do_frame(dt);
            })
    }
}