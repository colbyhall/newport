pub(crate) use {
	newport_asset as asset,
	newport_ecs as ecs,
	newport_engine as engine,
	newport_gpu as gpu,
	newport_graphics as graphics,
	newport_math as math,
};

mod components;
mod game;
mod game_state;
mod render_state;

#[cfg(feature = "editor")]
mod editor;

pub use {
	components::*,
	game::*,
	game_state::*,
	render_state::*,
};
