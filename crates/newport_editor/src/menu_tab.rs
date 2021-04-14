use crate::*;

pub struct MenuTab {
    selected:  bool,
    text:      String,
}

impl MenuTab {
    pub fn new(selected: bool, text: impl Into<String>) -> Self {
        Self { 
            selected: selected, 
            text: text.into() 
        }
    }
}

impl Widget for MenuTab {
    fn ui(self, ui: &mut Ui) -> Response {
        let Self { selected, text, .. } = self;

        let button_padding = ui.spacing().button_padding;
        let total_extra = button_padding + button_padding;

        let text_style = TextStyle::Button;
        let galley = if ui.wrap_text() {
            ui.fonts()
                .layout_multiline(text_style, text, ui.available_width() - total_extra.x)
        } else {
            ui.fonts().layout_no_wrap(text_style, text)
        };

        let mut desired_size = galley.size + 2.0 * button_padding;
        desired_size.y = desired_size.y.at_least(ui.spacing().interact_size.y);
        let (rect, response) = ui.allocate_at_least(desired_size, Sense::click());
        response.widget_info(|| {
            WidgetInfo::selected(WidgetType::SelectableLabel, selected, &galley.text)
        });

        let text_cursor = ui
            .layout()
            .align_size_within_rect(galley.size, rect.shrink2(button_padding))
            .min;

        let visuals = ui.style().visuals.widgets.active;

        if selected || response.hovered() {
            let rect = rect.expand(visuals.expansion);
            ui.painter()
                .rect(rect, 0.0, visuals.bg_fill, visuals.bg_stroke);
            
                if selected {
                    let rect = Rect::from_min_max(pos2(rect.min.x, rect.max.y - 2.0), rect.max);
                    ui.painter()
                        .rect(rect, 0.0, visuals.fg_stroke.color, visuals.bg_stroke);
            }
        } 

        ui.painter().galley(text_cursor, galley, visuals.fg_stroke.color);
        response
    }
}