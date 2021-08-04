use crate::{
	Viewport,
	ViewportId,
};

use gpu::Buffer;
use gpu::{
	Gpu,
	RenderPassRecorder,
};
use graphics::Vertex;

use engine::Engine;
use math::Matrix4;

use std::collections::HashMap;

pub struct RenderState {
	pub viewports: HashMap<ViewportId, Viewport>,

	pub primitives: Vec<Box<dyn Primitive>>, // TODO: These should all be built into some bump allocator
	pub primitive_transforms: Vec<Matrix4>,
}

impl RenderState {
	pub fn render(&self) -> HashMap<ViewportId, gpu::Texture> {
		let engine = Engine::as_ref();
		let gpu: &Gpu = engine.module().unwrap();
		let device = gpu.device();

		let transform_buffer = device
			.create_buffer(
				gpu::BufferUsage::CONSTANTS,
				gpu::MemoryType::HostVisible,
				self.primitive_transforms.len(),
			)
			.unwrap();
		transform_buffer.copy_to(&self.primitive_transforms[..]);

		let view_matrices: Vec<Matrix4> = self
			.viewports
			.iter()
			.map(|(_id, viewport)| {
				let aspect_ratio = viewport.width as f32 / viewport.height as f32;
				let projection = Matrix4::perspective(viewport.fov, aspect_ratio, 10000.0, 0.1);
				// TODO: View matrix
				projection
			})
			.collect();

		let views_buffer = device
			.create_buffer(
				gpu::BufferUsage::CONSTANTS,
				gpu::MemoryType::HostVisible,
				view_matrices.len(),
			)
			.unwrap();
		views_buffer.copy_to(&view_matrices[..]);

		let mut result = HashMap::with_capacity(self.viewports.len());
		self.viewports.iter().for_each(|(id, viewport)| {
			let backbuffer = device
				.create_texture(
					gpu::TextureUsage::SAMPLED | gpu::TextureUsage::COLOR_ATTACHMENT,
					gpu::MemoryType::DeviceLocal,
					gpu::Format::RGBA_U8,
					viewport.width,
					viewport.height,
					1,
				)
				.unwrap();

			result.insert(*id, backbuffer);
		});

		result
	}
}

pub trait Primitive {
	fn record(&self, recorder: RenderPassRecorder) -> RenderPassRecorder;
}

pub struct MeshRenderPrimitive {
	pub vertex_buffer: Buffer<Vertex>,
	pub index_buffer: Buffer<u32>,
}

impl Primitive for MeshRenderPrimitive {
	fn record(&self, recorder: RenderPassRecorder) -> RenderPassRecorder {
		recorder
	}
}
