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

#[derive(Serialize, Deserialize, Default)]
#[serde(default)]
pub struct EditorConfig {
	pub style: Style,
}

impl Config for EditorConfig {
	const NAME: &'static str = "Editor";
	const FILE: &'static str = CONFIG_FILE;
}

#[derive(Serialize, Deserialize)]
#[serde(default)]
pub struct Style {
	background: Color,
	background_s: Color,
	background_h: Color,

	foreground: Color,
}

impl Style {
	pub fn text(&self, text: impl ToString) -> Text {
		Text::new(text).color(self.foreground)
	}
}

impl Default for Style {
	fn default() -> Self {
		Self {
			background: Color::from_srgb(0x282828FF),
			background_s: Color::from_srgb(0x32303FFF),
			background_h: Color::from_srgb(0x1D2021FF),

			foreground: Color::from_srgb(0xEBDBB2FF),
		}
	}
}

pub struct Editor;
impl Module for Editor {
	fn new() -> Self {
		let config = ConfigManager::read::<EditorConfig>();

		let gui: &mut Gui = Engine::module_mut_checked().unwrap();
		let mut canvas = gui.canvas().borrow_mut();
		let canvas: &mut WidgetContainer<Canvas> = canvas.as_any_mut().downcast_mut().unwrap();
		canvas.slot_with(Panel::new().color(config.style.background), |gui| {
			gui.slot_with(VerticalBox, |gui| {
				gui.slot_with(Panel::new().color(config.style.background_h), |gui| {
					gui.slot_with(HorizontalBox, |gui| {
						gui.slot(config.style.text("Foo Bar")).margin(5.0);
					})
					.alignment(Alignment2::CENTER_FILL);
				})
				.alignment(Alignment2::CENTER_FILL);
			})
			.alignment(Alignment2::FILL_FILL);
		});

		Self
	}

	fn depends_on(builder: &mut Builder) -> &mut Builder {
		builder
			.module::<Gui>()
			.module::<ConfigManager>()
			.register(EditorConfig::variant())
	}
}
