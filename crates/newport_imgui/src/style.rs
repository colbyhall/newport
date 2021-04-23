use std::any::{ Any, TypeId };
use std::collections::HashMap;

use crate::graphics::FontCollection;
use crate::math::{ Rect, Vector2, Color };
use crate::engine::Engine;
use crate::asset::{ AssetManager, AssetRef };
use crate::{ Builder, DARK };

pub trait Style = 'static + Default + Clone + Any;

pub struct StyleMap {
    inner: HashMap<TypeId, Box<dyn Any>>,
}

impl StyleMap {
    pub fn new() -> Self {
        Self {
            inner: HashMap::with_capacity(16)
        }
    }

    pub fn get<T: Style>(&mut self) -> T {
        let id = TypeId::of::<T>();

        if !self.inner.contains_key(&id) {
            self.inner.insert(id, Box::new(vec![T::default()]));
        }
        self.inner.get(&id).unwrap().downcast_ref::<Vec<T>>().unwrap().last().unwrap().clone()
    }

    pub fn push<T: Style>(&mut self, t: T) {
        let id = TypeId::of::<T>();

        if self.inner.contains_key(&id) {
            self.inner.insert(id, Box::new(vec![T::default()]));
        }
        self.inner.get_mut(&id).unwrap().downcast_mut::<Vec<T>>().unwrap().push(t);
    }

    pub fn pop<T: Style>(&mut self) {
        let id = TypeId::of::<T>();

        if self.inner.contains_key(&id) {
            self.inner.insert(id, Box::new(vec![T::default()]));
            return;
        }

        let vec = self.inner.get_mut(&id).unwrap().downcast_mut::<Vec<T>>().unwrap();
        if vec.len() > 1 {
            vec.pop();
        }
    }
}

#[derive(Clone, Copy)]
pub struct Padding(pub Rect);

impl Default for Padding {
    fn default() -> Self {
        Self((5.0, 5.0, 5.0, 5.0).into())
    }
}

#[derive(Clone, Copy)]
pub struct Margin(pub Rect);

impl Default for Margin {
    fn default() -> Self {
        Self((2.0, 2.0, 2.0, 2.0).into())
    }
}

#[derive(Copy, Clone)]
pub enum Alignment {
    Left,
    Center,
    Right,
}

#[derive(Clone)]
pub struct TextStyle {
    pub font: AssetRef<FontCollection>,
    pub size: u32,
    pub alignment: Alignment,
    pub color: Color,
}

#[derive(Clone)]
pub struct LabelStyle(pub TextStyle);

impl LabelStyle {
    pub fn min_size(builder: &mut Builder) -> Vector2 {
        let padding = builder.style().get::<Padding>();
        let margin  = builder.style().get::<Margin>();
        let style   = builder.style().get::<LabelStyle>();
        
        let mut fc = style.0.font.write();
        let font = fc.font_at_size(style.0.size, builder.input().dpi).unwrap();
        Vector2::new(
            padding.0.min.x + padding.0.max.x + margin.0.min.x + margin.0.max.x + padding.0.min.x + padding.0.max.x + margin.0.min.x + margin.0.max.x, 
            font.height + padding.0.min.y + padding.0.max.y + margin.0.min.y + margin.0.max.y + padding.0.min.x + padding.0.max.x + margin.0.min.x + margin.0.max.x
        )
    }

    pub fn string_rect(builder: &mut Builder, label: &str) -> Rect {
        let style = builder.style().get::<LabelStyle>();

        let mut fc = style.0.font.write();
        let font = fc.font_at_size(style.0.size, builder.input().dpi).unwrap();

        font.string_rect(label, builder.layout.space_left().x)
    }
}

impl Default for LabelStyle {
    fn default() -> Self {
        let engine = Engine::as_ref();
        let asset_manager = engine.module::<AssetManager>().unwrap();

        let font = asset_manager.find("assets/fonts/menlo_regular.ttf").unwrap();

        Self(TextStyle{
            font: font,
            size: 12,
            alignment: Alignment::Center,
            color: DARK.fg,
        })
    }
}