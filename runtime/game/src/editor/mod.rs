use edgui::{
	Button,
	Context,
	DrawState,
	Label,
	Panel,
	RawInput,
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

pub struct Editor {
	context: Context,
	draw_state: DrawState,
	input: Option<RawInput>,

	present_pipeline: AssetRef<GraphicsPipeline>,

	dt: Option<f32>,
}

impl Module for Editor {
	fn new() -> Self {
		Self {
			context: Context::new(),
			draw_state: DrawState::new(),
			input: None,

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

				let canvas = {
					let context = &mut editor.context;

					let mut input = editor.input.take().unwrap_or_default();

					input.viewport = (
						0.0,
						0.0,
						backbuffer.width() as f32,
						backbuffer.height() as f32,
					)
						.into();

					let dt = editor.dt.take().unwrap_or_default();

					input.dt = dt;
					input.dpi = Engine::window().unwrap().scale_factor() as f32;

					context.begin_frame(input);
					Panel::top("menu_bar", 50.0).build(context, |gui| {
						gui.add(Label::new("Testing 123"));
						gui.add(Label::new("Testing 123"));
						gui.add(Label::new("Testing 123"));
						gui.add(Label::new("Testing 123"));
						gui.add(Label::new("Testing 123"));

						if gui.add(Button::new("Hello World")).clicked() {
							info!("Hello World");
						}
						if gui.add(Button::new("Hello World2")).clicked() {
							info!("Hello World");
						}
					});
					Panel::bottom("context_bar", 24.0).build(context, |gui| {
						gui.add(Label::new(format!(
							"{:.2}ms/{}fps",
							dt * 1000.0,
							Engine::fps()
						)));
					});
					context.end_frame()
				};

				let gfx = GraphicsRecorder::new();
				let (gfx, imgui) = editor.draw_state.record(canvas, gfx, &editor.context);
				let imgui = imgui.unwrap();

				let receipt = gfx
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
							.bind_pipeline(&editor.present_pipeline)
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
