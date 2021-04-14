use newport_asset::{ Asset, LoadError, de };
use crate::{ gpu, Graphics };
use gpu::{ BufferUsage, MemoryType, TextureUsage, Format, Wrap, Filter, Layout };
use newport_log::error;
use newport_engine::Engine;

use serde::{ Serialize, Deserialize };
use stb_image::image;
use image::LoadResult;

use std::path::{ PathBuf, Path };
use std::fs;

#[derive(Serialize, Deserialize)]
pub struct Texture {
    raw:  PathBuf,
    srgb: bool,

    #[serde(skip)]
    gpu:  Option<gpu::Texture>,
}

impl Texture {
    pub fn gpu(&self) -> &gpu::Texture {
        self.gpu.as_ref().unwrap()
    }
}

impl Asset for Texture {
    fn load(path: &Path) -> Result<Self, LoadError> {
        let file = fs::read_to_string(path).map_err(|_| LoadError::FileNotFound)?;
        let mut texture: Self = de::from_str(&file).map_err(|_| LoadError::DataError)?;

        let raw = fs::read(&texture.raw).map_err(|_| LoadError::FileNotFound)?;

        let raw_texture = match image::load_from_memory(&raw[..]) {
            LoadResult::Error(err) => {
                error!("Failed to load texture from file due to {}", err);
                return Err(LoadError::DataError);
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

        texture.gpu = Some(raw_texture);
        Ok(texture)
    }

    fn unload(_asset: Self) {
        
    }

    fn extension() -> &'static str {
        "tex"
    }
}