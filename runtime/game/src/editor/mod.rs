use edgui::{
	Context,
	DrawState,
	Label,
	LayoutStyle,
	Panel,
	RawInput,
	TextStyle,
};

use gpu::{
	Gpu,
	GraphicsPipeline,
	GraphicsRecorder,
};

use engine::{
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

	dt: f32,
}

impl Module for Editor {
	fn new() -> Self {
		Self {
			context: Context::new(),
			draw_state: DrawState::new(),
			input: None,

			present_pipeline: AssetRef::new("{62b4ffa0-9510-4818-a6f2-7645ec304d8e}").unwrap(),

			dt: 0.0,
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
				editor.dt = dt;
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

					input.dt = editor.dt;
					input.dpi = Engine::window().unwrap().scale_factor() as f32;

					context.begin_frame(input);
					let layout_style: LayoutStyle = context.style().get();
					let text_style: TextStyle = context.style().get();

					let height = text_style.label_height()
						+ layout_style.padding.min.y
						+ layout_style.padding.max.y;
					Panel::top("menu_bar", height).build(context, |gui| {
						gui.add(Label::new("Testing 123"));
					});
					context.end_frame()
				};

				let gfx = GraphicsRecorder::new();
				let (gfx, imgui) = editor.draw_state.record(canvas, gfx, &editor.context);
				let imgui = imgui.unwrap();

				let receipt = gfx
					.render_pass(&[&backbuffer], |ctx| {
						ctx.bind_pipeline(&editor.present_pipeline)
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
