use newport_gpu::*;
use newport_math::*;
use newport_asset::*;
use newport_engine::Engine;

use crate::Graphics;

use std::{mem::size_of, thread_local};
use std::collections::HashMap;
use std::fs;

use freetype::{ Library, Face };
use freetype::face::LoadFlag;

thread_local! {
    static FREETYPE_LIB: Library = Library::init().unwrap();
}

pub struct FontCollection {
    face:  Face,
    fonts: HashMap<u32, Font>,
}

impl FontCollection {
    pub const NUM_GLYPHS : usize = 512;

    pub fn new(file: Vec<u8>) -> Result<FontCollection, ()> {
        let face = FREETYPE_LIB.with(|lib| {
            let face = lib.new_memory_face(file, 0);
            if face.is_err() {
                return Err(());
            }
            Ok(face.unwrap())
        })?;

        Ok(FontCollection{
            face: face,
            fonts: HashMap::new(),
        })
    }

    pub fn font_at_size(&mut self, size: u32, dpi: f32) -> Option<&Font> {
        let font = self.fonts.get(&size);
        if font.is_none() {
            let resolution = (96.0 * dpi) as u32;
    
            // NOTE: Most of this was copied from https://gist.github.com/baines/b0f9e4be04ba4e6f56cab82eef5008ff#file-freetype-atlas-c-L17
            self.face.set_char_size(0, (size << 6) as _, resolution, resolution).ok()?;
    
            let size_metrics = self.face.size_metrics()?;
            let max_dim = ((size_metrics.height >> 6) + 1) * (Self::NUM_GLYPHS as f32).sqrt().ceil() as i32;
            let mut tex_width = 1;
            while tex_width < max_dim { tex_width <<= 1 };
            let tex_height = tex_width;
    
            let mut glyphs = Vec::new(); 
            glyphs.resize(Self::NUM_GLYPHS, Glyph::default());
            let mut pixels = Vec::new();
            pixels.resize((tex_width * tex_height) as usize, 0);
            let mut pen_x = 0;
            let mut pen_y = tex_height - 1;
    
            for i in 0..Self::NUM_GLYPHS {
                self.face.load_char(i, LoadFlag::RENDER | LoadFlag::FORCE_AUTOHINT | LoadFlag::TARGET_LIGHT).unwrap();
                let glyph       = self.face.glyph();
                let bmp         = glyph.bitmap();
                let bmp_width   = bmp.width();
                let bmp_rows    = bmp.rows();
                let bmp_pitch   = bmp.pitch();
                let bmp_buffer  = bmp.buffer();
    
                if pen_x + bmp_width >= tex_width {
                    pen_x = 0;
                    pen_y -= (size_metrics.height >> 6) + 1;
                }
    
                for row in 0..bmp_rows {
                    for col in 0..bmp_width {
                        let x = pen_x + col;
                        let y = pen_y - row;

                        let alpha = bmp_buffer[(row * bmp_pitch + col) as usize];
                        let color: u32 = 0xFFFFFF00 | (alpha as u32);

                        pixels[(y * tex_width + x) as usize] = color;
                    }
                }
    
                let x0 = pen_x;
                let y0 = pen_y - bmp_rows;
                let x1 = pen_x + bmp_width;
                let y1 = pen_y + 2;

                let width = (x1 - x0) as f32;
                let height = (y1 - y0) as f32;
    
                let uv0 = Vector2::new(x0 as f32 / tex_width as f32, y0 as f32 / tex_height as f32);
                let uv1 = Vector2::new(x1 as f32 / tex_width as f32, y1 as f32 / tex_height as f32);
    
                glyphs[i] = Glyph {
                    width: width,
                    height: height,
    
                    bearing_x:  (glyph.bitmap_left()) as _,
                    bearing_y:  (glyph.bitmap_top()) as _,
                    advance:    (glyph.advance().x >> 6) as _,
    
                    uv: (uv0, uv1).into(),
                };
    
                pen_x += bmp_width + 1;
            }
    
            let ascent  = (self.face.ascender() >> 6) as f32;
            let descent = (self.face.descender() >> 6) as f32;
            let height  = (size_metrics.height >> 6) as f32;
    
            let scale   = height / (ascent - descent);
            let ascent  = ascent * scale;
            let descent = descent * scale;
    
            let graphics = Engine::as_ref().module::<Graphics>()?;
            let device   = graphics.device();
    
            let pixel_buffer = device.create_buffer(
                BufferUsage::TRANSFER_SRC, 
                MemoryType::HostVisible, 
                pixels.len() * size_of::<u32>()
            ).ok()?;
            pixel_buffer.copy_to(&pixels[..]);
    
            let atlas = device.create_texture(
                TextureUsage::TRANSFER_DST | TextureUsage::SAMPLED,
                MemoryType::DeviceLocal, 
                Format::RGBA_U8,
                tex_width as u32,
                tex_height as u32,
                1,
                Wrap::Clamp,
                Filter::Nearest,
                Filter::Nearest
            ).ok()?;
    
            let mut gfx = device.create_graphics_context().ok()?;
            {
                gfx.begin();
                gfx.resource_barrier_texture(&atlas, Layout::Undefined, Layout::TransferDst);
                gfx.copy_buffer_to_texture(&atlas, &pixel_buffer);
                gfx.resource_barrier_texture(&atlas, Layout::TransferDst, Layout::ShaderReadOnly);
                gfx.end();
            }
    
            let receipt = device.submit_graphics(vec![gfx], &[]);
            receipt.wait();
    
            self.fonts.insert(size, Font{
                size,
    
                ascent,
                descent,
                height,
    
                glyphs,
                atlas
            });
        }

        self.fonts.get(&size)
    }
}

impl Asset for FontCollection {
    fn load(path: &Path) -> Result<Self, LoadError> {
        let font_file = fs::read(path).map_err(|_| LoadError::FileNotFound)?;
        FontCollection::new(font_file).map_err(|_| LoadError::DataError)
    }

    fn extension() -> &'static str {
        "ttf"
    }
}

pub struct Font {
    pub size: u32,

    pub ascent:   f32,
    pub descent:  f32,
    pub height:   f32,

    pub glyphs: Vec<Glyph>,
    pub atlas:  Texture,
}

impl Font {
    pub fn glyph_from_char(&self, c: char) -> Option<&Glyph> {
        self.glyphs.get(c as usize)
    }

    pub fn string_rect(&self, s: &str, max_width: f32) -> Rect {
        let mut height = self.height;
        let mut width = 0.0;
        let mut x = 0.0;
        for c in s.chars() {
            let space_glyph = self.glyph_from_char(' ').unwrap();
            if x + space_glyph.advance > max_width { 
                x = 0.0; 
                height += self.height;
            }

            match c {
                '\n' => {
                    x = 0.0;
                    height += self.height;
                    continue;
                },
                '\r' => {
                    x = 0.0;
                    continue;
                },
                '\t' => {
                    x += space_glyph.advance * 4.0;
                },
                _ => {
                    let g = self.glyph_from_char(c);
                    if g.is_none() {
                        x += self.glyph_from_char('?').unwrap().advance;
                    } else {
                        x += g.unwrap().advance;
                    }
                },
            }
            if x > width { width = x; }
        }

        let min = Vector2::new(0.0, -height);
        let max = Vector2::new(width, 0.0);
        Rect::from_min_max(min, max)
    }
}

#[derive(Default, Clone, Copy, Debug)]
pub struct Glyph {
    pub width: f32,
    pub height: f32,

    pub bearing_x: f32,
    pub bearing_y: f32,
    pub advance:   f32,

    pub uv: Rect,
}