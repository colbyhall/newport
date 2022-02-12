#![feature(associated_type_defaults)]

use {
	draw2d::{
		Draw2d,
		FontCollection,
		Painter,
	},
	engine::{
		Builder,
		Engine,
		Event as EngineEvent,
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
	input::*,
	math::{
		Color,
		Mat3,
		Mat4,
		Point2,
		Rect,
		Vec2,
		Vec3,
	},
	resources::{
		Handle,
		ResourceManager,
	},
	std::{
		any::Any,
		cell::RefCell,
		fmt::Debug,
		ops::Deref,
		rc::Rc,
	},
};

#[allow(dead_code)]
pub struct Gui {
	canvas: WidgetHandle,
	pipeline: Handle<GraphicsPipeline>,
	viewport: Vec2,

	hovered: Option<WidgetHandle>,
	focused: Option<WidgetHandle>,
}

impl Gui {
	pub fn canvas(&self) -> &WidgetHandle {
		&self.canvas
	}

	fn hit_test(&self, current: &WidgetHandle, mouse_position: Vec2) -> Option<WidgetHandle> {
		let borrow = current.borrow();

		let num_children = borrow.num_children();
		for i in 0..num_children {
			if let Some(child) = borrow.child(i) {
				if let Some(result) = self.hit_test(child, mouse_position) {
					return Some(result);
				}
			}
		}

		let layout = borrow.layout()?;
		if layout.absolute_bounds().point_overlap(mouse_position)
			&& borrow.visibility() == Visibility::Visible
		{
			Some(current.clone())
		} else {
			None
		}
	}

	fn handle_event(&mut self, widget: WidgetHandle, event: Event) {
		let mut container = widget.borrow_mut();
		let reply = container.handle_event(event);
		match reply {
			Reply::Focus => {
				self.focused = Some(widget.clone());
				container.set_focused(true);
			}
			Reply::Unfocus => {
				self.focused = None;
				container.set_focused(false);
			}
			_ => {}
		}
	}
}

impl Module for Gui {
	const LOCAL: bool = true;

	fn new() -> Self {
		Self {
			canvas: WidgetHandle::new(Canvas),
			pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}")
				.unwrap_or_default(),
			viewport: Vec2::ZERO,

			hovered: None,
			focused: None,
		}
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder
			.module::<Draw2d>()
			.module::<ResourceManager>()
			.process_input(|event| {
				let gui: &mut Gui = Engine::module_mut_checked().unwrap();

				match event {
					EngineEvent::MouseMove(x, y) => {
						let window = Engine::window().unwrap();
						let dpi = window.scale_factor() as f32;
						let mouse_position = Vec2::new(*x / dpi, *y / dpi);
						let old = gui.hovered.clone();
						gui.hovered = gui.hit_test(&gui.canvas, mouse_position);
						if gui.hovered != old {
							if let Some(old) = old {
								{
									let mut borrow = old.borrow_mut();
									borrow.set_hovered(false);
								}
								gui.handle_event(old, Event::UnHovered);
							}
							if let Some(hovered) = gui.hovered.clone() {
								{
									let mut borrow = hovered.borrow_mut();
									borrow.set_hovered(true);
								}
								gui.handle_event(hovered, Event::Hovered);
							}
						}
					}
					EngineEvent::MouseLeave => {
						if let Some(hovered) = gui.hovered.clone() {
							{
								let mut borrow = hovered.borrow_mut();
								borrow.set_hovered(false);
							}
							gui.handle_event(hovered, Event::UnHovered);
						}
						gui.hovered = None;
					}
					EngineEvent::Char(c) => {
						if let Some(focused) = gui.focused.clone() {
							gui.handle_event(focused, Event::Char(*c));
						}
					}
					EngineEvent::MouseButton {
						mouse_button,
						pressed,
					} => {
						if *pressed {
							if let Some(hovered) = gui.hovered.clone() {
								gui.handle_event(
									hovered,
									Event::Button {
										input: *mouse_button,
										pressed: *pressed,
									},
								);
							}
						} else {
							let event = Event::Button {
								input: *mouse_button,
								pressed: *pressed,
							};
							if let Some(focused) = gui.focused.clone() {
								gui.handle_event(focused, event);
							} else if let Some(hovered) = gui.hovered.clone() {
								gui.handle_event(hovered, event);
							}
						}
					}
					EngineEvent::Key { key, pressed } => {
						if let Some(hovered) = gui.hovered.clone() {
							gui.handle_event(
								hovered,
								Event::Button {
									input: *key,
									pressed: *pressed,
								},
							);
						}
					}
					_ => {}
				}
			})
			.display(|| {
				let gui: &mut Gui = Engine::module_mut_checked().unwrap();

				let window = Engine::window().unwrap();
				let dpi = window.scale_factor() as f32;
				let viewport = window.inner_size();
				let viewport = Vec2::new(viewport.width as f32 / dpi, viewport.height as f32 / dpi);

				let mut canvas = gui.canvas.borrow_mut();
				if canvas.layout().is_none() || gui.viewport != viewport {
					gui.viewport = viewport;
					canvas.update_layout(Layout {
						local_bounds: Rect::from_min_max(Point2::ZERO, viewport),
						local_to_absolute: Mat3::IDENTITY,
					});
				}

				let mut painter = Painter::new();
				canvas.paint(&mut painter);
				if !painter.is_empty() {
					let (vertices, indices) = painter.finish().unwrap();
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
								.set_pipeline(&pipeline)
								.set_vertex_buffer(&vertices)
								.set_index_buffer(&indices)
								.set_constants("imports", &imports, 0)
								.draw_indexed(indices.len(), 0)
						})
						.texture_barrier(
							&backbuffer,
							gpu::Layout::ColorAttachment,
							gpu::Layout::Present,
						)
						.submit();

					device.display(&[receipt]);
					device.wait_for_idle();
				}
			})
	}
}

#[allow(unused_variables)]
pub trait Widget: Debug + 'static + Sized {
	type Slot: Slot = InvalidSlot;
	const MAX_SLOTS: Option<usize> = Some(0);
	const DEFAULT_VISIBILITY: Visibility;

	fn update_layout(container: &WidgetContainer<Self>) {}

	fn desired_size(container: &WidgetContainer<Self>) -> Vec2;

	fn paint(container: &WidgetContainer<Self>, painter: &mut Painter) {}

	fn handle_event(container: &mut WidgetContainer<Self>, event: Event) -> Reply {
		Reply::None
	}
}

pub trait Slot: Debug + 'static {
	fn new(child: WidgetHandle) -> Self;
	fn child(&self) -> &WidgetHandle;
	fn child_mut(&mut self) -> &mut WidgetHandle;
}

#[derive(Debug)]
pub struct InvalidSlot;
impl Slot for InvalidSlot {
	fn new(_widget: WidgetHandle) -> Self {
		unreachable!()
	}

	fn child(&self) -> &WidgetHandle {
		unreachable!()
	}

	fn child_mut(&mut self) -> &mut WidgetHandle {
		unreachable!()
	}
}

#[derive(Debug, Clone, Copy)]
pub struct Layout {
	pub local_bounds: Rect,
	pub local_to_absolute: Mat3,
}

impl Layout {
	pub fn new(parent: Layout, local_bounds: Rect) -> Self {
		Self {
			local_bounds,
			local_to_absolute: parent.local_to_absolute * Mat3::translate(parent.local_bounds.min),
		}
	}

	pub fn absolute_bounds(&self) -> Rect {
		let min = self.local_to_absolute * Vec3::append(self.local_bounds.min, 1.0);
		let max = self.local_to_absolute * Vec3::append(self.local_bounds.max, 1.0);
		Rect::from_min_max(min.xy(), max.xy())
	}
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Visibility {
	Visible,
	Hidden,
	Collapsed,
	HitTestInvisible,
}

pub enum Event {
	Hovered,
	UnHovered,
	Button { input: Input, pressed: bool },
	Char(char),
}

pub enum Reply {
	None,
	Focus,
	Unfocus,
}

pub enum EventHandler<T: Widget> {
	None,
	Closure(Box<dyn FnMut(&WidgetContainer<T>) + 'static>),
}

impl<T: Widget> Default for EventHandler<T> {
	fn default() -> Self {
		Self::None
	}
}

impl<T: Widget> Debug for EventHandler<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::None => f.write_str("None"),
			Self::Closure(_) => f.write_str("Closure"),
		}
	}
}

pub struct WidgetContainer<T: Widget> {
	pub widget: T,
	pub parent: Option<WidgetHandle>,
	pub slots: Vec<T::Slot>,
	pub visibility: Visibility,
	pub focused: bool,
	pub hovered: bool,

	pub layout: Option<Layout>,
}

impl<T: Widget> WidgetContainer<T> {
	pub fn slot(&mut self, widget: impl Widget) -> &mut T::Slot {
		self.slots.push(T::Slot::new(WidgetHandle::new(widget)));
		self.slots.last_mut().unwrap()
	}
	pub fn slot_with<W: Widget>(
		&mut self,
		widget: W,
		slots: impl FnOnce(&mut LayoutBuilder<W::Slot>),
	) -> &mut T::Slot {
		self.slots
			.push(T::Slot::new(WidgetHandle::new_with(widget, slots)));
		self.slots.last_mut().unwrap()
	}
}

impl<T: Widget> Debug for WidgetContainer<T> {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("WidgetContainer")
			.field("parent", &self.parent.as_ref().map(|f| f.as_ptr()))
			.field("widget", &self.widget)
			.field("slots", &self.slots)
			.field("visibility", &self.visibility)
			.field("layout", &self.layout)
			.finish()
	}
}

pub trait AnyWidgetContainer: Debug {
	fn parent(&self) -> Option<&WidgetHandle>;
	fn parent_mut(&mut self) -> &mut Option<WidgetHandle>;

	fn as_any(&self) -> &dyn Any;
	fn as_any_mut(&mut self) -> &mut dyn Any;

	fn set_focused(&mut self, focused: bool);
	fn set_hovered(&mut self, focused: bool);

	fn update_layout(&mut self, layout: Layout);
	fn layout(&self) -> Option<&Layout>;

	fn desired_size(&self) -> Vec2;
	fn paint(&self, painter: &mut Painter);

	fn handle_event(&mut self, event: Event) -> Reply;

	fn child(&self, index: usize) -> Option<&WidgetHandle>;
	fn child_mut(&mut self, index: usize) -> Option<&mut WidgetHandle>;
	fn num_children(&self) -> usize;

	fn visibility(&self) -> Visibility;

	fn id(&self) -> usize;
}

impl<T: Widget> AnyWidgetContainer for WidgetContainer<T> {
	fn parent(&self) -> Option<&WidgetHandle> {
		self.parent.as_ref()
	}

	fn parent_mut(&mut self) -> &mut Option<WidgetHandle> {
		&mut self.parent
	}

	fn as_any(&self) -> &dyn Any {
		self
	}

	fn as_any_mut(&mut self) -> &mut dyn Any {
		self
	}

	fn set_focused(&mut self, focused: bool) {
		self.focused = focused;
	}

	fn set_hovered(&mut self, hovered: bool) {
		self.hovered = hovered;
	}

	fn update_layout(&mut self, layout: Layout) {
		self.layout = Some(layout);
		T::update_layout(self);
	}

	fn desired_size(&self) -> Vec2 {
		T::desired_size(self)
	}

	fn paint(&self, painter: &mut Painter) {
		T::paint(self, painter);

		// Draw the bounds for this widget
		// if let Some(layout) = &self.layout {
		// 	let absolute = layout.absolute_bounds();
		// 	painter.stroke_rect(absolute, 1.0, Color::RED);
		// }
	}

	fn handle_event(&mut self, event: Event) -> Reply {
		T::handle_event(self, event)
	}

	fn layout(&self) -> Option<&Layout> {
		self.layout.as_ref()
	}

	fn child(&self, index: usize) -> Option<&WidgetHandle> {
		self.slots.get(index).map(|f| f.child())
	}

	fn child_mut(&mut self, index: usize) -> Option<&mut WidgetHandle> {
		self.slots.get_mut(index).map(|f| f.child_mut())
	}

	fn num_children(&self) -> usize {
		self.slots.len()
	}

	fn visibility(&self) -> Visibility {
		self.visibility
	}

	fn id(&self) -> usize {
		self as *const Self as usize
	}
}

#[derive(Default)]
pub struct LayoutBuilder<T: Slot> {
	slots: Vec<T>,
}

impl<T: Slot> LayoutBuilder<T> {
	pub fn slot(&mut self, widget: impl Widget) -> &mut T {
		self.slots.push(T::new(WidgetHandle::new(widget)));
		self.slots.last_mut().unwrap()
	}
	pub fn slot_with<W: Widget>(
		&mut self,
		widget: W,
		slots: impl FnOnce(&mut LayoutBuilder<W::Slot>),
	) -> &mut T {
		self.slots
			.push(T::new(WidgetHandle::new_with(widget, slots)));
		self.slots.last_mut().unwrap()
	}
}

#[derive(Clone)]
pub struct WidgetHandle(Rc<RefCell<dyn AnyWidgetContainer>>);

impl WidgetHandle {
	pub fn new<T: Widget>(widget: T) -> Self {
		Self(Rc::new(RefCell::new(WidgetContainer {
			parent: None,
			widget,
			layout: None,
			visibility: T::DEFAULT_VISIBILITY,
			slots: Vec::default(),
			focused: false,
			hovered: false,
		})))
	}

	pub fn new_with<T: Widget>(widget: T, slots: impl FnOnce(&mut LayoutBuilder<T::Slot>)) -> Self {
		let mut builder: LayoutBuilder<T::Slot> = LayoutBuilder { slots: Vec::new() };
		slots(&mut builder);

		let result = Self(Rc::new(RefCell::new(WidgetContainer {
			parent: None,
			widget,
			layout: None,
			visibility: T::DEFAULT_VISIBILITY,
			slots: builder.slots,
			focused: false,
			hovered: false,
		})));

		{
			let mut container = result.borrow_mut();

			let num_children = container.num_children();
			for i in 0..num_children {
				if let Some(child) = container.child_mut(i) {
					let mut child = child.borrow_mut();
					*child.parent_mut() = Some(result.clone())
				}
			}
		}
		result
	}
}

impl PartialEq for WidgetHandle {
	fn eq(&self, other: &Self) -> bool {
		let a = self.borrow();
		let b = other.borrow();
		a.id() == b.id()
	}
}

impl Deref for WidgetHandle {
	type Target = RefCell<dyn AnyWidgetContainer>;
	fn deref(&self) -> &Self::Target {
		&self.0
	}
}

impl Debug for WidgetHandle {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let container = self.borrow();
		f.debug_struct("WidgetRef")
			.field("address", &self.as_ptr())
			.field("container", &container)
			.finish()
	}
}

pub type Padding = Margin;

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

impl From<f32> for Margin {
	fn from(x: f32) -> Self {
		Self {
			bottom: x,
			left: x,
			top: x,
			right: x,
		}
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

	pub const FILL_LEFT: Self = Self {
		vertical: Alignment::Fill,
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

	pub const FILL_CENTER: Self = Self {
		vertical: Alignment::Fill,
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

	pub const FILL_RIGHT: Self = Self {
		vertical: Alignment::Fill,
		horizontal: Alignment::RIGHT,
	};

	pub const BOTTOM_FILL: Self = Self {
		vertical: Alignment::BOTTOM,
		horizontal: Alignment::Fill,
	};

	pub const CENTER_FILL: Self = Self {
		vertical: Alignment::Center,
		horizontal: Alignment::Fill,
	};

	pub const TOP_FILL: Self = Self {
		vertical: Alignment::TOP,
		horizontal: Alignment::Fill,
	};

	pub const FILL_FILL: Self = Self {
		vertical: Alignment::Fill,
		horizontal: Alignment::Fill,
	};

	pub const fn new(vertical: Alignment, horizontal: Alignment) -> Self {
		Self {
			vertical,
			horizontal,
		}
	}
}

pub fn rect_in_rect(parent: Rect, desired: Vec2, alignment: Alignment2) -> Rect {
	// Used for centering
	let parent_size = parent.size();

	let (x0, x1) = match alignment.horizontal {
		Alignment::LEFT => {
			let x0 = parent.min.x;
			let x1 = x0 + desired.x;
			(x0, x1)
		}
		Alignment::RIGHT => {
			let x1 = parent.max.x;
			let x0 = x1 - desired.x;
			(x0, x1)
		}
		Alignment::Center => {
			let center = parent.min.x + parent_size.x / 2.0;
			let half_desired = desired.x / 2.0;
			(center - half_desired, center + half_desired)
		}
		Alignment::Fill => (parent.min.x, parent.max.x),
	};

	let (y0, y1) = match alignment.vertical {
		Alignment::BOTTOM => {
			let y0 = parent.min.y;
			let y1 = x0 + desired.y;
			(y0, y1)
		}
		Alignment::TOP => {
			let y1 = parent.max.y;
			let y0 = y1 - desired.y;
			(y0, y1)
		}
		Alignment::Center => {
			let center = parent.min.y + parent_size.y / 2.0;
			let half_desired = desired.y / 2.0;
			(center - half_desired, center + half_desired)
		}
		Alignment::Fill => (parent.min.y, parent.max.y),
	};

	Rect::from((x0, y0, x1, y1))
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Sizing {
	Automatic,
	Fill(f32),
}

impl Default for Sizing {
	fn default() -> Self {
		Self::Automatic
	}
}

#[derive(Default, Debug)]
pub struct Text {
	pub text: String, // TODO: This should be some localized string structure probably
	pub font: Handle<FontCollection>,
	pub size: u32,
	pub color: Color,
	pub alignment: Alignment2,
}

impl Text {
	pub fn new(text: impl ToString) -> Self {
		Self {
			text: text.to_string(),
			font: Handle::default(),
			size: 12,
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
	type Slot = InvalidSlot;
	const DEFAULT_VISIBILITY: Visibility = Visibility::HitTestInvisible;

	fn desired_size(container: &WidgetContainer<Self>) -> Vec2 {
		let font = container.widget.font.read();
		let font = font
			.font_at_size(container.widget.size, 2.0)
			.expect("Invalid font size");
		font.string_rect(&container.widget.text, 1000000.0).size()
	}

	fn paint(container: &WidgetContainer<Self>, painter: &mut Painter) {
		let absolute = container.layout.unwrap().absolute_bounds();

		let font = container.widget.font.read();
		let font = font
			.font_at_size(container.widget.size, 2.0)
			.expect("Invalid font size");

		painter.text(
			&container.widget.text,
			container.widget.color,
			absolute.top_left(),
			&font,
		);
	}
}

#[derive(Default, Debug)]
pub struct Button {
	normal: Color,
	hovered: Color,
	focused: Color,

	on_pressed: EventHandler<Self>,
}

impl Button {
	pub fn new() -> Self {
		Self {
			normal: Color::WHITE,
			hovered: Color::GREEN,
			focused: Color::BLACK,

			on_pressed: EventHandler::None,
		}
	}

	pub fn on_pressed(mut self, on_pressed: impl FnMut(&WidgetContainer<Self>) + 'static) -> Self {
		self.on_pressed = EventHandler::Closure(Box::new(on_pressed));
		self
	}
}

pub type ButtonSlot = PanelSlot;

impl Widget for Button {
	type Slot = ButtonSlot;
	const MAX_SLOTS: Option<usize> = Some(1);
	const DEFAULT_VISIBILITY: Visibility = Visibility::Visible;

	fn update_layout(container: &WidgetContainer<Self>) {
		let layout = container.layout.unwrap();
		if let Some(slot) = &container.slots.get(0) {
			let mut child = slot.child().borrow_mut();
			let desired = child.desired_size();

			let local_bounds = rect_in_rect(
				Rect::from_min_max(Vec2::ZERO, layout.local_bounds.size()),
				desired,
				slot.alignment,
			);

			child.update_layout(Layout::new(layout, local_bounds));
		}
	}

	fn desired_size(container: &WidgetContainer<Self>) -> Vec2 {
		if let Some(slot) = &container.slots.get(0) {
			let child = slot.child.borrow();
			child.desired_size() + slot.padding.size() + slot.margin.size()
		} else {
			Vec2::ZERO
		}
	}

	fn paint(container: &WidgetContainer<Self>, painter: &mut Painter) {
		let color = if container.focused {
			container.widget.focused
		} else if container.hovered {
			container.widget.hovered
		} else {
			container.widget.normal
		};
		painter.fill_rect(container.layout.unwrap().absolute_bounds(), color);

		if let Some(slot) = &container.slots.get(0) {
			let child = slot.child().borrow();
			child.paint(painter);
		}
	}

	fn handle_event(container: &mut WidgetContainer<Self>, event: Event) -> Reply {
		match event {
			Event::Button { input, pressed } => {
				if input == MOUSE_BUTTON_LEFT {
					if pressed {
						Reply::Focus
					} else {
						let mut handler =
							std::mem::replace(&mut container.widget.on_pressed, EventHandler::None);
						#[allow(clippy::single_match)]
						match &mut handler {
							EventHandler::Closure(x) => (x)(container),
							_ => {}
						}
						let _ = std::mem::replace(&mut container.widget.on_pressed, handler);
						Reply::Unfocus
					}
				} else {
					Reply::Unfocus
				}
			}
			_ => Reply::None,
		}
	}
}

#[derive(Default, Debug)]
pub struct Panel {
	color: Color,
}

impl Panel {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn color(mut self, color: impl Into<Color>) -> Self {
		self.color = color.into();
		self
	}
}

impl Widget for Panel {
	type Slot = PanelSlot;
	const MAX_SLOTS: Option<usize> = Some(1);
	const DEFAULT_VISIBILITY: Visibility = Visibility::Visible;

	fn update_layout(container: &WidgetContainer<Self>) {
		let layout = container.layout.unwrap();
		if let Some(slot) = &container.slots.get(0) {
			let mut child = slot.child().borrow_mut();
			let desired = child.desired_size();

			let local_bounds = rect_in_rect(
				Rect::from_min_max(Vec2::ZERO, layout.local_bounds.size()),
				desired,
				slot.alignment,
			);

			child.update_layout(Layout::new(layout, local_bounds));
		}
	}

	fn desired_size(container: &WidgetContainer<Self>) -> Vec2 {
		if let Some(slot) = &container.slots.get(0) {
			let child = slot.child.borrow();
			child.desired_size() + slot.padding.size() + slot.margin.size()
		} else {
			Vec2::ZERO
		}
	}

	fn paint(container: &WidgetContainer<Self>, painter: &mut Painter) {
		painter.fill_rect(
			container.layout.unwrap().absolute_bounds(),
			container.widget.color,
		);

		if let Some(slot) = &container.slots.get(0) {
			let child = slot.child().borrow();
			child.paint(painter);
		}
	}
}

#[derive(Debug)]
pub struct PanelSlot {
	pub child: WidgetHandle,

	pub alignment: Alignment2,
	pub margin: Margin,
	pub padding: Padding,
}

impl PanelSlot {
	pub fn alignment(&mut self, alignment: Alignment2) -> &mut Self {
		self.alignment = alignment;
		self
	}

	pub fn margin(&mut self, margin: impl Into<Margin>) -> &mut Self {
		self.margin = margin.into();
		self
	}

	pub fn padding(&mut self, padding: impl Into<Padding>) -> &mut Self {
		self.padding = padding.into();
		self
	}
}

impl Slot for PanelSlot {
	fn new(child: WidgetHandle) -> Self {
		Self {
			child,

			alignment: Alignment2::default(),
			margin: Margin::default(),
			padding: Padding::default(),
		}
	}

	fn child(&self) -> &WidgetHandle {
		&self.child
	}

	fn child_mut(&mut self) -> &mut WidgetHandle {
		&mut self.child
	}
}

#[derive(Debug)]
pub struct BoxSlot {
	pub child: WidgetHandle,

	pub alignment: Alignment2,
	pub margin: Margin,
	pub padding: Padding,
	pub sizing: Sizing,
}

impl BoxSlot {
	pub fn alignment(&mut self, alignment: Alignment2) -> &mut Self {
		self.alignment = alignment;
		self
	}

	pub fn margin(&mut self, margin: impl Into<Margin>) -> &mut Self {
		self.margin = margin.into();
		self
	}

	pub fn padding(&mut self, padding: impl Into<Padding>) -> &mut Self {
		self.padding = padding.into();
		self
	}

	pub fn sizing(&mut self, sizing: Sizing) -> &mut Self {
		self.sizing = sizing;
		self
	}
}

impl Slot for BoxSlot {
	fn new(child: WidgetHandle) -> Self {
		Self {
			child,

			alignment: Alignment2::default(),
			margin: Margin::default(),
			padding: Padding::default(),
			sizing: Sizing::default(),
		}
	}

	fn child(&self) -> &WidgetHandle {
		&self.child
	}

	fn child_mut(&mut self) -> &mut WidgetHandle {
		&mut self.child
	}
}

#[derive(Debug)]
pub struct VerticalBox;
impl Widget for VerticalBox {
	type Slot = BoxSlot;
	const DEFAULT_VISIBILITY: Visibility = Visibility::HitTestInvisible;

	fn update_layout(container: &WidgetContainer<Self>) {
		let layout = container.layout.unwrap();
		let mut y = 0.0;
		for slot in container.slots.iter() {
			let mut child = slot.child().borrow_mut();
			let desired = child.desired_size();

			let x0 = slot.margin.left;
			let x1 = layout.local_bounds.size().x - slot.margin.right;
			y += slot.margin.bottom;
			let y0 = y;
			y += desired.y;
			let y1 = y;
			y += slot.margin.top;
			let available = Rect::from((x0, y0, x1, y1));
			let local_bounds = rect_in_rect(available, desired, slot.alignment);

			child.update_layout(Layout::new(layout, local_bounds));
		}
	}

	fn desired_size(container: &WidgetContainer<Self>) -> Vec2 {
		let mut size = Vec2::ZERO;

		for slot in container.slots.iter() {
			let child = slot.child.borrow();
			let desired = child.desired_size() + slot.margin.size() + slot.padding.size();
			size.y += desired.y;
			if size.x < desired.x {
				size.x = desired.x;
			}
		}

		size
	}

	fn paint(container: &WidgetContainer<Self>, painter: &mut Painter) {
		for slot in container.slots.iter() {
			let child = slot.child().borrow();
			child.paint(painter);
		}
	}
}

#[derive(Debug)]
pub struct HorizontalBox;
impl Widget for HorizontalBox {
	type Slot = BoxSlot;
	const DEFAULT_VISIBILITY: Visibility = Visibility::HitTestInvisible;

	fn update_layout(container: &WidgetContainer<Self>) {
		let layout = container.layout.unwrap();
		let mut x = 0.0;
		for slot in container.slots.iter() {
			let mut child = slot.child().borrow_mut();
			let desired = child.desired_size();

			x += slot.margin.left;
			let x0 = x;
			x += desired.x;
			let x1 = x;
			x += slot.margin.right;
			let y0 = slot.margin.bottom;
			let y1 = layout.local_bounds.size().y - slot.margin.top;
			let available = Rect::from((x0, y0, x1, y1));
			let local_bounds = rect_in_rect(available, desired, slot.alignment);

			child.update_layout(Layout::new(layout, local_bounds));
		}
	}

	fn desired_size(container: &WidgetContainer<Self>) -> Vec2 {
		let mut size = Vec2::ZERO;

		for slot in container.slots.iter() {
			let child = slot.child.borrow();
			let desired = child.desired_size() + slot.margin.size() + slot.padding.size();
			size.x += desired.x;
			if size.y < desired.y {
				size.y = desired.y;
			}
		}

		size
	}

	fn paint(container: &WidgetContainer<Self>, painter: &mut Painter) {
		for slot in container.slots.iter() {
			let child = slot.child().borrow();
			child.paint(painter);
		}
	}
}

#[derive(Debug)]
// TODO: Anchors
// TODO: Sizing
pub struct CanvasSlot {
	pub child: WidgetHandle,
}

impl Slot for CanvasSlot {
	fn new(child: WidgetHandle) -> Self {
		Self { child }
	}

	fn child(&self) -> &WidgetHandle {
		&self.child
	}

	fn child_mut(&mut self) -> &mut WidgetHandle {
		&mut self.child
	}
}

#[derive(Debug)]
pub struct Canvas;
impl Widget for Canvas {
	type Slot = CanvasSlot;
	const DEFAULT_VISIBILITY: Visibility = Visibility::HitTestInvisible;

	fn update_layout(container: &WidgetContainer<Self>) {
		let layout = container.layout.unwrap();
		for slot in container.slots.iter() {
			let mut child = slot.child().borrow_mut();
			let desired = child.desired_size();
			let local_bounds = rect_in_rect(
				Rect::from_min_max(Vec2::ZERO, layout.local_bounds.size()),
				desired,
				Alignment2::FILL_FILL,
			);

			child.update_layout(Layout::new(layout, local_bounds));
		}
	}

	fn desired_size(_container: &WidgetContainer<Self>) -> Vec2 {
		Vec2::ZERO
	}

	fn paint(container: &WidgetContainer<Self>, painter: &mut Painter) {
		for slot in container.slots.iter() {
			let child = slot.child().borrow();
			child.paint(painter);
		}
	}
}
