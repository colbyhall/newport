use crate::*;

use engine::Engine;

pub struct GraphicsCommandBuffer(pub(crate) api::GraphicsCommandBuffer);

impl GraphicsCommandBuffer {
	pub fn submit(self) -> Receipt {
		let device = self.0.owner.clone();
		device.submit_graphics(vec![self.0], &[])
	}

	pub fn submit_but_wait_on(self, receipts: &[Receipt]) -> Receipt {
		let device = self.0.owner.clone();
		device.submit_graphics(vec![self.0], receipts)
	}
}

pub struct GraphicsRecorder(pub(crate) api::GraphicsCommandBuffer);

impl GraphicsRecorder {
	pub fn new() -> Self {
		let engine = Engine::as_ref();
		let gpu: &Gpu = engine
			.module()
			.expect("Engine must depend on Gpu module if no device is provided.");
		Self::new_in(gpu.device())
	}

	pub fn new_in(device: &Device) -> Self {
		let mut inner = api::GraphicsCommandBuffer::new(device.0.clone()).unwrap();
		inner.begin();
		GraphicsRecorder(inner)
	}

	pub fn resource_barrier_texture(
		mut self,
		texture: &Texture,
		old_layout: Layout,
		new_layout: Layout,
	) -> Self {
		self.0
			.resource_barrier_texture(texture.0.clone(), old_layout, new_layout);
		self
	}

	pub fn copy_buffer_to_texture<T: Sized>(mut self, dst: &Texture, src: &Buffer<T>) -> Self {
		self.0
			.copy_buffer_to_texture(dst.0.clone(), src.api.clone());
		self
	}

	pub fn copy_buffer_to_buffer<T: Sized>(mut self, dst: &Buffer<T>, src: &Buffer<T>) -> Self {
		self.0
			.copy_buffer_to_buffer(dst.api.clone(), src.api.clone());
		self
	}

	pub fn render_pass(
		mut self,
		attachments: &[&Texture],
		pass: impl FnOnce(RenderPassRecorder) -> RenderPassRecorder,
	) -> Self {
		let mut a = Vec::with_capacity(attachments.len());
		attachments.iter().for_each(|e| a.push(e.0.clone()));

		self.0.begin_render_pass(&a[..]).unwrap();
		let mut recorder = pass(RenderPassRecorder(self)).0;
		recorder.0.end_render_pass();

		recorder
	}

	pub fn finish(mut self) -> GraphicsCommandBuffer {
		self.0.end();
		GraphicsCommandBuffer(self.0)
	}
}

pub struct RenderPassRecorder(GraphicsRecorder);

impl RenderPassRecorder {
	pub fn clear_color(mut self, color: impl Into<Color>) -> Self {
		self.0 .0.clear_color(color.into());
		self
	}

	pub fn clear_depth(mut self, depth: f32) -> Self {
		self.0 .0.clear_depth(depth);
		self
	}

	pub fn bind_pipeline(mut self, pipeline: &GraphicsPipeline) -> Self {
		self.0 .0.bind_pipeline(pipeline.0.clone());
		self
	}

	pub fn bind_scissor(mut self, scissor: Option<Rect>) -> Self {
		self.0 .0.bind_scissor(scissor);
		self
	}

	pub fn bind_vertex_buffer<T: Sized>(mut self, buffer: &Buffer<T>) -> Self {
		self.0 .0.bind_vertex_buffer(buffer.api.clone());
		self
	}

	pub fn bind_index_buffer<T: Sized>(mut self, buffer: &Buffer<T>) -> Self {
		self.0 .0.bind_index_buffer(buffer.api.clone());
		self
	}

	pub fn draw(mut self, vertex_count: usize, first_vertex: usize) -> Self {
		self.0 .0.draw(vertex_count, first_vertex);
		self
	}

	pub fn draw_indexed(mut self, index_count: usize, first_index: usize) -> Self {
		self.0 .0.draw_indexed(index_count, first_index);
		self
	}

	pub fn bind_constants<T: Sized>(
		mut self,
		name: &str,
		buffer: &Buffer<T>,
		index: usize,
	) -> Self {
		self.0 .0.bind_constants(name, buffer.api.clone(), index);
		self
	}

	pub fn bind_texture(mut self, name: &str, texture: &Texture) -> Self {
		self.0 .0.bind_texture(name, texture.0.clone());
		self
	}
}
