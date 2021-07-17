use crate::*;

pub struct GraphicsContext {
    pub(crate) api: api::GraphicsContext,
}

impl GraphicsContext {
    pub fn begin(&mut self) {
        self.api.begin();
    }

    pub fn end(&mut self) {
        self.api.end();
    }

    pub fn resource_barrier_texture(&mut self, texture: &Texture, old_layout: Layout, new_layout: Layout) {
        self.api.resource_barrier_texture(texture.0.clone(), old_layout, new_layout);
    }

    pub fn copy_buffer_to_texture<T: Sized>(&mut self, dst: &Texture, src: &Buffer<T>) {
        self.api.copy_buffer_to_texture(dst.0.clone(), src.api.clone());
    }

    pub fn copy_buffer_to_buffer<T: Sized>(&mut self, dst: &Buffer<T>, src: &Buffer<T>) {
        self.api.copy_buffer_to_buffer(dst.api.clone(), src.api.clone());
    }

    pub fn render_pass<'b>(&mut self, render_pass: &RenderPass, attachments: &[&Texture], pass: impl FnOnce(RenderPassContext<'_, 'b>)) {
        let mut a = Vec::with_capacity(attachments.len());
        attachments.iter().for_each(|e| a.push(e.0.clone()) );

        self.api.begin_render_pass(render_pass.0.clone(), &a[..]);
        {
            pass(RenderPassContext{ gfx: self, current_pipeline: None, constants: [0; 32] });
        }
        self.api.end_render_pass();
    }
}

pub struct RenderPassContext<'a, 'b> {
    gfx: &'a mut GraphicsContext,
    current_pipeline: Option<&'b Pipeline>,
    constants: [u32; 32],
}

impl<'a, 'b> RenderPassContext<'a, 'b> {
    pub fn clear(self, color: impl Into<Color>) -> Self {
        self.gfx.api.clear(color.into());
        self
    }

    pub fn bind_pipeline(mut self, pipeline: &'b Pipeline) -> Self {
        self.gfx.api.bind_pipeline(pipeline.api.clone());
        self.current_pipeline = Some(pipeline);
        self
    }

    pub fn bind_scissor(self, scissor: Option<Rect>) -> Self {
        self.gfx.api.bind_scissor(scissor);
        self
    }

    pub fn bind_vertex_buffer<T: Sized>(self, buffer: &Buffer<T>) -> Self {
        self.gfx.api.bind_vertex_buffer(buffer.api.clone());
        self
    }

    pub fn bind_index_buffer<T: Sized>(self, buffer: &Buffer<T>) -> Self {
        self.gfx.api.bind_index_buffer(buffer.api.clone());
        self
    }
    
    pub fn draw(self, vertex_count: usize, first_vertex: usize) -> Self{ 
        self.gfx.api.draw(vertex_count, first_vertex);
        self
    }
    
    pub fn draw_indexed(self, index_count: usize, first_index: usize) -> Self {
        self.gfx.api.draw_indexed(index_count, first_index);
        self
    }

    pub fn bind<T: Sized>(mut self, name: &str, buffer: &Buffer<T>) -> Self {
        let pipeline = self.current_pipeline.unwrap();
        for (index, (import_name, imports)) in pipeline.file.imports.iter().enumerate() {
            if name != import_name {
                continue;
            }

            match imports {
                Imports::Entire(_) => {
                    self.constants[index] = buffer.bindless().unwrap();
                },
                _ => unreachable!()
            }
        }
        self
    }

    pub fn bind_index<T: Sized>(mut self, name: &str, buffer: &Buffer<T>, index: usize) -> Self {
        let pipeline = self.current_pipeline.unwrap();
        for (bindless_index, (import_name, imports)) in pipeline.file.imports.iter().enumerate() {
            if name != import_name {
                continue;
            }

            match imports {
                Imports::Indexed(_) => {
                    let bindless = buffer.bindless().unwrap();
                    self.constants[bindless_index] = ((bindless & 0xffff) << 16) | ((index as u32) & 0xffff);
                },
                _ => unreachable!()
            }
        }
        self
    }
}