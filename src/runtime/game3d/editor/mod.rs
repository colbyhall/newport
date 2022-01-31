use ecs::ReadStorage;

use {
	crate::Game,
	ecs::Entity,
	ecs::Named,
	ecs::Query,
	egui::{
		CentralPanel,
		Egui,
		EguiScope,
		SidePanel,
		TopBottomPanel,
	},

	engine::{
		Builder,
		Engine,
		Module,
	},
	math::*,
};

pub struct Editor {
	dt: f32,
	selected_entity: Option<Entity>,
}
impl Module for Editor {
	fn new() -> Self {
		Self {
			dt: 0.0,
			selected_entity: None,
		}
	}
	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Egui>()
			.tick(|dt| {
				let editor: &mut Editor = unsafe { Engine::module_mut().unwrap() };
				editor.dt = dt;
			})
			.register(EguiScope::new(|ctx| {
				let game: &Game = Engine::module().unwrap();
				let Editor {
					dt,
					selected_entity,
					..
				} = unsafe { Engine::module_mut().unwrap() };

				TopBottomPanel::bottom("context_bar").show(ctx, |ui| {
					ui.with_layout(egui::Layout::left_to_right(), |ui| {
						ui.label(Engine::name());
						ui.with_layout(egui::Layout::right_to_left(), |ui| {
							ui.label(format!(
								"{} {}",
								engine::ENGINE_NAME,
								engine::ENGINE_VERSION
							));
							ui.separator();
							ui.label(format!("{} FPS ({:.2}ms)", Engine::fps(), *dt * 1000.0))
						});
					});
				});

				SidePanel::right("details").show(ctx, |ui| {
					egui::CollapsingHeader::new("World Inspector").show(ui, |ui| {
						egui::ScrollArea::new([true, false]).show(ui, |ui| {
							let named: ReadStorage<Named> = game.world.read();
							let entities = Query::new().read(&named).execute(&game.world);
							for e in entities.iter().copied() {
								let name = named.get(e).unwrap();

								let selected = selected_entity.map(|f| f == e).unwrap_or_default();
								if ui.selectable_label(selected, &name.name).clicked() {
									*selected_entity = Some(e);
								}
							}
						});
					});
				});

				CentralPanel::default()
					.frame(egui::Frame::none())
					.show(ctx, |ui| {
						let space_available = ui.available_size();
						unsafe {
							let space_available = Vec2::new(space_available.x, space_available.y);
							let viewport = game.viewport.get();
							*viewport = space_available;
						}

						if let Some(display) = game.renderer.to_display() {
							let bindless = display.diffuse_buffer.bindless().unwrap_or_default();
							ui.image(egui::TextureId::User(bindless as u64), space_available)
						} else {
							ui.label("Viewport")
						}
					});
			}))
	}
}
