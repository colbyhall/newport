use asset::{
	AssetManager,
	AssetRef,
};
use engine::{
	Builder,
	Engine,
	Module,
};
use gpu::{
	Gpu,
	GraphicsPipeline,
	GraphicsRecorder,
	Layout,
};
use graphics::Graphics;
use imgui::{
	Context,
	DrawState,
	LayoutStyle,
	Panel,
	RawInput,
	TextStyle,
};

pub struct Editor {
	imgui: Context,
	draw_state: DrawState,
	input: Option<RawInput>,

	present_pipeline: AssetRef<GraphicsPipeline>,
}

impl Module for Editor {
	fn new() -> Self {
		Self {
			imgui: Context::new(),
			draw_state: DrawState::new(),
			input: None,

			present_pipeline: AssetRef::new("{62b4ffa0-9510-4818-a6f2-7645ec304d8e}").unwrap(),
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

				if editor.input.is_none() {
					editor.input = Some(RawInput::default());
				}

				editor.input.as_mut().unwrap().dt = dt;
			})
			.display(|| {
				let device = Gpu::device();
				let backbuffer = device
					.acquire_backbuffer()
					.expect("Swapchain failed to find a back buffer");

				let editor: &mut Editor = unsafe { Engine::module_mut().unwrap() };
				let Editor {
					imgui,
					draw_state,
					input,
					present_pipeline,
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

					input.dpi = Engine::window().unwrap().scale_factor() as f32;

					imgui.begin_frame(input);
					let layout_style: LayoutStyle = imgui.style().get();
					let text_style: TextStyle = imgui.style().get();

					let height = text_style.label_height()
						+ layout_style.padding.min.y
						+ layout_style.padding.max.y;
					Panel::top("menu_bar", height).build(imgui, |builder| {
						builder.button("Position").clicked();

						builder.button("Rotation").clicked();
					});
					imgui.end_frame()
				};

				let (gfx, imgui) = draw_state.record(canvas, GraphicsRecorder::new(), imgui);
				let imgui = imgui.unwrap();

				let receipt = gfx
					.render_pass(&[&backbuffer], |ctx| {
						ctx.bind_pipeline(present_pipeline)
							.bind_texture("texture", &imgui)
							.draw(3, 0)
					})
					.resource_barrier_texture(&backbuffer, Layout::ColorAttachment, Layout::Present)
					.submit();

				device.display(&[receipt]);
			})
	}
}
