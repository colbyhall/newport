use newport_engine::{ Module, Engine, EngineBuilder, WindowEvent };
use newport_graphics::Graphics;
use newport_gpu::*;
use newport_egui as egui;
use egui::Egui;

use std::sync::{ Mutex, MutexGuard };

struct TestPanel {
    name: String,
    age:  i32,
}

struct EditorInner {
    gui: Egui,

    test_panel: TestPanel,
}

pub struct Editor(Mutex<EditorInner>);

impl Editor {
    fn lock(&self) -> MutexGuard<EditorInner> {
        self.0.lock().unwrap()
    }
}

impl Module for Editor {
    fn new() -> Self {
        Self(Mutex::new(EditorInner{
            gui: Egui::new(),

            test_panel: TestPanel {
                name: "Billy Bob".to_string(),
                age: 45,
            }
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
                let graphics = engine.module::<Graphics>().unwrap();
                let device = graphics.device();

                let mut editor = engine.module::<Editor>().unwrap().lock();

                editor.gui.begin_frame(dt);
                egui::SidePanel::left("side_panel", 300.0).show(&editor.gui.ctx().clone(), |ui| {
                    ui.heading("My egui Application");
                    ui.horizontal(|ui| {
                        ui.label("Your name: ");
                        ui.text_edit_singleline(&mut editor.test_panel.name);
                    });
                    ui.add(egui::Slider::new(&mut editor.test_panel.age, 0..=120).text("age"));
                    if ui.button("Click each year").clicked() {
                        editor.test_panel.age += 1;
                    }
                    ui.label(format!("Hello '{}', age {}", editor.test_panel.name, editor.test_panel.age));

                });
                
                egui::Window::new("Settings").show(&editor.gui.ctx().clone(), |ui| {
                    editor.gui.ctx().settings_ui(ui)
                });

                let (_, clipped_meshes) = editor.gui.end_frame();

                device.update_bindless();

                let backbuffer = device.acquire_backbuffer();
                let mut gfx = device.create_graphics_context().unwrap();
                gfx.begin();
                {
                    gfx.begin_render_pass(&graphics.backbuffer_render_pass(), &[&backbuffer]);
                    gfx.clear((0.0, 0.0, 0.0, 0.0));
                    editor.gui.draw(clipped_meshes, &mut gfx);
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