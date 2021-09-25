use crate::ecs::Query;
use engine::Engine;
use gpu::{
	Buffer,
	BufferUsage,
	Format,
	GraphicsRecorder,
	Layout,
	MemoryType,
	Texture,
	TextureUsage,
};
use math::{
	Color,
	Matrix4,
	Vector2,
	Vector4,
};

use std::sync::Mutex;

use crate::components::{
	Camera,
	MeshRender,
	Transform,
};
use crate::game::GameState;

use std::mem;

pub enum Frame {
	None,
	DrawList(DrawList),
	RenderedScene(RenderedScene),
}

struct FrameContainerInner {
	frames: [Frame; 8],
	current: usize,
}

pub struct FrameContainer(Mutex<FrameContainerInner>);

impl FrameContainer {
	pub fn push_scene(&self, scene: DrawList) {
		let mut inner = self.0.lock().unwrap();

		let current = inner.current;
		let len = inner.frames.len();

		inner.frames[current % len] = Frame::DrawList(scene);
	}

	pub async fn render_scene(&self) {
		let frame = {
			let mut inner = self.0.lock().unwrap();

			let current = inner.current - 1;
			let len = inner.frames.len();
			let frame = &mut inner.frames[current % len];
			mem::replace(frame, Frame::None)
		};

		let scene = match frame {
			Frame::DrawList(scene) => scene,
			_ => return,
		};

		let world_transforms_buffer = Buffer::new(
			BufferUsage::CONSTANTS,
			MemoryType::HostVisible,
			scene.world_transforms.len(),
		)
		.unwrap();
		world_transforms_buffer.copy_to(&scene.world_transforms);

		let window = Engine::window().unwrap();
		let window_size = window.inner_size();

		let diffuse_buffer = Texture::new(
			TextureUsage::SAMPLED | TextureUsage::COLOR_ATTACHMENT,
			Format::RGBA_U8,
			window_size.width,
			window_size.height,
			1,
		)
		.unwrap();

		let depth_buffer = Texture::new(
			TextureUsage::DEPTH_ATTACHMENT,
			Format::D24_S8,
			window_size.width,
			window_size.height,
			1,
		)
		.unwrap();

		let viewport = Vector2::new(window_size.width as f32, window_size.height as f32);
		let proj = Matrix4::perspective(scene.camera.fov, viewport.x / viewport.y, 1000.0, 0.1);
		let view = Matrix4::rotate(scene.camera_transform.rotation.inverse())
			* Matrix4::translate(-scene.camera_transform.location);

		let axis_adjustment = Matrix4 {
			x_column: Vector4::new(0.0, 0.0, -1.0, 0.0),
			y_column: Vector4::new(1.0, 0.0, 0.0, 0.0),
			z_column: Vector4::new(0.0, 1.0, 0.0, 0.0),
			w_column: Vector4::new(0.0, 0.0, 0.0, 1.0),
		};

		let view_buffer = Buffer::new(BufferUsage::CONSTANTS, MemoryType::HostVisible, 1).unwrap();
		view_buffer.copy_to(&[proj * axis_adjustment * view]);

		GraphicsRecorder::new()
			.resource_barrier_texture(&diffuse_buffer, Layout::Undefined, Layout::ColorAttachment)
			.resource_barrier_texture(&depth_buffer, Layout::Undefined, Layout::DepthAttachment)
			.render_pass(&[&diffuse_buffer, &depth_buffer], |ctx| {
				let mut ctx = ctx.clear_color(Color::BLACK).clear_depth(1.0);

				for (index, mesh) in scene.meshes.iter().enumerate() {
					ctx = ctx
						.bind_pipeline(&mesh.pipeline)
						.bind_vertex_buffer(&mesh.mesh.vertex_buffer)
						.bind_index_buffer(&mesh.mesh.index_buffer)
						.bind_constants("imports", &world_transforms_buffer, index)
						.bind_constants("camera", &view_buffer, 0)
						.draw_indexed(mesh.mesh.indices.len(), 0)
				}

				ctx
			})
			.resource_barrier_texture(
				&diffuse_buffer,
				Layout::ColorAttachment,
				Layout::ShaderReadOnly,
			)
			.submit()
			.wait();

		let mut inner = self.0.lock().unwrap();

		let current = inner.current - 1;
		let len = inner.frames.len();
		let frame = &mut inner.frames[current % len];

		*frame = Frame::RenderedScene(RenderedScene {
			diffuse_buffer,
			depth_buffer,
		})
	}

	pub fn to_display(&self) -> Option<RenderedScene> {
		let inner = self.0.lock().unwrap();
		let current = inner.current - 2;
		let len = inner.frames.len();

		let frame = &inner.frames[current % len];
		match frame {
			Frame::RenderedScene(scene) => Some(scene.clone()),
			_ => None,
		}
	}

	pub fn advance_frame(&self) {
		let mut inner = self.0.lock().unwrap();
		inner.current += 1;
	}
}

impl FrameContainer {
	pub fn new() -> Self {
		Self(Mutex::new(FrameContainerInner {
			frames: [
				Frame::None,
				Frame::None,
				Frame::None,
				Frame::None,
				Frame::None,
				Frame::None,
				Frame::None,
				Frame::None,
			],
			current: 8,
		}))
	}
}

impl Default for FrameContainer {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone)]
pub struct DrawList {
	meshes: Vec<MeshRender>,
	world_transforms: Vec<Matrix4>,

	camera_transform: Transform,
	camera: Camera,
}

impl DrawList {
	pub async fn build(game_state: &GameState) -> Self {
		let mut query = Query::builder()
			.read::<MeshRender>()
			.read::<Transform>()
			.execute(game_state.world());

		let mut world_transforms = Vec::with_capacity(query.len());
		let mut meshes = Vec::with_capacity(query.len());

		for it in query.iter() {
			let transform: &Transform = it.get().unwrap();
			let mesh: &MeshRender = it.get().unwrap();

			world_transforms.push(transform.to_matrix4());
			meshes.push(mesh.clone());
		}

		let mut query = Query::builder()
			.read::<Camera>()
			.read::<Transform>()
			.execute(game_state.world());

		let mut camera_transform = None;
		let mut camera = None;
		for it in query.iter() {
			let transform: &Transform = it.get().unwrap();
			let cam: &Camera = it.get().unwrap();

			camera_transform = Some(*transform);
			camera = Some(cam.clone());

			// For some reason this has to be here to prevent a clippy bug
			if camera_transform.is_some() && camera.is_some() {
				break;
			}
		}
		let camera_transform = camera_transform.unwrap_or_default();
		let camera = camera.unwrap_or_default();

		Self {
			meshes,
			world_transforms,

			camera_transform,
			camera,
		}
	}
}

#[derive(Clone)]
pub struct RenderedScene {
	pub diffuse_buffer: Texture,
	pub depth_buffer: Texture,
}
