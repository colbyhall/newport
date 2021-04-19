use crate::{ Retained, Id, Builder, Painter, Layout, Input, Mesh };

use std::collections::HashMap;

struct Layer {
    id:      Id,
    painter: Painter,
}

pub struct Context {
    input:      Option<Input>,
    layers:     Vec<Layer>,
    retained:   HashMap<Id, Box<dyn Retained>>,
}

impl Context {
    pub fn new() -> Self {
        Self {
            input:      None,
            layers:     Vec::with_capacity(32),
            retained:   HashMap::with_capacity(128),
        }
    }

    pub fn builder(&mut self, id: Id, layout: Layout) -> Builder {
        Builder{
            id:     id,
            layout: layout,

            painter: Painter::new(),
            context: self,
        }
    }

    pub(crate) fn push_layer(&mut self, id: Id, painter: Painter) {
        self.layers.push(Layer{ id, painter });
    }

    pub fn begin_frame(&mut self, input: Input) {
        self.input = Some(input);
    }

    pub fn end_frame(&mut self) -> Mesh {
        let mut mesh = Mesh{
            vertices: Vec::with_capacity(2048),
            indices: Vec::with_capacity(2048),
        };

        self.layers.drain(..).for_each(|it| it.painter.tesselate(&mut mesh));
        
        mesh
    }

    pub fn input(&self) -> &Input {
        self.input.as_ref().unwrap()
    }
}