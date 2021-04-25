use crate::{
    Builder, 
    Event, 
    Id, 
    Input, 
    Layout, 
    Mesh, 
    Painter, 
    RawInput, 
    Retained, 
    Style
};
use crate::math::{ Vector2, Rect };

use std::collections::HashMap;

struct Layer {
    painter: Painter,
}

#[derive(Copy, Clone)]
pub struct InputState {
    pub mouse_location: Option<Vector2>,

    pub dt:  f32,
    pub dpi: f32,

    pub key_down: [bool; 256],
    pub last_key_down: [bool; 256],

    pub mouse_button_down: [bool; 3],
    pub last_mouse_button_down: [bool; 3],

    pub viewport: Rect,
}

impl InputState {
    pub fn was_key_pressed(&self, key: Input) -> bool {
        self.key_down[key.as_key().0 as usize] && !self.last_key_down[key.as_key().0 as usize]
    }

    pub fn was_key_released(&self, key: Input) -> bool {
        !self.key_down[key.as_key().0 as usize] && self.last_key_down[key.as_key().0 as usize]
    }

    pub fn was_primary_clicked(&self) -> bool {
        self.mouse_button_down[0] && !self.last_mouse_button_down[0]
    }

    pub fn was_middle_clicked(&self) -> bool {
        self.mouse_button_down[1] && !self.last_mouse_button_down[0]
    }

    pub fn was_secondary_clicked(&self) -> bool {
        self.mouse_button_down[2] && !self.last_mouse_button_down[0]
    }

    pub fn was_primary_released(&self) -> bool {
        !self.mouse_button_down[0] && self.last_mouse_button_down[0]
    }

    pub fn was_middle_released(&self) -> bool {
        !self.mouse_button_down[1] && self.last_mouse_button_down[0]
    }

    pub fn was_secondary_released(&self) -> bool {
        !self.mouse_button_down[2] && self.last_mouse_button_down[0]
    }

    pub fn mouse_is_over(&self, rect: Rect) -> bool {
        match self.mouse_location {
            Some(loc) => rect.point_overlap(loc),
            None => false
        }
    }
}

pub struct Context {
    pub(crate) input:      InputState,
    layers:     Vec<Layer>,
    _retained:   HashMap<Id, Box<dyn Retained>>,

    pub(crate) hovered: Option<Id>,
    pub(crate) focused: Option<Id>,

    pub(crate) style: Option<Style>, // HACK: Style refers to assets and theyre not loaded most of the time a context is created

    canvas: Rect,
}

impl Context {
    pub fn new() -> Self {
        Self {
            input: InputState{
                mouse_location: None,
                
                dt:  0.0,
                dpi: 0.0,

                key_down: [false; 256],
                last_key_down: [false; 256],
                
                mouse_button_down: [false; 3],
                last_mouse_button_down: [false; 3],

                viewport: Rect::default(),
            },
            layers:     Vec::with_capacity(32),
            _retained:   HashMap::with_capacity(128),
            
            hovered: None,
            focused: None,

            style: None,

            canvas: Rect::default(),
        }
    }

    pub fn builder(&mut self, id: impl Into<Id>, layout: Layout) -> Builder {
        Builder{
            id:     id.into(),
            layout: layout,

            painter: Painter::new(),
            context: self,
        }
    }

    pub(crate) fn push_layer(&mut self, painter: Painter) {
        self.layers.push(Layer{ painter });
    }

    pub fn begin_frame(&mut self, mut input: RawInput) {
        let mut input_state = self.input;

        if self.style.is_none() {
            self.style = Some(Style::default());
        }

        input_state.last_key_down = input_state.key_down;
        input_state.last_mouse_button_down = input_state.mouse_button_down;

        input.events.drain(..).for_each(|event| {
            match event {
                Event::Key{ key, pressed } => {
                    let (key_code, _) = key.as_key();
                    input_state.key_down[key_code as usize] = pressed;
                },
                Event::MouseButton{ mouse_button, pressed, position } => {
                    let code = mouse_button.as_mouse_button();
                    input_state.mouse_button_down[code as usize] = pressed;
                    input_state.mouse_location = Some((position.0 as f32, position.1 as f32).into());
                },
                Event::MouseMove(x, y) => {
                    input_state.mouse_location = Some((x as f32, y as f32).into());
                },
                Event::MouseLeave => {
                    input_state.mouse_location = None;
                },
                _ => { }
            }
        });

        input_state.viewport = input.viewport;
        input_state.dt = input.dt;
        input_state.dpi = input.dpi;

        self.input = input_state;
        self.canvas = self.input.viewport;
        self.style = Some(Style::default());
    }

    pub fn end_frame(&mut self) -> Mesh {
        let mut mesh = Mesh{
            vertices: Vec::with_capacity(2048),
            indices: Vec::with_capacity(2048),
        };

        self.layers.drain(..).for_each(|it| it.painter.tesselate(&mut mesh));
        
        mesh
    }

    pub fn style(&self) -> Style {
        match &self.style {
            Some(it) => it.clone(),
            None => Style::default(),
        }
    }

    pub fn set_style(&mut self, style: Style) {
        self.style = Some(style);
    }
}

impl Context {
    pub fn split_canvas_top(&mut self, size: f32) -> Rect {
        let max = self.canvas.max;
        
        self.canvas.max.y -= size;

        let min = Vector2::new(self.canvas.min.x, self.canvas.max.y);

        (min, max).into()
    }

    pub fn split_canvas_bottom(&mut self, size: f32) -> Rect {
        let min = self.canvas.min;
        
        self.canvas.min.y += size;

        let max = Vector2::new(self.canvas.max.x, self.canvas.min.y);

        (min, max).into()
    }
}