use std::fmt::Debug;

use asset::Asset;

use asset::Importer;
use gpu::{
	Buffer,
	BufferUsage,
	GraphicsRecorder,
	MemoryType,
};

use math::{
	Vector2,
	Vector3,
};

use serde::{
	self,
	Deserialize,
	Serialize,
};

#[derive(Serialize, Deserialize, Default, Debug, PartialEq)]
pub struct Vertex {
	pub position: Vector3,

	#[serde(default)]
	pub normal: Vector3,

	#[serde(default)]
	pub tangent: Vector3,

	#[serde(default)]
	pub bitangent: Vector3,

	#[serde(default)]
	pub uv0: Vector2,

	#[serde(default)]
	pub uv1: Vector2,
}

pub struct Mesh {
	pub vertices: Vec<Vertex>,
	pub indices: Vec<u32>,

	pub vertex_buffer: Buffer<Vertex>,
	pub index_buffer: Buffer<u32>,
}

impl Debug for Mesh {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		f.debug_struct("Mesh")
			.field("vertices", &self.vertices)
			.field("indices", &self.indices)
			.finish()
	}
}

impl Asset for Mesh {
	fn default_uuid() -> Option<asset::UUID> {
		Some("{03383b92-566f-4036-aeb4-850b61685ea6}".into())
	}
}

#[derive(Serialize, Deserialize)]
pub(crate) struct MeshGltfImporter {}

impl Importer for MeshGltfImporter {
	type Target = Mesh;

	fn import(&self, bytes: &[u8]) -> asset::Result<Self::Target> {
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
		transfer_vertex.copy_to(&vertices[..]);

		let transfer_index = gpu::Buffer::new(
			BufferUsage::TRANSFER_SRC,
			MemoryType::HostVisible,
			indices.len(),
		)?;
		transfer_index.copy_to(&indices[..]);

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
}
