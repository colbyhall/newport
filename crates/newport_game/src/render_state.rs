use crate::{
    Viewport,
    ViewportId,

    math,
    gpu::GraphicsContext,
};

use math::Matrix4;

use std::{
    collections::HashMap,
};

pub struct RenderState {
    pub viewports: HashMap<ViewportId, Viewport>,

    pub primitives: Vec<Box<dyn Primitive>>,
    pub primitive_transforms: Vec<Matrix4>,
}

pub trait Primitive {
    fn record(&self, index: usize, gfx: &mut GraphicsContext);
}