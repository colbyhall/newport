use crate::math::Color;

use lazy_static::lazy_static;

pub struct Palette {
	pub bg: Color,
	pub fg: Color,

	pub bg_h: Color,
	pub bg_s: Color,

	pub bg0: Color,
	pub bg1: Color,
	pub bg2: Color,
	pub bg3: Color,
	pub bg4: Color,

	pub fg0: Color,
	pub fg1: Color,
	pub fg2: Color,
	pub fg3: Color,
	pub fg4: Color,

	pub red0: Color,
	pub green0: Color,
	pub yellow0: Color,
	pub blue0: Color,
	pub purple0: Color,
	pub aqua0: Color,
	pub gray0: Color,
	pub orange0: Color,

	pub red1: Color,
	pub green1: Color,
	pub yellow1: Color,
	pub blue1: Color,
	pub purple1: Color,
	pub aqua1: Color,
	pub gray1: Color,
	pub orange1: Color,
}

// TODO: SRGB -> LINEAR
lazy_static! {
	pub static ref DARK: Palette = Palette {
		bg: Color::from_srgb(0x282828FF),
		fg: Color::from_srgb(0xebdbb2FF),

		bg_h: Color::from_srgb(0x1d2021FF),
		bg_s: Color::from_srgb(0x32302fFF),

		bg0: Color::from_srgb(0x282828FF),
		bg1: Color::from_srgb(0x3c3836FF),
		bg2: Color::from_srgb(0x504945FF),
		bg3: Color::from_srgb(0x665c54FF),
		bg4: Color::from_srgb(0x7c6f64FF),

		fg0: Color::from_srgb(0xfbf1c7FF),
		fg1: Color::from_srgb(0xebdbb2FF),
		fg2: Color::from_srgb(0xd5c4a1FF),
		fg3: Color::from_srgb(0xbdae93FF),
		fg4: Color::from_srgb(0xa89984FF),

		red0: Color::from_srgb(0xcc241dFF),
		green0: Color::from_srgb(0x98971aFF),
		yellow0: Color::from_srgb(0xd79921FF),
		blue0: Color::from_srgb(0x458588FF),
		purple0: Color::from_srgb(0xb16286FF),
		aqua0: Color::from_srgb(0x689d6aFF),
		gray0: Color::from_srgb(0xa89984FF),
		orange0: Color::from_srgb(0xd65d0eFF),

		red1: Color::from_srgb(0xfb4934FF),
		green1: Color::from_srgb(0xb8bb26FF),
		yellow1: Color::from_srgb(0xfabd2fFF),
		blue1: Color::from_srgb(0x83a598FF),
		purple1: Color::from_srgb(0xd3869bFF),
		aqua1: Color::from_srgb(0x8ec07cFF),
		gray1: Color::from_srgb(0x928375FF),
		orange1: Color::from_srgb(0xfe8019FF),
	};
}
