pub use newport_imgui::*;

pub(crate) use newport_gpu as gpu;
pub(crate) use newport_os as os;
pub(crate) use newport_math as math;
pub(crate) use newport_engine as engine;
pub(crate) use newport_graphics as graphics;

use engine::{ Module, Engine, EngineBuilder, WindowEvent };
use graphics::{ Graphics };
use math::{ Color, Rect };

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

        let dpi = engine.dpi();
        let backbuffer = device.acquire_backbuffer();

        let mut editor = self.lock();
        let EditorInner {
            gui,
            input,
            draw_state,
            pages,
            selected_page
        } = &mut *editor;

        let mesh = {
            let mut input = input.take().unwrap_or_default();
            
            input.viewport = (0.0, 0.0, backbuffer.width() as f32, backbuffer.height() as f32).into();
            input.dt = dt;
            input.dpi = dpi;

            gui.begin_frame(input);

            // Top title bar which holds the pages, title, and window buttons
            {
                let mut style = gui.style();
                style.padding = (12.0, 5.0, 12.0, 5.0).into();
                style.margin = Rect::default();
                style.inactive_background = DARK.bg_h;
                style.unhovered_background = DARK.bg_h;
                let height = style.label_height_with_padding();
    
                gui.set_style(style);
    
                Panel::top("menu_bar", height).build(gui, |builder| {
                    builder.add_spacing(5.0);

                    for (index, it) in pages.iter().enumerate() {
                        let is_selected = index == *selected_page;
                        if MenuButton::new(it.name(), it.can_close(), is_selected).build(builder).clicked() {
                            if is_selected {
                                *selected_page = 0;
                            } else {
                                *selected_page = index;
                            }
                        }
                    }

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
                    });
                });
            }

            // Bottom bar
            {
                let mut style = Style::default();
                style.padding = (2.0, 2.0, 2.0, 2.0).into();
                style.inactive_background = DARK.yellow1;
                style.inactive_foreground = DARK.bg;

                let height = style.label_height_with_padding();
    
                gui.set_style(style);

                Panel::bottom("bottom_bar", height).build(gui, |builder| {
                    builder.label(format!("{} - Newport Engine", engine.name()));

                    let bounds = builder.layout.push_size(builder.layout.space_left());
                    builder.layout(Layout::right_to_left(bounds), |builder| {
                        builder.label(format!("{:.2}ms [{} FPS] | Idle", dt * 1000.0, engine.fps()));
                    });
                });
            }

            if *selected_page >= pages.len() {
                *selected_page = 0;
            }

            let page = &mut pages[*selected_page];
            page.show(gui);

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

                if engine.window().is_minimized() {
                    return;
                }
            
                editor.do_frame(dt);
            })
    }
}

pub struct MenuButton {
    id:        Id,
    label:     String,
    can_close: bool,
    selected:  bool,
}

impl MenuButton {
    pub fn new(label: impl Into<String>, can_close: bool, selected: bool) -> Self {
        let label = label.into();

        Self {
            id:         Id::from(&label),
            label:      label,
            can_close:  can_close,
            selected:   selected,
        }
    }

    pub fn id(mut self, id: impl ToId) -> Self {
        self.id = id.to_id();
        self
    }
}

impl MenuButton {
    #[must_use = "If a response is not being used then use a label"]
    pub fn build(self, builder: &mut Builder) -> ButtonResponse {
        let style = builder.style();

        let label_rect = style.string_rect(&self.label, style.label_size, None).size();
        let bounds = builder.content_bounds(label_rect);
        
        let response = button_control(self.id, bounds, builder);
        
        let is_focused = self.selected;
        let is_hovered = builder.is_hovered(self.id);
        
        let (background_color, foreground_color) = {
            let background_color = if is_focused {
                DARK.bg_s
            } else if is_hovered {
                style.hovered_background
            } else {
                style.unhovered_background
            };

            let foreground_color = if is_focused {
                style.inactive_foreground
            } else if is_hovered {
                style.hovered_foreground
            } else {
                style.unhovered_foreground
            };

            (background_color, foreground_color)
        };

        builder.painter.rect(bounds).color(background_color).roundness((0.0, 0.0, 5.0, 5.0));
        
        let at = Rect::from_pos_size(bounds.pos(), label_rect).top_left();
        builder.painter
            .text(self.label, at, &style.font, style.label_size, builder.input().dpi)
            .color(foreground_color)
            .scissor(bounds);

        response
    }
}