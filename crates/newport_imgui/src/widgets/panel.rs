use crate::{
    Id,
    Context,
    ToId,
    Builder,
    Layout,
    ColorStyle,
    Shape,
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
        let color: ColorStyle = builder.style().get();
        builder.painter.push_shape(Shape::solid_rect(layout.bounds(), color.inactive_background, 0.0));
        contents(&mut builder);
        builder.finish();
    }
}