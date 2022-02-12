use {
	crate::Transform,
	ecs::{
		Query,
		System,
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
	math::{
		Color,
		Mat4,
		Point3,
		Quat,
		Vec2,
		Vec3,
		Vec4,
	},
	resources::{
		Handle,
		Importer,
		Resource,
	},
	serde::{
		Deserialize,
		Serialize,
	},
	std::{
		fmt,
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

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Vertex {
	pub position: Vec3,

	#[serde(default)]
	pub normal: Vec3,

	#[serde(default)]
	pub tangent: Vec3,

	#[serde(default)]
	pub bitangent: Vec3,

	#[serde(default)]
	pub uv0: Vec2,

	#[serde(default)]
	pub uv1: Vec2,
}

pub struct Mesh {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u32>,

	pub vertex_buffer: Buffer<Vertex>,
	pub index_buffer: Buffer<u32>,
}

impl fmt::Debug for Mesh {
	fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
		f.debug_struct("Mesh")
			.field("vertices", &self.vertices)
			.field("indices", &self.indices)
			.finish()
	}
}

impl Resource for Mesh {
	fn default_uuid() -> Option<engine::Uuid> {
		Some("{03383b92-566f-4036-aeb4-850b61685ea6}".into())
	}
}

#[derive(Serialize, Deserialize)]
pub(crate) struct MeshGltfImporter {}

impl Importer for MeshGltfImporter {
	type Target = Mesh;

	fn import(&self, bytes: &[u8]) -> resources::Result<Self::Target> {
		let (gltf, buffers, _images) = gltf::import_slice(bytes)?;

		let mut vertex_count = 0;
		let mut index_count = 0;

		let mesh = gltf.meshes().next().unwrap();
		for primitive in mesh.primitives() {
			let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

			vertex_count += reader.read_positions().unwrap().count();
			index_count += match reader.read_indices().unwrap() {
				gltf::mesh::util::ReadIndices::U8(iter) => iter.count(),
				gltf::mesh::util::ReadIndices::U16(iter) => iter.count(),
				gltf::mesh::util::ReadIndices::U32(iter) => iter.count(),
			};
		}

		let mut vertices = Vec::with_capacity(vertex_count);
		let mut indices = Vec::with_capacity(index_count);

		for primitive in mesh.primitives() {
			let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

			let base = vertices.len() as u32;
			for index in reader.read_indices().unwrap().into_u32() {
				indices.push(base + index);
			}

			let mut normals = reader.read_normals().unwrap();

			match reader.read_tex_coords(0) {
				Some(uvs) => {
					let mut uvs = uvs.into_f32();
					let positions = reader.read_positions().unwrap();
					for position in positions {
						let normal = normals.next().unwrap_or_default();
						let uv = uvs.next().unwrap();

						vertices.push(Vertex {
							position: position.into(),
							normal: normal.into(),
							uv0: uv.into(),
							..Default::default()
						});
					}
				}
				None => {
					let positions = reader.read_positions().unwrap();
					for position in positions {
						let normal = normals.next().unwrap_or_default();

						vertices.push(Vertex {
							position: position.into(),
							normal: normal.into(),
							..Default::default()
						});
					}
				}
			}
		}

		let transfer_vertex = gpu::Buffer::new(
			BufferUsage::TRANSFER_SRC,
			MemoryType::HostVisible,
			vertices.len(),
		)?;
		transfer_vertex.copy_to(&vertices[..]).unwrap();

		let transfer_index = gpu::Buffer::new(
			BufferUsage::TRANSFER_SRC,
			MemoryType::HostVisible,
			indices.len(),
		)?;
		transfer_index.copy_to(&indices[..]).unwrap();

		let vertex_buffer = gpu::Buffer::new(
			BufferUsage::TRANSFER_DST | BufferUsage::VERTEX,
			MemoryType::DeviceLocal,
			vertices.len(),
		)?;

		let index_buffer = gpu::Buffer::new(
			BufferUsage::TRANSFER_DST | BufferUsage::INDEX,
			MemoryType::DeviceLocal,
			indices.len(),
		)?;

		GraphicsRecorder::new()
			.copy_buffer_to_buffer(&vertex_buffer, &transfer_vertex)
			.copy_buffer_to_buffer(&index_buffer, &transfer_index)
			.submit()
			.wait();

		Ok(Self::Target {
			vertices,
			indices,
			vertex_buffer,
			index_buffer,
		})
	}

	fn export(&self, _resource: &Self::Target, _file: &mut std::fs::File) -> resources::Result<()> {
		todo!()
	}
}

#[derive(Default, Clone, Serialize, Deserialize, Debug)]
pub struct MeshFilter {
	pub mesh: Handle<Mesh>,
	pub pipeline: Handle<GraphicsPipeline>, // TODO: Material system
}

#[derive(Clone, Debug)]
pub struct DebugShape {
	line_width: f32,
	color: Color,

	time_left: f32,

	location: Point3,
	rotation: Quat,
	variant: DebugShapeVariant,
}

impl DebugShape {
	pub fn color(&mut self, color: Color) -> &mut Self {
		self.color = color;
		self
	}

	pub fn line_width(&mut self, line_width: f32) -> &mut Self {
		self.line_width = line_width;
		self
	}
}

#[derive(Clone, Debug)]
#[allow(dead_code)]
enum DebugShapeVariant {
	Line { end: Point3 },
	Box { extent: Vec3 },
	Sphere { radius: f32 },
	Capsule { half_height: f32, radius: f32 },
	Plane { normal: Vec3, size: f32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct DebugManager {
	#[serde(skip)]
	shapes: Vec<DebugShape>,

	#[serde(skip)]
	pipeline: Handle<GraphicsPipeline>,
}

impl DebugManager {
	const DEFAULT_COLOR: Color = Color::CYAN;
	const DEFAULT_LINE_WIDTH: f32 = 0.01;

	pub fn new() -> Self {
		Self {
			shapes: Vec::with_capacity(2048),
			pipeline: Handle::find_or_load("{063952B6-40B8-4D22-A26F-339185008B76}").unwrap(),
		}
	}

	pub fn draw_line(&mut self, a: Point3, b: Point3, life_time: f32) -> &mut DebugShape {
		self.shapes.push(DebugShape {
			line_width: Self::DEFAULT_LINE_WIDTH,
			color: Self::DEFAULT_COLOR,

			time_left: life_time,

			location: a,
			rotation: Quat::IDENTITY,
			variant: DebugShapeVariant::Line { end: b },
		});
		self.shapes.last_mut().unwrap()
	}

	pub fn draw_box(
		&mut self,
		location: Point3,
		rotation: Quat,
		extent: Vec3,
		life_time: f32,
	) -> &mut DebugShape {
		self.shapes.push(DebugShape {
			line_width: Self::DEFAULT_LINE_WIDTH,
			color: Self::DEFAULT_COLOR,

			time_left: life_time,

			location,
			rotation,
			variant: DebugShapeVariant::Box { extent },
		});
		self.shapes.last_mut().unwrap()
	}
}

#[derive(Clone)]
pub struct DebugSystem;
impl System for DebugSystem {
	fn run(&self, world: &World, dt: f32) {
		let mut debug_managers = world.write::<DebugManager>();
		let debug_manager = match debug_managers.get_mut(world.singleton) {
			Some(c) => c,
			None => {
				world.insert(&mut debug_managers, world.singleton, DebugManager::new());
				debug_managers.get_mut(world.singleton).unwrap()
			}
		};

		// Update "time_left" for all shapes and then remove any that are "dead"
		debug_manager
			.shapes
			.iter_mut()
			.for_each(|e| e.time_left -= dt);
		debug_manager.shapes.retain(|e| e.time_left > 0.0)
	}
}

#[derive(Clone, Debug)]
struct DrawListData {
	model: Mat4,
	color: Color,
}

#[derive(Clone, Debug)]
pub struct DrawList {
	meshes: Vec<MeshFilter>,
	world_transforms: Vec<DrawListData>,

	debug_shapes: Vec<DebugShape>,

	camera_transform: Transform,
	camera: Camera,

	viewport: Vec2,
}

impl DrawList {
	pub fn build(world: &World, viewport: Vec2) -> Self {
		let filters = world.read::<MeshFilter>();
		let transforms = world.read::<Transform>();

		let entities = Query::new().read(&filters).read(&transforms).execute(world);

		let mut world_transforms = Vec::with_capacity(entities.len());
		let mut mesh_filters = Vec::with_capacity(entities.len());

		const COLORS: &[Color] = &[
			Color::RED,
			Color::GREEN,
			Color::BLUE,
			Color::WHITE,
			Color::BLACK,
			Color::CYAN,
			Color::YELLOW,
			Color::MAGENTA,
		];

		for (index, e) in entities.iter().copied().enumerate() {
			let transform = transforms.get(e).unwrap();
			let filter = filters.get(e).unwrap();

			let color = COLORS[index & (COLORS.len() - 1)];
			world_transforms.push(DrawListData {
				model: transform.local_mat4(),
				color,
			});
			mesh_filters.push(filter.clone());
		}

		let cameras = world.read::<Camera>();
		let entities = Query::new().read(&cameras).read(&transforms).execute(world);

		let mut camera_transform = None;
		let mut camera = None;
		for e in entities.iter().copied() {
			let transform = transforms.get(e).unwrap();
			let cam = cameras.get(e).unwrap();

			camera_transform = Some(transform.clone());
			camera = Some(cam.clone());

			// For some reason this has to be here to prevent a clippy bug
			if camera_transform.is_some() && camera.is_some() {
				break;
			}
		}
		let camera_transform = camera_transform.unwrap_or_default();
		let camera = camera.unwrap_or_default();

		let debug_managers = world.read::<DebugManager>();
		let debug_shapes = match debug_managers.get(world.singleton) {
			Some(e) => e.shapes.clone(),
			None => Vec::default(),
		};

		Self {
			meshes: mesh_filters,
			world_transforms,

			debug_shapes,

			camera_transform,
			camera,

			viewport,
		}
	}
}

#[allow(clippy::large_enum_variant)]
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

impl Renderer {
	pub fn push_scene(&self, scene: DrawList) {
		let mut inner = self.0.lock().unwrap();

		let current = inner.current;
		let len = inner.frames.len();

		inner.frames[current % len] = Frame::DrawList(scene);
	}

	pub fn render_scene(&self) {
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

		// TODO: Should this be done in the scene building part?
		#[allow(dead_code)]
		struct DebugVertex {
			position: Vec3,
			color: Color,
		}
		fn debug_batch_line(
			vertices: &mut Vec<DebugVertex>,
			a: Point3,
			b: Point3,
			up: Vec3,
			line_width: f32,
			color: Color,
		) {
			let mut forward = (b - a).norm();
			let mut up = up;
			let mut right = Vec3::cross(forward, up);
			Vec3::orthonormal_basis(&mut forward, &mut right, &mut up);

			let half_line_width = line_width / 2.0;

			let bl = a - right * half_line_width;
			let br = a + right * half_line_width;
			let tl = b - right * half_line_width;
			let tr = b + right * half_line_width;

			vertices.push(DebugVertex {
				position: bl,
				color,
			});
			vertices.push(DebugVertex {
				position: tl,
				color,
			});
			vertices.push(DebugVertex {
				position: tr,
				color,
			});

			vertices.push(DebugVertex {
				position: bl,
				color,
			});
			vertices.push(DebugVertex {
				position: tr,
				color,
			});
			vertices.push(DebugVertex {
				position: br,
				color,
			});

			let bl = a - up * half_line_width;
			let br = a + up * half_line_width;
			let tl = b - up * half_line_width;
			let tr = b + up * half_line_width;

			vertices.push(DebugVertex {
				position: bl,
				color,
			});
			vertices.push(DebugVertex {
				position: tl,
				color,
			});
			vertices.push(DebugVertex {
				position: tr,
				color,
			});

			vertices.push(DebugVertex {
				position: bl,
				color,
			});
			vertices.push(DebugVertex {
				position: tr,
				color,
			});
			vertices.push(DebugVertex {
				position: br,
				color,
			});
		}
		let mut debug_vertices = Vec::with_capacity(4096);
		for shape in scene.debug_shapes.iter() {
			match &shape.variant {
				DebugShapeVariant::Line { end } => {
					let forward = (*end - shape.location).norm();
					let up = if forward.dot(Vec3::UP) >= 0.5 {
						Vec3::FORWARD
					} else {
						Vec3::UP
					};
					debug_batch_line(
						&mut debug_vertices,
						shape.location,
						*end,
						up,
						shape.line_width,
						shape.color,
					)
				}
				DebugShapeVariant::Box { extent } => {
					let forward = shape.rotation.forward() * extent.x;
					let right = shape.rotation.right() * extent.y;
					let up = shape.rotation.up() * extent.z;

					let fbl = shape.location + forward - up - right;
					let fbr = shape.location + forward - up + right;
					let ftl = shape.location + forward + up - right;
					let ftr = shape.location + forward + up + right;
					let bbl = shape.location + -forward - up - right;
					let bbr = shape.location + -forward - up + right;
					let btl = shape.location + -forward + up - right;
					let btr = shape.location + -forward + up + right;

					let up = up.norm();
					let forward = forward.norm();

					debug_batch_line(
						&mut debug_vertices,
						fbl,
						fbr,
						up,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						fbl,
						bbl,
						up,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						fbr,
						bbr,
						up,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						ftl,
						ftr,
						up,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						ftl,
						btl,
						up,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						ftr,
						btr,
						up,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						bbl,
						bbr,
						up,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						ftl,
						fbl,
						forward,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						ftr,
						fbr,
						forward,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						btl,
						bbl,
						forward,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						btr,
						bbr,
						forward,
						shape.line_width,
						shape.color,
					);
					debug_batch_line(
						&mut debug_vertices,
						btl,
						btr,
						up,
						shape.line_width,
						shape.color,
					);
				}
				_ => unimplemented!(),
			}
		}
		let debug_vertex_buffer = if debug_vertices.is_empty() {
			None
		} else {
			let buffer = Buffer::new(
				BufferUsage::VERTEX,
				MemoryType::HostVisible,
				debug_vertices.len(),
			)
			.unwrap();
			buffer.copy_to(&debug_vertices).unwrap();
			Some(buffer)
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

		let debug_pipeline =
			Handle::find_or_load("{063952B6-40B8-4D22-A26F-339185008B76}").unwrap();
		let debug_pipeline = debug_pipeline.read();

		GraphicsRecorder::new()
			.texture_barrier(&diffuse_buffer, Layout::Undefined, Layout::ColorAttachment)
			.texture_barrier(&depth_buffer, Layout::Undefined, Layout::DepthAttachment)
			.render_pass(&[&diffuse_buffer, &depth_buffer], |ctx| {
				ctx.clear_color(Color::BLACK).clear_depth(1.0);

				// Draw all the meshes in the world with their given pipeline
				// TODO: Actual material based renderer
				for (index, filter) in scene.meshes.iter().enumerate() {
					let pipeline = filter.pipeline.read();
					let mesh = filter.mesh.read();
					ctx.set_pipeline(&pipeline)
						.set_vertex_buffer(&mesh.vertex_buffer)
						.set_index_buffer(&mesh.index_buffer)
						.set_constants("imports", &world_transforms_buffer, index)
						.set_constants("camera", &view_buffer, 0)
						.draw_indexed(mesh.indices.len(), 0);
				}

				// Draw all debug geometry
				// TODO: Should this be disabled on cooked build?
				if let Some(buffer) = debug_vertex_buffer {
					ctx.set_pipeline(&debug_pipeline)
						.set_vertex_buffer(&buffer)
						.set_constants("camera", &view_buffer, 0)
						.draw(buffer.len(), 0);
				}

				ctx
			})
			.texture_barrier(
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

impl Default for Renderer {
	fn default() -> Self {
		Self::new()
	}
}

#[derive(Clone)]
pub struct RenderedScene {
	pub diffuse_buffer: Texture,
	pub depth_buffer: Texture,
}
