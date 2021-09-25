use crate::GRUVBOX;
use asset::AssetRef;
use graphics::FontCollection;
use math::{
	Color,
	Vector2,
};

#[derive(Clone)]
pub struct Style {
	pub font_collection: AssetRef<FontCollection>,
	pub text_size: u32,
	pub header_size: u32,

	pub margin: Vector2,
	pub button_padding: Vector2,

	pub theme: Theme,
}

impl Style {
	pub fn new() -> Self {
		Self::default()
	}

	pub fn text_size(&self, text: &str, wrap: Option<f32>) -> Vector2 {
		let font = self
			.font_collection
			.font_at_size(self.text_size, 1.0)
			.unwrap(); // NOTE: I don't think DPI matters here

		font.string_rect(text, wrap.unwrap_or(f32::INFINITY)).size()
	}
}

impl Default for Style {
	fn default() -> Self {
		Self {
			font_collection: AssetRef::new("{cdb5cd33-004d-4518-ab20-93475b735cfa}").unwrap(),
			text_size: 12,
			header_size: 16,

			margin: [4.0, 4.0].into(),
			button_padding: [10.0, 10.0].into(),

			theme: Theme {
				window_background: GRUVBOX.bg_s,
				text: GRUVBOX.fg,

				unhovered_button: GRUVBOX.bg0,
				hovered_button: GRUVBOX.bg1,
				focused_button: GRUVBOX.bg2,
			},
		}
	}
}

#[derive(Clone)]
pub struct Theme {
	pub window_background: Color,
	pub text: Color,

	pub unhovered_button: Color,
	pub hovered_button: Color,
	pub focused_button: Color,
}
