use crate::graphics::FontCollection;
use crate::math::{ Color, Rect, Vector2 };
use crate::engine::Engine;
use crate::asset::{ AssetManager, AssetRef };
use crate::DARK;

#[derive(Clone)]
pub enum Sizing {
    MinMax(Vector2, Vector2),
    Fill(bool, bool)
}

#[derive(Clone)]
pub struct Style {
    pub inactive_background: Color,
    pub inactive_foreground: Color,

    pub unhovered_background: Color,
    pub unhovered_foreground: Color,

    pub hovered_background:   Color,
    pub hovered_foreground:   Color,

    pub focused_background:   Color,
    pub focused_foreground:   Color,

    pub selected_background:   Color,
    pub selected_foreground:   Color,

    pub font: AssetRef<FontCollection>,
    pub label_size: u32,
    pub header_size: u32,

    pub margin:  Rect,
    pub padding: Rect,

    pub sizing:  Sizing,
}

impl Style {
    pub fn string_rect(&self, string: &str, size: u32, wrap: Option<f32>) -> Rect {
        let mut fc = self.font.write();
        let font = fc.font_at_size(size, 1.0).unwrap(); // NOTE: I don't think DPI matters here

        font.string_rect(string, wrap.unwrap_or(f32::INFINITY))
    }

    pub fn content_size(&self, mut needed: Vector2, available: Vector2) -> Vector2 {
        needed += self.padding.min + self.padding.max;

        match self.sizing {
            Sizing::MinMax(min, max) => {
                let needed = needed.max(min);
                needed.min(max)
            },
            Sizing::Fill(width, height) => {
                if width {
                    needed.x = available.x - self.margin.width();
                }

                if height {
                    needed.y = available.y - self.margin.height();
                }

                needed
            }
        }
    }

    pub fn spacing_size(&self, size: Vector2) -> Vector2 {
        size + self.margin.min + self.margin.max
    }

    pub fn label_height(&self) -> f32 {
        let mut fc = self.font.write();
        let font = fc.font_at_size(self.label_size, 1.0).unwrap(); // NOTE: I don't think DPI matters here
        font.ascent - font.descent
    }

    pub fn label_height_with_padding(&self) -> f32 {
        let height = self.label_height();
        height + self.padding.min.y + self.padding.max.y
    }
}

impl Default for Style {
    fn default() -> Self {
        let asset_manager = Engine::as_ref().module::<AssetManager>().unwrap();

        let font = asset_manager.find("assets/fonts/consola.ttf").unwrap();

        Self {
            inactive_background: DARK.bg_s,
            inactive_foreground: DARK.fg,

            unhovered_background: DARK.bg,
            unhovered_foreground: DARK.fg,

            hovered_background: DARK.bg1,
            hovered_foreground: DARK.fg2,

            focused_background: DARK.bg2,
            focused_foreground: DARK.fg3,

            selected_background: DARK.blue1,
            selected_foreground: DARK.bg1,

            font: font,
            label_size:  10,
            header_size: 14,

            margin:  (5.0, 5.0, 5.0, 5.0).into(),
            padding: (5.0, 5.0, 5.0, 5.0).into(),

            sizing:  Sizing::MinMax(-Vector2::INFINITY, Vector2::INFINITY)
        }
    }
}