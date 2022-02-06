use {
	config::{
		Config,
		ConfigManager,
	},

	engine::{
		Builder,
		Engine,
		Module,
	},
	gui::*,
	math::Color,
	serde::{
		Deserialize,
		Serialize,
	},
};

pub const CONFIG_FILE: &str = "editor.toml";

#[derive(Serialize, Deserialize)]
pub struct Style {
	background: Color,
}

impl Config for Style {
	const FILE: &'static str = CONFIG_FILE;
}

impl Default for Style {
	fn default() -> Self {
		Self {
			background: Color::from_srgb(0x282828FF),
		}
	}
}

pub struct Editor;
impl Module for Editor {
	fn new() -> Self {
		let style = ConfigManager::read::<Style>();

		let gui: &mut Gui = Engine::module_mut_checked().unwrap();
		let mut canvas = gui.canvas().borrow_mut();
		let canvas: &mut WidgetContainer<Canvas> = canvas.as_any_mut().downcast_mut().unwrap();
		canvas.slot_with(Panel::new().color(style.background), |gui| {
			gui.slot_with(VerticalBox, |gui| {});
		});

		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Gui>()
			.module::<ConfigManager>()
			.register(Style::variant())
	}
}
