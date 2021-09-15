use crate::{
	Canvas,
	Event,
	Gui,
	Id,
	InputState,
	Layout,
	Painter,
	RawInput,
	Retained,
	StyleMap,
};
use math::{
	Rect,
	Vector2,
};

use std::collections::HashMap;

struct Layer {
	painter: Painter,
}

pub struct Context {
	pub(crate) input: InputState,
	layers: Vec<Layer>,
	retained: HashMap<Id, Box<dyn Retained>>,

	pub(crate) hovered: Option<Id>,
	pub(crate) focused: Option<Id>,

	pub(crate) style: StyleMap,

	canvas: Rect,
}

impl Context {
	pub fn new() -> Self {
		Self {
			input: InputState {
				mouse_location: None,
				last_mouse_location: None,

				dt: 0.0,
				dpi: 0.0,

				key_pressed: [false; 256],
				key_down: [false; 256],
				last_key_down: [false; 256],

				mouse_button_down: [false; 3],
				last_mouse_button_down: [false; 3],

				scroll: 0.0,

				viewport: Rect::default(),

				text_input: String::new(),
			},
			layers: Vec::with_capacity(32),
			retained: HashMap::with_capacity(128),

			hovered: None,
			focused: None,

			style: StyleMap::new(),

			canvas: Rect::default(),
		}
	}

	pub fn builder(&mut self, id: impl Into<Id>, layout: Layout) -> Gui {
		let mut painter = Painter::new();
		painter.push_scissor(layout.bounds());
		Gui {
			id: id.into(),
			layout,

			painter,
			context: self,
		}
	}

	pub(crate) fn push_layer(&mut self, painter: Painter) {
		self.layers.push(Layer { painter });
	}

	pub fn begin_frame(&mut self, mut input: RawInput) {
		let mut input_state = self.input.clone();

		input_state.last_key_down = input_state.key_down;
		input_state.last_mouse_button_down = input_state.mouse_button_down;
		input_state.scroll = 0.0;
		input_state.last_mouse_location = input_state.mouse_location;
		input_state.text_input = String::new();
		input_state.key_pressed = [false; 256];

		let dpi = input.dpi;

		input.events.drain(..).for_each(|event| match event {
			Event::Key { key, pressed } => {
				let (key_code, _) = key.as_key();
				input_state.key_down[key_code as usize] = pressed;
				if pressed {
					input_state.key_pressed[key_code as usize] = true;
				}
			}
			Event::MouseButton {
				mouse_button,
				pressed,
			} => {
				let code = mouse_button.as_mouse_button();
				input_state.mouse_button_down[code as usize] = pressed;
			}
			Event::MouseMove(x, y) => {
				input_state.mouse_location = Some((x as f32 / dpi, y as f32 / dpi).into());
			}
			Event::MouseLeave => {
				input_state.mouse_location = None;
			}
			Event::MouseWheel(_, y) => {
				input_state.scroll = -y;
			}
			Event::Char(c) => {
				input_state.text_input.push(c);
			}
			_ => {}
		});

		input_state.viewport = (input.viewport.min / dpi, input.viewport.max / dpi).into();
		input_state.dt = input.dt;
		input_state.dpi = dpi;

		self.input = input_state;
		self.canvas = self.input.viewport;
	}

	pub fn end_frame(&mut self) -> Canvas {
		let mut mesh = Canvas {
			vertices: Vec::with_capacity(2048),
			indices: Vec::with_capacity(2048),

			width: (self.input.viewport.width() * self.input.dpi) as u32,
			height: (self.input.viewport.height() * self.input.dpi) as u32,
		};

		self.layers
			.drain(..)
			.for_each(|it| it.painter.tesselate(&mut mesh));

		mesh
	}

	pub fn style(&mut self) -> &mut StyleMap {
		&mut self.style
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

	pub fn take_canvas(&mut self) -> Rect {
		let result = self.canvas;
		self.canvas = Default::default();
		result
	}
}

impl Context {
	pub fn retained<T: Retained + Default + Clone>(&mut self, id: Id) -> T {
		let retained = {
			let entry = self.retained.get_mut(&id);
			if let Some(entry) = entry {
				entry
			} else {
				self.retained.insert(id, Box::new(T::default()));
				self.retained.get_mut(&id).unwrap()
			}
		};

		retained.as_any_mut().downcast_mut::<T>().unwrap().clone()
	}

	pub fn set_retained<T: Retained>(&mut self, id: Id, t: T) {
		self.retained.insert(id, Box::new(t));
	}
}

impl Default for Context {
	fn default() -> Self {
		Self::new()
	}
}
