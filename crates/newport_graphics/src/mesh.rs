use crate::{
    asset,
    gpu,
    math,
    engine,
    serde,

    Graphics,
};

use asset::{
    Asset,
    deserialize,
    UUID,
};

use gpu::{
    Buffer,
    VertexAttribute,
    BufferUsage,
    MemoryType,
};

use engine::{
    Engine,
};

use math::{
    Vector3, 
    Vector2,
};

use std::{
    mem::size_of,
    path::{ Path, PathBuf },
};

use serde::{ 
    Serialize, 
    Deserialize, 
};

use gltf;

#[derive(Serialize, Deserialize, Default)]
#[serde(crate = "self::serde")]
pub struct Vertex {
    pub position:  Vector3,

    #[serde(default)]
    pub normal:    Vector3,

    #[serde(default)]
    pub tangent:   Vector3,

    #[serde(default)]
    pub bitangent: Vector3,
    
    #[serde(default)]
    pub uv0: Vector2,

    #[serde(default)]
    pub uv1: Vector2,
}

impl gpu::Vertex for Vertex {
    fn attributes() -> Vec<VertexAttribute> {
        vec![
            VertexAttribute::Vector3,
            VertexAttribute::Vector3,
            VertexAttribute::Vector3,
            VertexAttribute::Vector3,

            VertexAttribute::Vector2,
            VertexAttribute::Vector2,
        ]
    }
}

pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices:  Vec<u32>,

    pub vertex_buffer: Buffer,
    pub index_buffer:  Buffer,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Mesh", crate = "self::serde")]
struct MeshFile {
    raw: PathBuf,
}

impl Asset for Mesh {
    fn load(bytes: &[u8], path: &Path) -> (UUID, Self) {
        let (id, mesh_file): (UUID, MeshFile) = deserialize(bytes).unwrap();

        let raw_path = path.with_file_name(mesh_file.raw);

        let (gltf, buffers, _images) = gltf::import(raw_path).unwrap();

        let mut vertex_count = 0;
        let mut index_count = 0;

        let mesh = gltf.meshes().nth(0).unwrap();
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
            let mut uvs = reader.read_tex_coords(0).unwrap().into_f32();


            let positions  = reader.read_positions().unwrap();
            for position in positions {
                let normal = normals.next().unwrap_or_default();
                let uv = uvs.next().unwrap();

                vertices.push(Vertex{
                    position: position.into(),
                    normal:   normal.into(),
                    uv0:      uv.into(),
                    ..Default::default()
                });
            }
        }

        let engine = Engine::as_ref();
        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let transfer_vertex = device.create_buffer(
            BufferUsage::TRANSFER_SRC, 
            MemoryType::HostVisible, 
            vertices.len() * size_of::<Vertex>(),
        ).unwrap();
        transfer_vertex.copy_to(&vertices[..]);

        let transfer_index = device.create_buffer(
            BufferUsage::TRANSFER_SRC, 
            MemoryType::HostVisible, 
            indices.len() * size_of::<u32>(),
        ).unwrap();
        transfer_index.copy_to(&indices[..]);

        let vertex_buffer = device.create_buffer(
            BufferUsage::TRANSFER_DST | BufferUsage::VERTEX, 
            MemoryType::DeviceLocal, 
            vertices.len() * size_of::<Vertex>(),
        ).unwrap();

        let index_buffer = device.create_buffer(
            BufferUsage::TRANSFER_DST | BufferUsage::INDEX, 
            MemoryType::DeviceLocal, 
            indices.len() * size_of::<u32>(),
        ).unwrap();

        let mut gfx = device.create_graphics_context().unwrap();
        {
            gfx.begin();
            gfx.copy_buffer_to_buffer(&vertex_buffer, &transfer_vertex);
            gfx.copy_buffer_to_buffer(&index_buffer, &transfer_index);
            gfx.end();
        }

        let receipt = device.submit_graphics(vec![gfx], &[]);
        receipt.wait();

        (id, Self{ vertices, indices, vertex_buffer, index_buffer })
    }
}