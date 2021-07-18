use crate::*;

pub struct GraphicsCommandBuffer {
	pub(crate) api: api::GraphicsCommandBuffer,
}

pub struct GraphicsRecorder {
	pub(crate) api: api::GraphicsCommandBuffer,
}

impl GraphicsRecorder {
	pub fn resource_barrier_texture(
		mut self,
		texture: &Texture,
		old_layout: Layout,
		new_layout: Layout,
	) -> Self {
		self.api
			.resource_barrier_texture(texture.0.clone(), old_layout, new_layout);
		self
	}

	pub fn copy_buffer_to_texture<T: Sized>(mut self, dst: &Texture, src: &Buffer<T>) -> Self {
		self.api
			.copy_buffer_to_texture(dst.0.clone(), src.api.clone());
		self
	}

	pub fn copy_buffer_to_buffer<T: Sized>(mut self, dst: &Buffer<T>, src: &Buffer<T>) -> Self {
		self.api
			.copy_buffer_to_buffer(dst.api.clone(), src.api.clone());
		self
	}

	pub fn render_pass<'a>(
		mut self,
		render_pass: &RenderPass,
		attachments: &[&Texture],
		pass: impl FnOnce(RenderPassRecorder<'a>) -> RenderPassRecorder<'a>,
	) -> Self {
		let mut a = Vec::with_capacity(attachments.len());
		attachments.iter().for_each(|e| a.push(e.0.clone()));

		self.api.begin_render_pass(render_pass.0.clone(), &a[..]);

		let mut recorder = pass(RenderPassRecorder {
			recorder: self,
			current_pipeline: None,
			push_constants: [0; 32],
		})
		.recorder;

		recorder.api.end_render_pass();

		recorder
	}

	pub fn finish(mut self) -> GraphicsCommandBuffer {
		self.api.end();
		GraphicsCommandBuffer { api: self.api }
	}
}

pub struct RenderPassRecorder<'a> {
	recorder: GraphicsRecorder,
	current_pipeline: Option<&'a Pipeline>,
	push_constants: [u32; 32],
}

impl<'a> RenderPassRecorder<'a> {
	pub fn clear(mut self, color: impl Into<Color>) -> Self {
		self.recorder.api.clear(color.into());
		self
	}

	pub fn bind_pipeline(mut self, pipeline: &'a Pipeline) -> Self {
		for (sampler, index) in pipeline.samplers.iter() {
			self.push_constants[*index] = sampler.bindless;
		}

		self.recorder.api.bind_pipeline(pipeline.api.clone());
		self.current_pipeline = Some(pipeline);
		self
	}

	pub fn bind_scissor(mut self, scissor: Option<Rect>) -> Self {
		self.recorder.api.bind_scissor(scissor);
		self
	}

	pub fn bind_vertex_buffer<T: Sized>(mut self, buffer: &Buffer<T>) -> Self {
		self.recorder.api.bind_vertex_buffer(buffer.api.clone());
		self
	}

	pub fn bind_index_buffer<T: Sized>(mut self, buffer: &Buffer<T>) -> Self {
		self.recorder.api.bind_index_buffer(buffer.api.clone());
		self
	}

	pub fn draw(mut self, vertex_count: usize, first_vertex: usize) -> Self {
		self.recorder.api.push_constants(&self.push_constants);
		self.recorder.api.draw(vertex_count, first_vertex);
		self
	}

	pub fn draw_indexed(mut self, index_count: usize, first_index: usize) -> Self {
		self.recorder.api.push_constants(&self.push_constants);
		self.recorder.api.draw_indexed(index_count, first_index);
		self
	}

	pub fn bind_constant<T: Sized>(mut self, name: &str, buffer: &Buffer<T>) -> Self {
		let pipeline = self.current_pipeline.unwrap();
		for (index, (import_name, imports)) in pipeline.file.constants.iter().enumerate() {
			if name != import_name {
				continue;
			}

			match imports {
				Constants::Entire(_) => {
					self.push_constants[index] = buffer.bindless().unwrap();
				}
				_ => unreachable!(),
			}
			self.recorder.api.bind_buffer(buffer.api.clone());
			break;
		}
		self
	}

	pub fn bind_constant_indexed<T: Sized>(
		mut self,
		name: &str,
		buffer: &Buffer<T>,
		index: usize,
	) -> Self {
		let pipeline = self.current_pipeline.unwrap();
		for (bindless_index, (import_name, constants)) in pipeline.file.constants.iter().enumerate()
		{
			if name != import_name {
				continue;
			}

			match constants {
				Constants::Indexed(_) => {
					let bindless = buffer.bindless().unwrap();
					self.push_constants[bindless_index] =
						((bindless & 0xffff) << 16) | ((index as u32) & 0xffff);
				}
				_ => unreachable!(),
			}
			self.recorder.api.bind_buffer(buffer.api.clone());
			break;
		}
		self
	}

	pub fn bind_texture(mut self, name: &str, texture: &Texture) -> Self {
		let pipeline = self.current_pipeline.unwrap();

		// Resources come after constants in push constants so we need to add the constants len
		let constants_len = pipeline.file.constants.len();

		for (index, (import_name, _)) in pipeline.file.resources.iter().enumerate() {
			if name != import_name {
				continue;
			}

			self.push_constants[index + constants_len] = texture.bindless().unwrap();
			self.recorder.api.bind_textures(texture.0.clone());
			break;
		}
		self
	}
}
