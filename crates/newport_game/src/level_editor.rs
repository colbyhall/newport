use crate::editor::{
    Context,
    Page,
    Panel,
    DARK,
    Style,
};

use crate::math::Rect;

pub struct LevelEditor;

impl Page for LevelEditor {
    fn can_close(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "Level Editor"
    }

    fn show(&mut self, ctx: &mut Context) {
        let mut style = Style::default();
        style.padding = (8.0, 5.0, 8.0, 5.0).into();
        style.margin = Rect::default();
        // style.inactive_background = style.focused_background;
        style.unhovered_background = DARK.bg_s;
        ctx.set_style(style.clone());

        Panel::top("info_bar", style.label_height_with_padding()).build(ctx, |builder| {
            builder.button("File");
            builder.button("Edit");
            builder.button("Selection");
            builder.button("View");
            builder.button("Run");
            builder.button("Help");
        });
    }
}