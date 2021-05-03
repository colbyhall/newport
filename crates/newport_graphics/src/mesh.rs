use crate::{
    asset,
    gpu,
    math,
    engine,

    Graphics,
};

use asset::{
    Asset,
    Path,
    LoadError,
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
    fs,
    mem::size_of,
};

use serde::{ Serialize, Deserialize, };

#[derive(Serialize, Deserialize)]
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
    vertex_buffer: Buffer,
    index_buffer:  Buffer,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Mesh")]
struct MeshFile {
    vertices: Vec<Vertex>,
    indices:  Vec<u32>,
}

impl Asset for Mesh {
    fn load(path: &Path) -> Result<Self, LoadError> {
        let file = fs::read_to_string(path).map_err(|_| LoadError::FileNotFound)?;
        let mesh_file: MeshFile = asset::de::from_str(&file).map_err(|_| LoadError::DataError)?;

        let engine = Engine::as_ref();
        let graphics = engine.module::<Graphics>().unwrap();
        let device = graphics.device();

        let transfer_vertex = device.create_buffer(
            BufferUsage::TRANSFER_SRC, 
            MemoryType::HostVisible, 
            mesh_file.vertices.len() * size_of::<Vertex>(),
        ).unwrap();
        transfer_vertex.copy_to(&mesh_file.vertices[..]);

        let transfer_index = device.create_buffer(
            BufferUsage::TRANSFER_SRC, 
            MemoryType::HostVisible, 
            mesh_file.indices.len() * size_of::<u32>(),
        ).unwrap();
        transfer_index.copy_to(&mesh_file.indices[..]);

        let vertex_buffer = device.create_buffer(
            BufferUsage::TRANSFER_DST | BufferUsage::VERTEX, 
            MemoryType::DeviceLocal, 
            mesh_file.vertices.len() * size_of::<Vertex>(),
        ).unwrap();

        let index_buffer = device.create_buffer(
            BufferUsage::TRANSFER_DST | BufferUsage::INDEX, 
            MemoryType::DeviceLocal, 
            mesh_file.indices.len() * size_of::<u32>(),
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

        Ok(Self{ vertex_buffer, index_buffer })
    }

    fn extension() -> &'static str {
        "mesh"
    }
}