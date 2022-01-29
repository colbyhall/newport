use {
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	gpu::{
		Buffer,
		BufferUsage,
		Constant,
		Format,
		Gpu,
		GraphicsPipeline,
		GraphicsRecorder,
		Layout,
		MemoryType,
		Shader,
		ShaderVariant,
	},
	math::{
		Color,
		Vec2,
	},
};

#[allow(dead_code)]
struct Vertex {
	position: Vec2,
	color: Color,
}

struct Triangle {
	pipeline: GraphicsPipeline,
	vertices: Buffer<Vertex>,
}

static SHADER: &str = "
	struct PSInput {
		float4 position : SV_POSITION;
		float4 color : COLOR;
	};

	PSInput VSMain(float2 position : POSITION, float4 color: COLOR) {
		PSInput result;
		result.position = float4(position.x, position.y, 0.0f, 1.0f);
		result.color = color;
		return result;
	}
	
	float4 PSMain( PSInput input ) : SV_TARGET {
		return input.color;
	}
";

static VERTICES: &[Vertex] = &[
	Vertex {
		position: Vec2::new(0.0, 0.5),
		color: Color::RED,
	},
	Vertex {
		position: Vec2::new(0.5, -0.5),
		color: Color::GREEN,
	},
	Vertex {
		position: Vec2::new(-0.5, -0.5),
		color: Color::BLUE,
	},
];

impl Module for Triangle {
	fn new() -> Self {
		let binary = gpu::compile("vertex.hlsl", SHADER, "VSMain", ShaderVariant::Vertex).unwrap();
		let vertex_shader = Shader::builder(&binary, ShaderVariant::Vertex)
			.main("VSMain")
			.spawn()
			.unwrap();

		let binary = gpu::compile("pixel.hlsl", SHADER, "PSMain", ShaderVariant::Pixel).unwrap();
		let pixel_shader = Shader::builder(&binary, ShaderVariant::Pixel)
			.main("PSMain")
			.spawn()
			.unwrap();

		let pipeline = GraphicsPipeline::builder()
			.attachments(&[Format::BGR_U8_SRGB])
			.vertex_attributes(&[Constant::Vector2, Constant::Color])
			.shaders(&[vertex_shader, pixel_shader])
			.spawn()
			.unwrap();

		let vertices =
			Buffer::new(BufferUsage::VERTEX, MemoryType::HostVisible, VERTICES.len()).unwrap();
		vertices.copy_to(VERTICES).unwrap();

		Self { pipeline, vertices }
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.module::<Gpu>().display(|| {
			let triangle: &Triangle = Engine::module().unwrap();
			let device = Gpu::device();
			let backbuffer = device.acquire_backbuffer().unwrap();

			let receipt = GraphicsRecorder::new()
				.render_pass(&[&backbuffer], |ctx| {
					ctx.clear_color(Color::BLACK)
						.set_pipeline(&triangle.pipeline)
						.set_vertex_buffer(&triangle.vertices)
						.draw(VERTICES.len(), 0)
				})
				.texture_barrier(&backbuffer, Layout::ColorAttachment, Layout::Present)
				.finish()
				.submit();

			device.display(&[receipt]);
		})
	}
}

define_run_module!(Triangle, "Triangle");
