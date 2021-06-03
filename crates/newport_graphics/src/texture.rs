use crate::{
    asset,
    gpu,
    serde,

    engine::Engine,

    Graphics,
};

use asset::{
    Asset,
    deserialize,
    UUID,
};

use gpu::{ 
    BufferUsage, 
    MemoryType, 
    TextureUsage, 
    Format, 
    Wrap, 
    Filter, 
    Layout 
};


use serde::{ 
    Serialize, 
    Deserialize 
};
use stb_image::{
    image,
    image::LoadResult,
};

use std::{
    path::{ PathBuf, Path },
    fs,
};


pub struct Texture {
    srgb: bool,

    gpu:  gpu::Texture,
}

impl Texture {
    pub fn srgb(&self) -> bool {
        self.srgb
    }
    
    pub fn gpu(&self) -> &gpu::Texture {
        &self.gpu
    }
}

#[derive(Serialize, Deserialize)]
#[serde(crate = "self::serde", rename = "Texture")]
struct TextureFile {
    raw:  PathBuf,
    srgb: bool,
}

impl Asset for Texture {
    fn load(bytes: &[u8], path: &Path) -> (UUID, Self) {
        let (id, texture): (UUID, TextureFile) = deserialize(bytes).unwrap();

        let raw_path = path.with_file_name(texture.raw);

        let raw = fs::read(&raw_path).unwrap();

        let raw_texture = match image::load_from_memory(&raw[..]) {
            LoadResult::Error(err) => {
                panic!("Failed to load texture from file due to {}", err);
            },
            LoadResult::ImageU8(image) => {
                let engine = Engine::as_ref();
                let graphics = engine.module::<Graphics>().unwrap();
                let device = graphics.device();

                assert_eq!(image.depth, 4, "Currently vulkan only supports 4 byte formats");

                let pixel_buffer = device.create_buffer(
                    BufferUsage::TRANSFER_SRC, 
                    MemoryType::HostVisible, 
                    image.data.len()
                ).unwrap();
                pixel_buffer.copy_to(&image.data[..]);

                let format = if texture.srgb {
                    Format::RGBA_U8_SRGB
                } else {
                    Format::RGBA_U8
                };

                let gpu_texture = device.create_texture(
                    TextureUsage::TRANSFER_DST | TextureUsage::SAMPLED,
                    MemoryType::DeviceLocal, 
                    format,
                    image.width as u32,
                    image.height as u32,
                    1,
                    Wrap::Clamp,
                    Filter::Linear,
                    Filter::Linear
                ).unwrap();

                let mut gfx = device.create_graphics_context().unwrap();
                {
                    gfx.begin();
                    gfx.resource_barrier_texture(&gpu_texture, Layout::Undefined, Layout::TransferDst);
                    gfx.copy_buffer_to_texture(&gpu_texture, &pixel_buffer);
                    gfx.resource_barrier_texture(&gpu_texture, Layout::TransferDst, Layout::ShaderReadOnly);
                    gfx.end();
                }
        
                let receipt = device.submit_graphics(vec![gfx], &[]);
                receipt.wait();

                gpu_texture
            },
            _ => unimplemented!()
        };

        (id, Texture{
            srgb: texture.srgb,
            gpu: raw_texture
        })
    }
}