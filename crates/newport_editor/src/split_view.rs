use crate::{
    Id,
    ToId,
    Context,
    Layout,
    Builder,
    Sizing,
    ButtonResponse,
    button_control,
    DARK,
    Style,
};

use crate::math::{
    Vector2,
    Rect,
};

pub struct SplitView {
    id: Id,
    view: View,
}

impl SplitView {
    pub fn new(id: impl ToId) -> Self {
        let mut view = View::new("main_view", 1.0);
        view.add_tab(TestTab1);
        view.add_tab(TestTab2);
        view.add_tab(TestTab3);
        Self{
            id:   id.to_id(),
            view: view
        }
    }
}

impl SplitView {
    pub fn build(&mut self, ctx: &mut Context) {
        let bounds = ctx.take_canvas();
        let mut builder = ctx.builder(self.id, Layout::up_to_down(bounds));
        let style = builder.style();
        builder.painter.rect(bounds).color(style.inactive_background);

        let bounds = Rect::from_min_max(bounds.min + 5.0, bounds.max - 5.0);
        builder.layout(Layout::up_to_down(bounds), |builder| {
            self.view.build(builder);
        });

        builder.finish();
    }
}

enum ViewChildren {
    None,
    Views{
        view:      Vec<View>,
        direction: Direction,
    },
    Tabs{
        tabs:     Vec<Box<dyn Tab>>,
        selected: usize,
    }
}

pub enum Direction {
    Horizontal,
    Vertical,
}

pub struct View {
    id:        Id,
    children:  ViewChildren,
    percent:   f32,
}

impl View {
    pub fn new(id: impl ToId, percent: f32) -> Self {
        Self {
            id: id.to_id(),
            children: ViewChildren::None,
            percent:  percent,
        }
    }

    pub fn add_tab(&mut self, tab: impl Tab + 'static) {
        match &mut self.children {
            ViewChildren::None => {
                let mut tabs: Vec<Box<dyn Tab>> = Vec::with_capacity(1);
                tabs.push(Box::new(tab));
                self.children = ViewChildren::Tabs{
                    tabs:     tabs,
                    selected: 0,
                }
            },
            ViewChildren::Tabs { tabs, selected } => {
                tabs.push(Box::new(tab));
                *selected = tabs.len() - 1;
            },
            _ => unreachable!()
        }
    }
}

impl View {
    pub fn build(&mut self, builder: &mut Builder) {
        match &mut self.children {
            ViewChildren::None => {
                let mut style = builder.style();
                let og = style.clone();
                style.sizing = Sizing::Fill(true, true);
                builder.set_style(style);
                builder.label("Empty View");
                builder.set_style(og);
            },
            ViewChildren::Tabs { tabs, selected } => {
                let mut style = builder.style();
                style.margin = Rect::default();
                style.padding = (8.0, 5.0, 8.0, 5.0).into();
                style.focused_background = DARK.bg_s;
                style.focused_foreground = DARK.fg;
                builder.set_style(style.clone());

                let layout = Layout::left_to_right(builder.layout.push_size(Vector2::new(0.0, style.label_height_with_padding())));
                builder.layout(layout, |builder|{
                    fn menu_button(builder: &mut Builder, label: &str, selected: bool) -> ButtonResponse {
                        let style = builder.style();

                        let label_rect = style.string_rect(label, style.label_size, None).size();
                        let bounds = builder.content_bounds(label_rect);
                        
                        let id = Id::from(label);

                        let response = button_control(id, bounds, builder);
                        
                        let is_focused = selected;
                        let is_hovered = builder.is_hovered(id);
                        
                        let (background_color, foreground_color) = {
                            let background_color = if is_focused {
                                style.focused_background
                            } else if is_hovered {
                                style.hovered_background
                            } else {
                                style.unhovered_background
                            };

                            let foreground_color = if is_focused {
                                style.focused_foreground
                            } else if is_hovered {
                                style.hovered_foreground
                            } else {
                                style.unhovered_foreground
                            };

                            (background_color, foreground_color)
                        };

                        builder.painter.rect(bounds).color(background_color);
                        let at = Rect::from_pos_size(bounds.pos(), label_rect).top_left();
                        builder.painter
                            .text(label.to_string(), at, &style.font, style.label_size, builder.input().dpi)
                            .color(foreground_color)
                            .scissor(bounds);

                        response
                    }

                    for (index, it) in tabs.iter().enumerate() {
                        if menu_button(builder, it.name(), index == *selected).clicked() {
                            *selected = index;
                        }
                    }
                });

                let style = Style::default();
                builder.set_style(style.clone());

                let available_size = builder.available_rect().size() * self.percent;
                let bounds = builder.layout.push_size(available_size);
                builder.painter.rect(bounds).color(style.inactive_background);
                builder.layout(Layout::up_to_down(bounds), |builder| {
                    tabs[*selected].build(builder);
                });

            }
            _ => unimplemented!()
        }
    }
}

pub trait Tab {
    fn name(&self) -> &str;

    fn build(&mut self, builder: &mut Builder);
}

pub struct TestTab1;
impl Tab for TestTab1 {
    fn name(&self) -> &str {
        "Test1"
    }

    fn build(&mut self, builder: &mut Builder) {
        builder.label("Test1");
    }
}

pub struct TestTab2;
impl Tab for TestTab2 {
    fn name(&self) -> &str {
        "Test2"
    }

    fn build(&mut self, builder: &mut Builder) {
        builder.label("Test2");
    }
}

pub struct TestTab3;
impl Tab for TestTab3 {
    fn name(&self) -> &str {
        "Test3"
    }

    fn build(&mut self, builder: &mut Builder) {
        builder.label("Test3");
    }
}