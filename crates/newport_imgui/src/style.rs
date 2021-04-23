use crate::graphics::FontCollection;
use crate::math::{ Color, Rect };
use crate::engine::Engine;
use crate::asset::{ AssetManager, AssetRef };
use crate::DARK;

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
}

impl Style {
    pub fn string_rect(&self, string: &str, size: u32, wrap: Option<f32>) -> Rect {
        let mut fc = self.font.write();
        let font = fc.font_at_size(size, 1.0).unwrap(); // NOTE: I don't think DPI matters here

        font.string_rect(string, wrap.unwrap_or(f32::INFINITY))
    }
}

impl Default for Style {
    fn default() -> Self {
        let asset_manager = Engine::as_ref().module::<AssetManager>().unwrap();

        let font = asset_manager.find("assets/fonts/menlo_regular.ttf").unwrap();

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
            label_size:  12,
            header_size: 16
        }
    }
}