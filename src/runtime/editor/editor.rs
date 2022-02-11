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
#[serde(default)]
pub struct Style {
	background: Color,
	background_s: Color,
	background_h: Color,

	foreground: Color,
}

impl Config for Style {
	const FILE: &'static str = CONFIG_FILE;
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
		let style = ConfigManager::read::<Style>();

		let gui: &mut Gui = Engine::module_mut_checked().unwrap();
		let mut canvas = gui.canvas().borrow_mut();
		let canvas: &mut WidgetContainer<Canvas> = canvas.as_any_mut().downcast_mut().unwrap();
		canvas.slot_with(Panel::new().color(style.background), |gui| {
			gui.slot_with(VerticalBox, |gui| {
				gui.slot_with(Panel::new().color(style.background_h), |gui| {
					gui.slot_with(HorizontalBox, |gui| {
						gui.slot_with(
							Button::new().on_pressed(|_| println!("Hello World!")),
							|gui| {
								gui.slot(Text::new("Foo Bar").color(style.foreground))
									.margin(5.0);
							},
						);
					})
					.alignment(Alignment2::CENTER_FILL);
				})
				.alignment(Alignment2::CENTER_FILL);
			})
			.alignment(Alignment2::FILL_FILL);
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
