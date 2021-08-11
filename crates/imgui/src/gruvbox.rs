use math::Color;

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
		bg: Color::from_srgb(0x282828ff),
		fg: Color::from_srgb(0xebdbb2ff),

		bg_h: Color::from_srgb(0x1d2021ff),
		bg_s: Color::from_srgb(0x32302fff),

		bg0: Color::from_srgb(0x282828ff),
		bg1: Color::from_srgb(0x3c3836ff),
		bg2: Color::from_srgb(0x504945ff),
		bg3: Color::from_srgb(0x665c54ff),
		bg4: Color::from_srgb(0x7c6f64ff),

		fg0: Color::from_srgb(0xfbf1c7ff),
		fg1: Color::from_srgb(0xebdbb2ff),
		fg2: Color::from_srgb(0xd5c4a1ff),
		fg3: Color::from_srgb(0xbdae93ff),
		fg4: Color::from_srgb(0xa89984ff),

		red0: Color::from_srgb(0xcc241dff),
		green0: Color::from_srgb(0x98971aff),
		yellow0: Color::from_srgb(0xd79921ff),
		blue0: Color::from_srgb(0x458588ff),
		purple0: Color::from_srgb(0xb16286ff),
		aqua0: Color::from_srgb(0x689d6aff),
		gray0: Color::from_srgb(0xa89984ff),
		orange0: Color::from_srgb(0xd65d0eff),

		red1: Color::from_srgb(0xfb4934ff),
		green1: Color::from_srgb(0xb8bb26ff),
		yellow1: Color::from_srgb(0xfabd2fff),
		blue1: Color::from_srgb(0x83a598ff),
		purple1: Color::from_srgb(0xd3869bff),
		aqua1: Color::from_srgb(0x8ec07cff),
		gray1: Color::from_srgb(0x928375ff),
		orange1: Color::from_srgb(0xfe8019ff),
	};
}
