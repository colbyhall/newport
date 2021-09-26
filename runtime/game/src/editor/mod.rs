use edgui::{
	Context,
	DrawState,
	Layout,
	Panel,
	RawInput,
	View,
};

use gpu::{
	Gpu,
	GraphicsPipeline,
	GraphicsRecorder,
};

use math::Color;

use engine::{
	info,
	Builder,
	Engine,
	Module,
};

use asset::{
	AssetManager,
	AssetRef,
};

use graphics::Graphics;

// use crate::Game;

pub struct Editor {
	context: Context,
	draw_state: DrawState,
	input: Option<RawInput>,

	view: View,

	present_pipeline: AssetRef<GraphicsPipeline>,

	dt: Option<f32>,
}

impl Module for Editor {
	fn new() -> Self {
		let view = View::new("whole", 1.0);
		Self {
			context: Context::new(),
			draw_state: DrawState::new(),
			input: None,

			view,

			present_pipeline: AssetRef::new("{62b4ffa0-9510-4818-a6f2-7645ec304d8e}").unwrap(),

			dt: None,
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<Graphics>()
			.module::<AssetManager>()
			.process_input(|event| {
				let editor: &mut Editor = unsafe { Engine::module_mut().unwrap() };

				if editor.input.is_none() {
					editor.input = Some(RawInput::default());
				}

				editor
					.input
					.as_mut()
					.unwrap()
					.events
					.push_back(event.clone());
			})
			.tick(|dt| {
				let editor: &mut Editor = unsafe { Engine::module_mut().unwrap() };
				editor.dt = Some(dt);
			})
			.display(|| {
				let device = Gpu::device();

				let backbuffer = device.acquire_backbuffer().unwrap();

				let editor: &mut Editor = unsafe { Engine::module_mut().unwrap() };
				let Editor {
					context,
					draw_state,
					view,
					present_pipeline,
					dt,
					input,
				} = editor;

				let canvas = {
					let mut input = input.take().unwrap_or_default();

					input.viewport = (
						0.0,
						0.0,
						backbuffer.width() as f32,
						backbuffer.height() as f32,
					)
						.into();

					let dt = dt.take().unwrap_or_default();

					input.dt = dt;
					input.dpi = Engine::window().unwrap().scale_factor() as f32;

					context.begin_frame(input);
					Panel::top("menu_bar", 50.0).build(context, |gui| {
						gui.horizontal(|gui| {
							gui.label("Your Mom");
							gui.label("Your Mom");
							gui.label("Your Mom");
							gui.label("Your Mom");

							gui.layout(Layout::right_to_left(), |gui| {
								if gui.button("This is a button").clicked() {
									info!("test123");
								}
							});
						});
					});
					Panel::bottom("context_bar", 24.0).build(context, |gui| {
						gui.layout(Layout::right_to_left(), |gui| {
							gui.label(format!("{:.2}ms/{}fps", dt * 1000.0, Engine::fps()));
						});
					});
					Panel::center("center").build(context, |gui| {
						// let game: &mut Game = unsafe { Engine::module_mut().unwrap() };

						// let bounds = gui.available_rect();
						// if let Some(scene) = game.frames.to_display() {
						// 	gui.painter().push_texture(
						// 		&scene.diffuse_buffer,
						// 		bounds,
						// 		Color::WHITE,
						// 		0.0,
						// 	);

						// 	game.viewport = bounds.size();
						// }

						view.add(gui);
					});
					context.end_frame()
				};

				let gfx = GraphicsRecorder::new();
				let (gfx, imgui) = draw_state.record(canvas, gfx, &editor.context);
				let imgui = imgui.unwrap();

				let receipt = gfx
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
							.bind_pipeline(present_pipeline)
							.bind_texture("texture", &imgui)
							.draw(3, 0)
					})
					.resource_barrier_texture(
						&backbuffer,
						gpu::Layout::ColorAttachment,
						gpu::Layout::Present,
					)
					.submit();

				device.display(&[receipt]);
			})
	}
}
