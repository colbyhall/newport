use crate::{ Painter, Context, Id, Layout };

pub struct Builder<'a> {
    pub id:      Id,
    pub layout:  Layout,

    pub painter: Painter,
    pub context: &'a mut Context,
}

impl<'a> Builder<'a> {
    pub fn finish(self) {
        self.context.push_layer(self.id, self.painter)
    }
}