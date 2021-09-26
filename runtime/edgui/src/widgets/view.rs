use crate::*;

pub trait ViewTab: 'static {
	fn name(&self) -> String;

	fn add(&mut self, gui: &mut Gui);
}

enum Children {
	None,
	Views {
		views: Vec<View>,
		direction: Direction,
	},
	Tabs {
		tabs: Vec<Box<dyn ViewTab>>,
		selected: usize,
	},
}

pub struct View {
	_id: Id,
	children: Children,
	percent: f32,
}

impl View {
	pub fn new(id: impl ToId, percent: f32) -> Self {
		Self {
			_id: id.to_id(),
			children: Children::None,
			percent,
		}
	}

	pub fn new_views(id: impl ToId, percent: f32, views: Vec<View>, direction: Direction) -> Self {
		Self {
			_id: id.to_id(),
			children: Children::Views { views, direction },
			percent,
		}
	}

	pub fn add_tab(&mut self, tab: impl ViewTab + 'static) -> &mut Self {
		match &mut self.children {
			Children::None => {
				self.children = Children::Tabs {
					tabs: vec![Box::new(tab)],
					selected: 0,
				}
			}
			Children::Tabs { tabs, selected } => {
				tabs.push(Box::new(tab));
				*selected = tabs.len() - 1;
			}
			_ => unreachable!(),
		}

		self
	}

	pub fn add_view(&mut self, view: View) {
		match &mut self.children {
			Children::None => {
				self.children = Children::Views {
					views: vec![view],
					direction: Direction::LeftToRight,
				}
			}
			Children::Views { views, .. } => {
				views.push(view);
			}
			_ => unreachable!(),
		}
	}
}

impl View {
	pub fn add(&mut self, gui: &mut Gui) {
		match &mut self.children {
			Children::None => {
				gui.label("Empty View");
			}
			Children::Tabs { tabs, selected } => gui.horizontal(|gui| {
				for (index, it) in tabs.iter().enumerate() {
					if gui.button(it.name()).clicked() {
						*selected = index;
					}
				}
			}),
			_ => unimplemented!(),
		}
	}
}
