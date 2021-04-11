use crate::{ Id, Painter, Context, Layout, widgets::Widget, Response };

pub struct GUI<'a> {
    pub(crate) id: Id,

    pub(crate) layout: Layout,

    pub(crate) painter: Painter,

    pub(crate) context: &'a mut Context,
}

impl<'a> GUI<'a> {
    pub fn new(id: Id, layout: Layout, context: &'a mut Context) -> Self {
        Self{
            id:      id,
            layout:  layout,
            painter: Painter::new(),
            context: context,
        }
    }

    pub fn add(&mut self, widget: impl Widget) -> Response {
        widget.gui(self)
    }
}