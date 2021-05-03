use crate::{
    engine,
    graphics,
    math,
    asset,
    gpu,
    os,

    Context,
    RawInput,
    DrawState,
    View,
    DARK,
    Layout,
    Panel,
    Style,
    Sizing,
    SPACING,
};
use engine::{ Module, Engine, EngineBuilder, InputEvent };
use graphics::{ Graphics, Texture };
use math::{ Color, Rect };
use asset::{ AssetRef, AssetManager };

use std::sync::{ Mutex, MutexGuard };

struct EditorAssets {
    _close_button: AssetRef<Texture>,
}

impl EditorAssets {
    fn new() -> Self {
        let asset_manager = Engine::as_ref().module::<AssetManager>().unwrap();
        Self{
            _close_button: asset_manager.find("assets/editor/close_button.tex").unwrap(),
        }
    }
}

#[allow(dead_code)]
struct EditorInner {
    gui:   Context,
    input: Option<RawInput>,

    draw_state: DrawState,
    assets:     EditorAssets,
    view:       View,
}

pub struct Editor(Mutex<EditorInner>);

impl Editor {
    pub fn set_view(&self, view: View) {
        let mut editor = self.lock();
        editor.view = view;
    }

    fn lock(&self) -> MutexGuard<EditorInner> {
        self.0.lock().unwrap()
    }

    fn do_frame(&self, dt: f32) {
        let engine = Engine::as_ref();

        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let dpi = engine.dpi();
        let backbuffer = device.acquire_backbuffer();

        let mut editor = self.lock();
        let EditorInner {
            gui,
            input,
            draw_state,
            view,
            ..
        } = &mut *editor;

        let mesh = {
            let mut input = input.take().unwrap_or_default();
            
            input.viewport = (0.0, 0.0, backbuffer.width() as f32, backbuffer.height() as f32).into();
            input.dt = dt;
            input.dpi = dpi;

            gui.begin_frame(input);

            // Top title bar which holds the pages, title, and window buttons
            let mut style = gui.style();
            style.padding = (12.0, 8.0, 12.0, 8.0).into();
            style.margin = Rect::default();
            style.inactive_background = DARK.bg_h;
            style.unhovered_background = DARK.bg_h;
            let height = style.label_height_with_padding();

            gui.set_style(style);

            Panel::top("menu_bar", height).build(gui, |builder| {
                let space = builder.available_rect();

                builder.button("File").clicked();
                builder.button("Edit").clicked();
                builder.button("View").clicked();
                builder.button("Run").clicked();
                builder.button("Help").clicked();

                let bounds = builder.layout.push_size(builder.layout.space_left());
                builder.layout(Layout::right_to_left(bounds), |builder| {
                    let og = builder.style();
                    let mut new = og.clone();
                    new.hovered_background = DARK.red0;
                    new.hovered_foreground = DARK.fg;
                    new.focused_background = DARK.red0;
                    new.focused_foreground = DARK.fg;
                    builder.set_style(new);
                    
                    if builder.button("Close").clicked() {
                        engine.shutdown();
                    }

                    builder.set_style(og);

                    if builder.button("Max").clicked() {
                        engine.maximize();
                    }

                    if builder.button("Min").clicked() {
                        engine.minimize();
                    }

                    let drag = builder.layout.available_rect();
                    let drag = Rect::from_pos_size(drag.pos() * builder.input().dpi, drag.size() * builder.input().dpi);
                    engine.set_custom_drag(drag);

                    builder.layout(Layout::left_to_right(space), |builder| {
                        let mut new = Style::default();
                        new.sizing = Sizing::Fill(true, true);
                        builder.set_style(new);

                        builder.label(format!("{} - Newport Editor", Engine::as_ref().name()));
                    });
                });
            });

            // Main view which all views are built off of
            let bounds = gui.take_canvas();
            let mut builder = gui.builder("view", Layout::up_to_down(bounds));
            let style = builder.style();
            builder.painter.rect(bounds).color(style.inactive_background);

            let bounds = Rect::from_min_max(bounds.min + SPACING, bounds.max - SPACING);
            builder.layout(Layout::up_to_down(bounds), |builder| {
                view.build(builder);
            });
            
            builder.finish();

            gui.end_frame()
        };

        device.update_bindless();

        let mut gfx = device.create_graphics_context().unwrap();
        gfx.begin();
        {
            gfx.begin_render_pass(&graphics.backbuffer_render_pass(), &[&backbuffer]);
            gfx.clear(Color::BLACK);
            draw_state.draw(mesh, &mut gfx, gui);
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
            assets:     EditorAssets::new(),

            view: View::new("main", 1.0),
        }))
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Graphics>()
            .process_input(|engine: &Engine, _window: &os::window::Window, event: &InputEvent| {
                let mut editor = engine.module::<Editor>().unwrap().lock(); // SPEED: Maybe this will be too slow????

                if editor.input.is_none() {
                    editor.input = Some(RawInput::default());
                }
                editor.input.as_mut().unwrap().events.push_back(event.clone());
            })
            .tick(|engine: &Engine, dt: f32| {
                let editor = engine.module::<Editor>().unwrap();

                if engine.window().is_minimized() {
                    return;
                }
            
                editor.do_frame(dt);
            })
    }
}