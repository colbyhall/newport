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
    path::Path,
};

use serde::{ 
    Serialize, 
    Deserialize, 
};

#[derive(Serialize, Deserialize)]
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
    pub vertex_buffer: Buffer,
    pub index_buffer:  Buffer,
}

#[derive(Serialize, Deserialize)]
#[serde(rename = "Mesh", crate = "self::serde")]
struct MeshFile {
    vertices: Vec<Vertex>,
    indices:  Vec<u32>,
}

impl Asset for Mesh {
    fn load(bytes: &[u8], _path: &Path) -> (UUID, Self) {
        let (id, mesh_file): (UUID, MeshFile) = deserialize(bytes).unwrap();

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

        (id, Self{ vertex_buffer, index_buffer })
    }
}