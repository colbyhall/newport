mod components;

use {
	components::*,
	ecs::*,
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	gpu::*,
	graphics::*,
	math::*,
	resources::*,
};

struct Game {
	schedule: Schedule,
	world: World,

	to_show: Option<Texture>,

	style: PainterStyle,
	draw_pipeline: Handle<GraphicsPipeline>,
}

impl Module for Game {
	fn new() -> Self {
		let mut style = PainterStyle::default();
		style.line_width = 20.0;

		let schedule = Schedule::builder().spawn();
		let world = World::default();

		Self {
			schedule,
			world,

			to_show: None,

			style,
			draw_pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}")
				.unwrap_or_default(),
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		register_components(builder)
			.module::<Graphics>()
			.module::<editor::Editor>()
			.tick(|dt| {
				let game: &Game = Engine::module().unwrap();
				Engine::wait_on(game.schedule.execute(&game.world, dt));
			})
	}
}

define_run_module!(Game, "Game Example");
