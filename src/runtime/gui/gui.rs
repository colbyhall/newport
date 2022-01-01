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
		MemoryType,
	},
	graphics::Graphics,
	math::{
		Color,
		Mat3,
		Mat4,
		Point2,
		Rect,
		Vec2,
	},
	resources::{
		Handle,
		ResourceManager,
	},
	std::{
		cell::RefCell,
		fmt::Debug,
		ops::Deref,
		rc::Rc,
	},
};

pub struct Gui {
	base: Option<WidgetRef>,
	pipeline: Handle<GraphicsPipeline>,
	viewport: Vec2,
}

impl Module for Gui {
	const LOCAL: bool = true;

	fn new() -> Self {
		let base = WidgetRef::new(Panel::new().slot(PanelSlot::new(Text::new("Hello World"))));

		Self {
			base: Some(base),
			pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}")
				.unwrap_or_default(),
			viewport: Vec2::ZERO,
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
				let viewport = window.inner_size();
				let viewport = Vec2::new(viewport.width as f32 / dpi, viewport.height as f32 / dpi);

				if let Some(base) = &mut gui.base {
					let mut base = base.borrow_mut();

					let changed = {
						let layout = base.layout_mut();
						if layout.is_none() || gui.viewport != viewport {
							gui.viewport = viewport;
							*layout = Some(Layout {
								local_bounds: Rect::from_min_max(Point2::ZERO, viewport),
								local_to_absolute: Mat3::IDENTITY,
							});
							true
						} else {
							false
						}
					};

					if changed {
						if let Some(layout) = base.layout() {
							base.widget().layout(layout);
						}
					}

					println!("{:#?}", base);
				}

				let device = Gpu::device();
				let backbuffer = device.acquire_backbuffer().unwrap();
				let pipeline = gui.pipeline.read();

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
					.resource_barrier_texture(
						&backbuffer,
						gpu::Layout::ColorAttachment,
						gpu::Layout::Present,
					)
					.submit();

				device.display(&[receipt]);
				device.wait_for_idle();
			})
	}
}

pub trait Widget: Debug + 'static {
	fn layout(&self, _layout: &Layout) {}

	fn desired_size(&self) -> Vec2;

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

	fn layout(&self) -> Option<&Layout>;
	fn layout_mut(&mut self) -> &mut Option<Layout>;
}

#[derive(Debug)]
pub struct Layout {
	pub local_bounds: Rect,
	pub local_to_absolute: Mat3,
}

struct Container<T: Widget> {
	parent: Option<WidgetRef>,
	widget: T,
	layout: Option<Layout>,
}

impl<T: Widget> Debug for Container<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("WidgetContainer")
			.field("parent", &self.parent.as_ref().map(|f| f.as_ptr()))
			.field("widget", &self.widget)
			.field("layout", &self.layout)
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

	fn layout(&self) -> Option<&Layout> {
		self.layout.as_ref()
	}

	fn layout_mut(&mut self) -> &mut Option<Layout> {
		&mut self.layout
	}
}

#[derive(Clone)]
pub struct WidgetRef(Rc<RefCell<dyn WidgetContainer>>);

impl WidgetRef {
	pub fn new(widget: impl Widget) -> Self {
		let result = Self(Rc::new(RefCell::new(Container {
			parent: None,
			widget,
			layout: None,
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

#[derive(Default, Debug)]
pub struct Text {
	text: String, // TODO: This should be some localized string structure probably
	color: Color,
	alignment: Alignment2,
}

impl Text {
	pub fn new(text: impl ToString) -> Self {
		Self {
			text: text.to_string(),
			color: Color::WHITE,
			alignment: Default::default(),
		}
	}

	pub fn color(mut self, color: impl Into<Color>) -> Self {
		self.color = color.into();
		self
	}

	pub fn alignment(mut self, alignment: Alignment2) -> Self {
		self.alignment = alignment;
		self
	}
}

impl Widget for Text {
	fn desired_size(&self) -> Vec2 {
		todo!()
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
	fn layout(&self, layout: &Layout) {
		// if let Some(slot) = &self.slot {}
	}

	fn desired_size(&self) -> Vec2 {
		if let Some(slot) = &self.slot {
			let child = slot.child.borrow();
			child.widget().desired_size() + slot.padding.size()
		} else {
			Vec2::ZERO
		}
	}

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

	pub alignment: Alignment2,
	pub margin: Margin,
	pub padding: Margin,
}

impl PanelSlot {
	pub fn new(child: impl Widget) -> Self {
		Self {
			child: WidgetRef::new(child),

			alignment: Alignment2::default(),
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

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Margin {
	pub bottom: f32,
	pub left: f32,
	pub top: f32,
	pub right: f32,
}

impl Margin {
	pub fn size(self) -> Vec2 {
		Vec2::new(self.left + self.right, self.bottom + self.top)
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Alignment {
	Min,
	Center,
	Max,
	Fill,
}

impl Alignment {
	pub const BOTTOM: Self = Self::Min;
	pub const TOP: Self = Self::Max;

	pub const LEFT: Self = Self::Min;
	pub const RIGHT: Self = Self::Max;
}

impl Default for Alignment {
	fn default() -> Self {
		Self::Center
	}
}

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
pub struct Alignment2 {
	pub vertical: Alignment,
	pub horizontal: Alignment,
}

impl Alignment2 {
	pub const BOTTOM_LEFT: Self = Self {
		vertical: Alignment::BOTTOM,
		horizontal: Alignment::LEFT,
	};

	pub const CENTER_LEFT: Self = Self {
		vertical: Alignment::Center,
		horizontal: Alignment::LEFT,
	};

	pub const TOP_LEFT: Self = Self {
		vertical: Alignment::TOP,
		horizontal: Alignment::LEFT,
	};

	pub const BOTTOM_CENTER: Self = Self {
		vertical: Alignment::BOTTOM,
		horizontal: Alignment::Center,
	};

	pub const CENTER_CENTER: Self = Self {
		vertical: Alignment::Center,
		horizontal: Alignment::Center,
	};

	pub const TOP_CENTER: Self = Self {
		vertical: Alignment::TOP,
		horizontal: Alignment::Center,
	};

	pub const BOTTOM_RIGHT: Self = Self {
		vertical: Alignment::BOTTOM,
		horizontal: Alignment::RIGHT,
	};

	pub const CENTER_RIGHT: Self = Self {
		vertical: Alignment::Center,
		horizontal: Alignment::RIGHT,
	};

	pub const TOP_RIGHT: Self = Self {
		vertical: Alignment::TOP,
		horizontal: Alignment::RIGHT,
	};

	pub const fn new(vertical: Alignment, horizontal: Alignment) -> Self {
		Self {
			vertical,
			horizontal,
		}
	}
}
