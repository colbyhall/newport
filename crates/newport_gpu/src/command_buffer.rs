use crate::*;

pub struct GraphicsCommandBuffer(pub(crate) api::GraphicsCommandBuffer);

pub struct GraphicsRecorder(pub(crate) api::GraphicsCommandBuffer);

impl GraphicsRecorder {
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
		render_pass: &RenderPass,
		attachments: &[&Texture],
		pass: impl FnOnce(RenderPassRecorder) -> RenderPassRecorder,
	) -> Self {
		let mut a = Vec::with_capacity(attachments.len());
		attachments.iter().for_each(|e| a.push(e.0.clone()));

		self.0.begin_render_pass(render_pass.0.clone(), &a[..]);
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
	pub fn clear(mut self, color: impl Into<Color>) -> Self {
		self.0 .0.clear(color.into());
		self
	}

	pub fn bind_pipeline(mut self, pipeline: &'a GraphicsPipeline) -> Self {
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
