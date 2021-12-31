use std::{
	fmt::Debug,
	ops::Deref,
};

use {
	engine::{
		Builder,
		Engine,
		Module,
	},
	gpu::{
		Buffer,
		BufferUsage,
		Gpu,
		GraphicsPipeline,
		GraphicsRecorder,
		Layout,
		MemoryType,
	},
	graphics::Graphics,
	math::{
		Color,
		Mat4,
		Vec2,
	},
	resources::{
		Handle,
		ResourceManager,
	},
	std::{
		cell::RefCell,
		rc::Rc,
	},
};

pub struct Gui {
	base: Option<WidgetRef>,
	pipeline: Handle<GraphicsPipeline>,
}

impl Module for Gui {
	const LOCAL: bool = true;

	fn new() -> Self {
		let base = WidgetRef::new(Panel::new().slot(PanelSlot::new(Panel::new())));

		println!("{:#?}", base);

		Self {
			base: Some(base),
			pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Graphics>()
			.module::<ResourceManager>()
			.display(|| {
				let gui: &mut Gui = Engine::module_mut_checked().unwrap();
				let window = Engine::window().unwrap();
				let dpi = window.scale_factor() as f32;

				let device = Gpu::device();
				let backbuffer = device.acquire_backbuffer().unwrap();
				let pipeline = gui.pipeline.read();

				let viewport = window.inner_size();
				let viewport = Vec2::new(viewport.width as f32 / dpi, viewport.height as f32 / dpi);

				let proj = Mat4::ortho(viewport.x, viewport.y, 1000.0, 0.1);
				let view = Mat4::translate([-viewport.x / 2.0, -viewport.y / 2.0, 0.0]);

				#[allow(dead_code)]
				struct Imports {
					view: Mat4,
				}

				let imports =
					Buffer::new(BufferUsage::CONSTANTS, MemoryType::HostVisible, 1).unwrap();
				imports.copy_to(&[Imports { view: proj * view }]).unwrap();

				let receipt = GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
						// .bind_pipeline(&pipeline)
						// .bind_vertex_buffer(&vertices)
						// .bind_index_buffer(&indices)
						// .bind_constants("imports", &imports, 0)
						// .draw_indexed(indices.len(), 0)
					})
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.submit();

				device.display(&[receipt]);
				device.wait_for_idle();
			})
	}
}

pub trait Widget: Debug + 'static {
	fn slot(&self, _index: usize) -> Option<&dyn Slot> {
		None
	}

	fn slot_mut(&mut self, _index: usize) -> Option<&mut dyn Slot> {
		None
	}

	fn num_slots(&self) -> Option<usize> {
		None
	}
}

pub trait WidgetContainer: Debug {
	fn parent(&self) -> Option<&WidgetRef>;
	fn parent_mut(&mut self) -> &mut Option<WidgetRef>;

	fn widget(&self) -> &dyn Widget;
	fn widget_mut(&mut self) -> &mut dyn Widget;
}

struct Container<T: Widget> {
	parent: Option<WidgetRef>,
	widget: T,
}

impl<T: Widget> Debug for Container<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("WidgetContainer")
			.field("parent", &self.parent.as_ref().map(|f| f.as_ptr()))
			.field("widget", &self.widget)
			.finish()
	}
}

impl<T: Widget> WidgetContainer for Container<T> {
	fn parent(&self) -> Option<&WidgetRef> {
		self.parent.as_ref()
	}

	fn parent_mut(&mut self) -> &mut Option<WidgetRef> {
		&mut self.parent
	}

	fn widget(&self) -> &dyn Widget {
		&self.widget
	}

	fn widget_mut(&mut self) -> &mut dyn Widget {
		&mut self.widget
	}
}

#[derive(Clone)]
pub struct WidgetRef(Rc<RefCell<dyn WidgetContainer>>);

impl WidgetRef {
	pub fn new(widget: impl Widget) -> Self {
		let result = Self(Rc::new(RefCell::new(Container {
			parent: None,
			widget,
		})));
		{
			let mut container = result.borrow_mut();

			let num_slots = container.widget().num_slots().unwrap_or_default();
			for i in 0..num_slots {
				if let Some(slot) = container.widget_mut().slot_mut(i) {
					let mut child = slot.child_mut().borrow_mut();
					*child.parent_mut() = Some(result.clone())
				}
			}
		}
		result
	}
}

impl Deref for WidgetRef {
	type Target = RefCell<dyn WidgetContainer>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Debug for WidgetRef {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let container = self.borrow();
		f.debug_struct("WidgetRef")
			.field("address", &self.as_ptr())
			.field("container", &container)
			.finish()
	}
}

pub trait Slot: Debug {
	fn child(&self) -> &WidgetRef;
	fn child_mut(&mut self) -> &mut WidgetRef;
}

#[derive(Default)]
pub struct Text {
	text: String, // TODO: This should be some localized string structure probably
	color: Color,
}

impl Text {
	pub fn new(text: impl ToString) -> Self {
		Self {
			text: text.to_string(),
			color: Color::WHITE,
		}
	}

	pub fn color(mut self, color: impl Into<Color>) -> Self {
		self.color = color.into();
		self
	}
}

#[derive(Default, Debug)]
pub struct Panel {
	slot: Option<PanelSlot>,
	color: Color,
}

impl Panel {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn slot(mut self, slot: PanelSlot) -> Self {
		self.slot = Some(slot);
		self
	}
}

impl Widget for Panel {
	fn slot(&self, index: usize) -> Option<&dyn Slot> {
		if index == 0 {
			if let Some(slot) = &self.slot {
				Some(slot)
			} else {
				None
			}
		} else {
			None
		}
	}

	fn slot_mut(&mut self, index: usize) -> Option<&mut dyn Slot> {
		if index == 0 {
			if let Some(slot) = &mut self.slot {
				Some(slot)
			} else {
				None
			}
		} else {
			None
		}
	}

	fn num_slots(&self) -> Option<usize> {
		Some(1)
	}
}

#[derive(Debug)]
pub struct PanelSlot {
	pub child: WidgetRef,

	pub margin: Margin,
	pub padding: Margin,
}

impl PanelSlot {
	pub fn new(child: impl Widget) -> Self {
		Self {
			child: WidgetRef::new(child),

			margin: Margin::default(),
			padding: Margin::default(),
		}
	}
}

impl Slot for PanelSlot {
	fn child(&self) -> &WidgetRef {
		&self.child
	}

	fn child_mut(&mut self) -> &mut WidgetRef {
		&mut self.child
	}
}

#[derive(Debug, Default)]
pub struct Margin {
	pub bottom: f32,
	pub left: f32,
	pub right: f32,
	pub top: f32,
}
