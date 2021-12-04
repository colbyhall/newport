use engine::{
	Builder,
	Engine,
	Module,
};

use egui::*;

pub struct Editor {
	dt: f32,
}

impl Module for Editor {
	fn new() -> Self {
		Self { dt: 0.0 }
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Egui>()
			.register(EguiScope::new(|ctx| {
				let editor = Engine::module::<Editor>().unwrap();

				TopBottomPanel::bottom("context_menu").show(ctx, |ui| {
					ui.horizontal(|ui| {
						ui.label(Engine::name());

						ui.with_layout(Layout::right_to_left(), |ui| {
							ui.label(format!(
								"{} {}",
								engine::ENGINE_NAME,
								engine::ENGINE_VERSION
							));
							ui.separator();
							ui.label(format!("{:.1}ms", editor.dt * 1000.0));
						});
					})
				});

				TopBottomPanel::top("page_bar").show(ctx, |ui| {
					menu::bar(ui, |ui| {
						menu::menu(ui, "File", |ui| {
							ui.add_space(3.0);

							if ui.button("New").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Open").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Save").clicked() {
								// …
							}

							if ui.button("Save As").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Exit").clicked() {
								// …
							}

							ui.add_space(3.0);
						});

						menu::menu(ui, "Edit", |ui| {
							ui.add_space(3.0);

							if ui.button("New").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Open").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Save").clicked() {
								// …
							}

							if ui.button("Save As").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Exit").clicked() {
								// …
							}

							ui.add_space(3.0);
						});

						menu::menu(ui, "Selection", |ui| {
							ui.add_space(3.0);

							if ui.button("New").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Open").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Save").clicked() {
								// …
							}

							if ui.button("Save As").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Exit").clicked() {
								// …
							}

							ui.add_space(3.0);
						});

						menu::menu(ui, "View", |ui| {
							ui.add_space(3.0);

							if ui.button("New").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Open").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Save").clicked() {
								// …
							}

							if ui.button("Save As").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Exit").clicked() {
								// …
							}

							ui.add_space(3.0);
						});

						menu::menu(ui, "yolo", |ui| {
							ui.add_space(3.0);

							if ui.button("New").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Open").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Save").clicked() {
								// …
							}

							if ui.button("Save As").clicked() {
								// …
							}

							ui.separator();

							if ui.button("Exit").clicked() {
								// …
							}

							menu::menu(ui, "Help", |ui| {
								ui.add_space(3.0);

								if ui.button("New").clicked() {
									// …
								}

								ui.separator();

								if ui.button("Open").clicked() {
									// …
								}

								ui.separator();

								if ui.button("Save").clicked() {
									// …
								}

								if ui.button("Save As").clicked() {
									// …
								}

								ui.separator();

								if ui.button("Exit").clicked() {
									// …
								}

								ui.add_space(3.0);
							});

							ui.add_space(3.0);
						});
					})
				});

				SidePanel::right("side")
					.resizable(true)
					.min_width(150.0)
					.show(ctx, |ui| {
						ui.label("hello world");
					});

				CentralPanel::default().show(ctx, |ui| {
					ui.label("This will be the viewport");
				});
			}))
			.tick(|dt| {
				let editor: &mut Editor = unsafe { Engine::module_mut().unwrap() };
				editor.dt = dt;
			})
	}
}
