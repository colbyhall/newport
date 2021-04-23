use crate::{
    Id,
    Context,
    ToId,
    Builder,
    Layout,
    DARK,
};

pub enum PanelVariant {
    Top,
}

pub struct Panel {
    id:      Id,
    variant: PanelVariant,
    size:    f32,
}

impl Panel {
    pub fn top(id: impl ToId, size: f32) -> Self {
        Self {
            id:      id.to_id(),
            variant: PanelVariant::Top,
            size:    size,
        }
    }
}

impl Panel {
    pub fn build(self, ctx: &mut Context, contents: impl FnOnce(&mut Builder)) {
        let layout = match self.variant {
            PanelVariant::Top => {
                let rect = ctx.split_canvas_top(self.size);
                Layout::left_to_right(rect)
            },
        };

        let mut builder = ctx.builder(self.id, layout);
        builder.painter.rect(layout.bounds()).color(DARK.bg);
        contents(&mut builder);
        builder.finish();
    }
}