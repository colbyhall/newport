use crate::{ 
    Painter, 
    Context, 
    Id, 
    Layout, 
    InputState, 
    Button, 
    ButtonResponse, 
    Style,
    Label,
    Spacing
};

pub struct Builder<'a> {
    pub id:      Id,
    pub layout:  Layout,

    pub painter: Painter,
    pub context: &'a mut Context,
}

impl<'a> Builder<'a> {
    pub fn finish(self) {
        self.context.push_layer(self.painter)
    }

    pub fn input(&self) -> &InputState {
        &self.context.input
    }

    pub fn is_focused(&self, id: Id) -> bool {
        match &self.context.focused {
            Some(focused) => *focused == id,
            None => false
        }
    }

    pub fn is_hovered(&self, id: Id) -> bool {
        match &self.context.hovered {
            Some(hovered) => *hovered == id,
            None => false
        }
    }

    pub fn focus(&mut self, id: Id) -> bool {
        if self.context.focused.is_none() {
            self.context.focused = Some(id);
            return true;
        }
        false
    }

    pub fn force_focus(&mut self, id: Id) {
        self.context.focused = Some(id);
    }

    pub fn unfocus(&mut self, id: Id) -> bool{
        if self.is_focused(id) {
            self.context.focused = None;
            return true;
        }
        false
    }

    pub fn hover(&mut self, id: Id) {
        self.context.hovered = Some(id);
    }

    pub fn unhover(&mut self, id: Id) -> bool {
        if self.is_hovered(id) {
            self.context.hovered = None;
            return true;
        }
        false
    }

    pub fn button(&mut self, label: impl Into<String>) -> ButtonResponse {
        Button::new(label).build(self)
    }

    pub fn label(&mut self, label: impl Into<String>){
        Label::new(label).build(self)
    }

    pub fn layout(&mut self, layout: Layout, content: impl FnOnce(&mut Builder)) {
        let current = self.layout;
        self.layout = layout;
        content(self);
        self.layout = current;
    }

    pub fn style(&self) -> Style {
        match &self.context.style {
            Some(it) => it.clone(),
            None => Style::default(),
        }
    }

    pub fn spacing(&self) -> Spacing {
        self.context.spacing.clone()
    }
}