use {
	crate::Transform,
	ecs::{
		Query,
		World,
	},
	gpu::{
		Buffer,
		BufferUsage,
		Format,
		GraphicsPipeline,
		GraphicsRecorder,
		Layout,
		MemoryType,
		Texture,
		TextureUsage,
	},
	graphics::Mesh,
	math::{
		Color,
		Mat4,
		Vec2,
		Vec3,
		Vec4,
	},
	resources::Handle,
	serde::{
		Deserialize,
		Serialize,
	},
	std::{
		mem,
		sync::Mutex,
	},
};

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct Camera {
	pub fov: f32,
}

impl Default for Camera {
	fn default() -> Self {
		Self { fov: 90.0 }
	}
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct MeshFilter {
	pub mesh: Handle<Mesh>,
	pub pipeline: Handle<GraphicsPipeline>, // TODO: Material system
}

pub enum Frame {
	None,
	DrawList(DrawList),
	RenderedScene(RenderedScene),
}

struct RendererInner {
	frames: [Frame; 8],
	current: usize,
}

pub struct Renderer(Mutex<RendererInner>);

impl Renderer {
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
		world_transforms_buffer
			.copy_to(&scene.world_transforms)
			.unwrap();

		let diffuse_buffer = Texture::new(
			TextureUsage::SAMPLED | TextureUsage::COLOR_ATTACHMENT,
			Format::RGBA_U8,
			scene.viewport.x as u32,
			scene.viewport.y as u32,
			1,
		)
		.unwrap();

		let depth_buffer = Texture::new(
			TextureUsage::DEPTH_ATTACHMENT,
			Format::D24_S8,
			scene.viewport.x as u32,
			scene.viewport.y as u32,
			1,
		)
		.unwrap();

		let viewport = scene.viewport;
		let proj = Mat4::perspective(scene.camera.fov, viewport.x / viewport.y, 1000.0, 0.1);
		let view = Mat4::rotate(scene.camera_transform.rotation.inverse())
			* Mat4::translate(-scene.camera_transform.location);

		let axis_adjustment = Mat4 {
			x_column: Vec4::new(0.0, 0.0, -1.0, 0.0),
			y_column: Vec4::new(1.0, 0.0, 0.0, 0.0),
			z_column: Vec4::new(0.0, 1.0, 0.0, 0.0),
			w_column: Vec4::new(0.0, 0.0, 0.0, 1.0),
		};

		#[allow(dead_code)]
		struct CameraProperties {
			view: Mat4,
			position: Vec3,
		}

		let view_buffer = Buffer::new(BufferUsage::CONSTANTS, MemoryType::HostVisible, 1).unwrap();
		view_buffer
			.copy_to(&[CameraProperties {
				view: proj * axis_adjustment * view,
				position: scene.camera_transform.location,
			}])
			.unwrap();

		GraphicsRecorder::new()
			.resource_barrier_texture(&diffuse_buffer, Layout::Undefined, Layout::ColorAttachment)
			.resource_barrier_texture(&depth_buffer, Layout::Undefined, Layout::DepthAttachment)
			.render_pass(&[&diffuse_buffer, &depth_buffer], |ctx| {
				let mut ctx = ctx.clear_color(Color::BLACK).clear_depth(1.0);

				for (index, filter) in scene.meshes.iter().enumerate() {
					let pipeline = filter.pipeline.read();
					let mesh = filter.mesh.read();
					ctx = ctx
						.bind_pipeline(&pipeline)
						.bind_vertex_buffer(&mesh.vertex_buffer)
						.bind_index_buffer(&mesh.index_buffer)
						.bind_constants("imports", &world_transforms_buffer, index)
						.bind_constants("camera", &view_buffer, 0)
						.draw_indexed(mesh.indices.len(), 0)
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

impl Renderer {
	pub fn new() -> Self {
		Self(Mutex::new(RendererInner {
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

impl Default for Renderer {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone, Debug)]
pub struct DrawList {
	meshes: Vec<MeshFilter>,
	world_transforms: Vec<Mat4>,

	camera_transform: Transform,
	camera: Camera,

	viewport: Vec2,
}

impl DrawList {
	pub async fn build(world: &World, viewport: Vec2) -> Self {
		let filters = world.read::<MeshFilter>().await;
		let transforms = world.read::<Transform>().await;

		let entities = Query::new()
			.read(&filters)
			.read(&transforms)
			.execute(world)
			.await;

		let mut world_transforms = Vec::with_capacity(entities.len());
		let mut mesh_filters = Vec::with_capacity(entities.len());

		for e in entities.iter() {
			let transform = transforms.get(e).unwrap();
			let filter = filters.get(e).unwrap();

			world_transforms.push(transform.to_mat4());
			mesh_filters.push(filter.clone());
		}

		let cameras = world.read::<Camera>().await;
		let entities = Query::new()
			.read(&cameras)
			.read(&transforms)
			.execute(world)
			.await;

		let mut camera_transform = None;
		let mut camera = None;
		for e in entities.iter() {
			let transform = transforms.get(e).unwrap();
			let cam = cameras.get(e).unwrap();

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
			meshes: mesh_filters,
			world_transforms,

			camera_transform,
			camera,

			viewport,
		}
	}
}

#[derive(Clone)]
pub struct RenderedScene {
	pub diffuse_buffer: Texture,
	pub depth_buffer: Texture,
}
