use newport_engine::{ Module, Engine, EngineBuilder, WindowEvent };
use newport_graphics::{ Graphics, Texture };
use newport_egui::*;
use newport_gpu as gpu;
use newport_os as os;

use newport_asset::{ AssetRef, AssetManager };

use std::sync::{ Mutex, MutexGuard, RwLock };

struct EditorAssets {
    engine_icon: AssetRef<Texture>,
}

struct EditorInner {
    gui:    Egui,
    assets: RwLock<Option<EditorAssets>>,
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

                let assets_lock = editor.assets.read().unwrap();
                let assets = assets_lock.as_ref().unwrap();

                let icon = assets.engine_icon.read();

                ui.image(TextureId::User(icon.gpu().bindless().unwrap() as u64), vec2(30.0, 30.0));

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
                    let mut window = engine.window();

                    if ui.button("🗙").clicked() {
                        engine.shutdown();
                    }

                    if ui.button("🗖").clicked() {
                        window.maximize();
                    }

                    if ui.button("🗕").clicked() {
                        window.minimize();
                    }

                    let right_used = size.x - ui.available_width();

                    ui.add_space(used - right_used);

                    let space_left = ui.available_rect_before_wrap_finite();
                    window.set_ignore_drag(!ui.rect_contains_pointer(space_left));

                    ui.centered_and_justified(|ui| {
                        let label = format!("{} - Newport Editor", engine.name());
                        ui.label(label)
                    })
                })
            })
        });

        let (_, clipped_meshes) = editor.gui.end_frame();
        device.update_bindless();

        if !ctx.is_using_pointer() && response.response.hovered() {
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
            gui:    Egui::new(),
            assets: RwLock::new(None),
        }))
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Graphics>()
            .process_input(|engine: &Engine, window: &os::window::Window, event: &WindowEvent| {
                let mut editor = engine.module::<Editor>().unwrap().lock(); // SPEED: Maybe this will be too slow????
                editor.gui.process_input(window, event);
            })
            .tick(|engine: &Engine, dt: f32| {
                let editor = engine.module::<Editor>().unwrap();
                editor.do_frame(dt);
            })
            .post_init(|engine: &Engine| {
                let asset_manager = engine.module::<AssetManager>().unwrap();
                
                let editor = engine.module::<Editor>().unwrap().lock();

                let mut editor_assets = editor.assets.write().unwrap();
                *editor_assets = Some(EditorAssets{
                    engine_icon: asset_manager.find("assets/branding/logo_white_small.tex").unwrap(),
                });
            })
    }
}