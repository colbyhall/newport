pub use base::*;

use {
	engine::{
		Builder,
		Engine,
		Event as EngineEvent,
		Module,
	},

	gpu::{
		Buffer,
		GraphicsPipeline,
		GraphicsRecorder,
		Texture,
	},
	newport_math::{
		srgb_to_linear,
		Color,
		Matrix4,
		Vector2,
		Vector3,
		Vector4,
	},
	platform::input::*,

	resources::Handle,
	std::sync::Arc,
};

struct EguiTexture {
	gpu: gpu::Texture,
	egui: Arc<base::Texture>,
}

pub struct Egui {
	context: CtxRef,
	input: Option<RawInput>,
	mouse_position: Option<Vector2>,
	tick: Option<f32>,

	pipeline: Handle<GraphicsPipeline>,
	texture: Option<EguiTexture>,
}

impl Module for Egui {
	fn new() -> Self {
		Self {
			context: CtxRef::default(),
			input: None,
			mouse_position: None,
			tick: None,

			pipeline: Handle::find_or_load("{1e1526a8-852c-47f7-8436-2bbb01fe8a22}").unwrap(),
			texture: None,
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<gpu::Gpu>()
			.module::<gpu::Gpu>()
			.module::<resources::ResourceManager>()
			.process_input(|event| {
				let egui: &mut Egui = unsafe { Engine::module_mut().unwrap() };
				if egui.input.is_none() {
					egui.input = Some(RawInput::default());
				}
				let input = egui.input.as_mut().unwrap();

				let window = Engine::window().unwrap();
				let viewport = window.inner_size();
				let dpi = window.scale_factor() as f32;
				let viewport =
					Vector2::new(viewport.width as f32 / dpi, viewport.height as f32 / dpi);

				match event {
					EngineEvent::Char(c) => {
						let mut s = String::new();
						s.push(*c);

						input.events.push(Event::Text(s));
					}
					EngineEvent::Key { key, pressed } => {
						let key = match *key {
							KEY_DOWN => Some(Key::ArrowDown),
							KEY_LEFT => Some(Key::ArrowLeft),
							KEY_RIGHT => Some(Key::ArrowRight),
							KEY_UP => Some(Key::ArrowUp),

							KEY_ESCAPE => Some(Key::Escape),
							KEY_TAB => Some(Key::Tab),
							KEY_BACKSPACE => Some(Key::Backspace),
							KEY_ENTER => Some(Key::Enter),
							KEY_SPACE => Some(Key::Space),
							KEY_INSERT => Some(Key::Insert),
							KEY_DELETE => Some(Key::Delete),
							KEY_HOME => Some(Key::Home),
							KEY_END => Some(Key::End),

							// TODO: PageUp
							// TODO: PageDown
							KEY_0 => Some(Key::Num0),
							KEY_1 => Some(Key::Num1),
							KEY_2 => Some(Key::Num2),
							KEY_3 => Some(Key::Num3),
							KEY_4 => Some(Key::Num4),
							KEY_5 => Some(Key::Num5),
							KEY_6 => Some(Key::Num6),
							KEY_7 => Some(Key::Num7),
							KEY_8 => Some(Key::Num8),
							KEY_9 => Some(Key::Num9),

							KEY_A => Some(Key::A),
							KEY_B => Some(Key::B),
							KEY_C => Some(Key::C),
							KEY_D => Some(Key::D),
							KEY_E => Some(Key::E),
							KEY_F => Some(Key::F),
							KEY_G => Some(Key::G),
							KEY_H => Some(Key::H),
							KEY_I => Some(Key::I),
							KEY_J => Some(Key::J),
							KEY_K => Some(Key::K),
							KEY_L => Some(Key::L),
							KEY_M => Some(Key::M),
							KEY_N => Some(Key::N),
							KEY_O => Some(Key::O),
							KEY_P => Some(Key::P),
							KEY_Q => Some(Key::Q),
							KEY_R => Some(Key::R),
							KEY_S => Some(Key::S),
							KEY_T => Some(Key::T),
							KEY_U => Some(Key::U),
							KEY_V => Some(Key::V),
							KEY_W => Some(Key::W),
							KEY_X => Some(Key::X),
							KEY_Y => Some(Key::Y),
							KEY_Z => Some(Key::Z),
							_ => None,
						};

						if key.is_none() {
							return;
						}
						let key = key.unwrap();
						input.events.push(Event::Key {
							key,
							pressed: *pressed,
							modifiers: Default::default(),
						});
					}
					EngineEvent::MouseButton {
						mouse_button,
						pressed,
					} => {
						let button = match *mouse_button {
							MOUSE_BUTTON_LEFT => Some(PointerButton::Primary),
							MOUSE_BUTTON_MIDDLE => Some(PointerButton::Middle),
							MOUSE_BUTTON_RIGHT => Some(PointerButton::Secondary),
							_ => None,
						};
						if button.is_none() {
							return;
						}
						let button = button.unwrap();

						let position = egui.mouse_position.unwrap_or_default();

						input.events.push(Event::PointerButton {
							pos: pos2(position.x, position.y),
							button,
							pressed: *pressed,
							modifiers: Default::default(),
						});
					}
					EngineEvent::MouseMove(x, y) => {
						let position = Vector2::new(*x as f32 / dpi, *y as f32 / dpi);
						let position = Vector2::new(position.x, viewport.y - position.y);
						egui.mouse_position = Some(position);
						input
							.events
							.push(Event::PointerMoved(pos2(position.x, position.y)));
					}
					EngineEvent::MouseLeave => {
						egui.mouse_position = None;
						input.events.push(Event::PointerGone);
					}
					_ => {}
				}
			})
			.tick(|dt| {
				let egui: &mut Egui = unsafe { Engine::module_mut().unwrap() };
				egui.tick = Some(dt);
			})
			.display(|| {
				let egui: &mut Egui = unsafe { Engine::module_mut().unwrap() };
				let dt = egui.tick.take().unwrap_or_default();

				// Gather up final information for input
				let window = Engine::window().unwrap();
				let dpi = window.scale_factor() as f32;
				let viewport = window.inner_size();
				let viewport =
					Vector2::new(viewport.width as f32 / dpi, viewport.height as f32 / dpi);

				let mut input = egui.input.take().unwrap_or_default();
				input.screen_rect = Some(Rect::from_min_max(
					pos2(0.0, 0.0),
					pos2(viewport.x, viewport.y),
				));
				input.predicted_dt = dt;
				input.pixels_per_point = Some(dpi);

				// Do egui frame using scope registers
				egui.context.begin_frame(input);
				let scopes: &[EguiScope] = Engine::register().unwrap_or_default();
				scopes.iter().for_each(|f| (f.0)(&egui.context));
				let (_output, shapes) = egui.context.end_frame();

				// Upload the egui texture if need be
				let texture = egui.context.texture();
				let texture = if egui.texture.is_none()
					|| egui.texture.is_some()
						&& egui.texture.as_ref().unwrap().egui.version != texture.version
				{
					let pixels: Vec<u32> = texture
						.pixels
						.iter()
						.map(|it| {
							let mut color: u32 = (*it as u32) << 24;
							color |= (*it as u32) << 16;
							color |= (*it as u32) << 8;
							color |= *it as u32;
							color
						})
						.collect();

					let pixel_buffer = Buffer::new(
						gpu::BufferUsage::TRANSFER_SRC,
						gpu::MemoryType::HostVisible,
						pixels.len(),
					)
					.unwrap();
					pixel_buffer.copy_to(&pixels[..]).unwrap();

					let gpu_texture = Texture::new(
						gpu::TextureUsage::TRANSFER_DST | gpu::TextureUsage::SAMPLED,
						gpu::Format::RGBA_U8_SRGB,
						texture.width as u32,
						texture.height as u32,
						1,
					)
					.unwrap();

					GraphicsRecorder::new()
						.resource_barrier_texture(
							&gpu_texture,
							gpu::Layout::Undefined,
							gpu::Layout::TransferDst,
						)
						.copy_buffer_to_texture(&gpu_texture, &pixel_buffer)
						.resource_barrier_texture(
							&gpu_texture,
							gpu::Layout::Undefined,
							gpu::Layout::TransferDst,
						)
						.finish()
						.submit()
						.wait();

					egui.texture = Some(EguiTexture {
						gpu: gpu_texture,
						egui: texture,
					});

					egui.texture.as_ref().unwrap()
				} else {
					egui.texture.as_ref().unwrap()
				};

				let clipped_meshes = egui.context.tessellate(shapes);
				if clipped_meshes.is_empty() {
					return;
				}

				// Load all of the vertex data into one giant buffer
				let mut vertex_buffer_len = 0;
				let mut index_buffer_len = 0;
				for it in clipped_meshes.iter() {
					vertex_buffer_len += it.1.vertices.len();
					index_buffer_len += it.1.indices.len();
				}

				let mut vertices = Vec::with_capacity(vertex_buffer_len);
				let mut indices = Vec::with_capacity(index_buffer_len);

				#[allow(dead_code)]
				struct Vertex {
					position: Vector2,
					uv: Vector2,
					scissor: Vector4,
					color: Color,
					tex: u32,
				}

				let mut indice_start = 0;
				for it in clipped_meshes.iter() {
					for vertex in it.1.vertices.iter() {
						let tex = match it.1.texture_id {
							TextureId::Egui => texture.gpu.bindless().unwrap(),
							TextureId::User(num) => num as u32,
						};

						let r = srgb_to_linear(vertex.color.r());
						let g = srgb_to_linear(vertex.color.g());
						let b = srgb_to_linear(vertex.color.b());
						let a = vertex.color.a() as f32 / 255.0;

						let color = Color::new(r, g, b, a);

						vertices.push(Vertex {
							position: Vector2::new(vertex.pos.x, vertex.pos.y),
							uv: Vector2::new(vertex.uv.x, vertex.uv.y),
							scissor: Vector4::new(it.0.min.x, it.0.min.y, it.0.max.x, it.0.max.y),
							color,
							tex,
						});
					}

					for index in it.1.indices.iter() {
						indices.push(index + indice_start);
					}

					indice_start += it.1.vertices.len() as u32;
				}

				let vertex_buffer = Buffer::new(
					gpu::BufferUsage::VERTEX,
					gpu::MemoryType::HostVisible,
					vertex_buffer_len,
				)
				.unwrap();
				vertex_buffer.copy_to(&vertices[..]).unwrap();

				let index_buffer = Buffer::new(
					gpu::BufferUsage::INDEX,
					gpu::MemoryType::HostVisible,
					index_buffer_len,
				)
				.unwrap();
				index_buffer.copy_to(&indices[..]).unwrap();

				let proj = Matrix4::ortho(viewport.x, -viewport.y, 1000.0, 0.1);
				let view =
					Matrix4::translate(Vector3::new(-viewport.x / 2.0, -viewport.y / 2.0, 0.0));

				#[allow(dead_code)]
				struct Imports {
					view: Matrix4,
				}

				let imports =
					Buffer::new(gpu::BufferUsage::CONSTANTS, gpu::MemoryType::HostVisible, 1)
						.unwrap();
				imports.copy_to(&[Imports { view: proj * view }]).unwrap();

				let device = gpu::Gpu::device();
				let backbuffer = device.acquire_backbuffer().unwrap();

				let pipeline = egui.pipeline.read();
				let receipt = GraphicsRecorder::new()
					.render_pass(&[&backbuffer], |ctx| {
						ctx.clear_color(Color::BLACK)
							.bind_pipeline(&pipeline)
							.bind_vertex_buffer(&vertex_buffer)
							.bind_index_buffer(&index_buffer)
							.bind_constants("imports", &imports, 0)
							.draw_indexed(indices.len(), 0)
					})
					.resource_barrier_texture(
						&backbuffer,
						gpu::Layout::ColorAttachment,
						gpu::Layout::Present,
					)
					.finish()
					.submit();

				device.display(&[receipt]);
				device.wait_for_idle();
			})
	}
}

pub struct EguiScope(Box<dyn Fn(&CtxRef) + 'static>);

impl EguiScope {
	pub fn new(f: impl Fn(&CtxRef) + 'static) -> Self {
		Self(Box::new(f))
	}
}
