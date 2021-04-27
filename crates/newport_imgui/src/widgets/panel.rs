use crate::{
    Id,
    Context,
    ToId,
    Builder,
    Layout,
};

pub enum PanelVariant {
    Top,
    Bottom,
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

    pub fn bottom(id: impl ToId, size: f32) -> Self {
        Self {
            id:      id.to_id(),
            variant: PanelVariant::Bottom,
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
            PanelVariant::Bottom => {
                let rect = ctx.split_canvas_bottom(self.size);
                Layout::left_to_right(rect)
            },
        };

        let mut builder = ctx.builder(self.id, layout);
        let style = builder.style();
        builder.painter.rect(layout.bounds()).color(style.inactive_background);
        contents(&mut builder);
        builder.finish();
    }
}