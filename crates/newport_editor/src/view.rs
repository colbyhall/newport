use crate::{
    Id,
    ToId,
    Layout,
    Builder,
    Sizing,
    ButtonResponse,
    button_control,
    DARK,
    Style,
    Direction,
    LayoutStyle,
    ColorStyle,
    TextStyle,
    Shape,
};

use crate::math::{
    Vector2,
    Rect,
};

pub const SPACING: f32 = 5.0;

enum ViewChildren {
    None,
    Views {
        views: Vec<View>,
        direction: Direction,
    },
    Tabs{
        tabs:      Vec<Box<dyn Tab>>,
        selected:  usize,
        hide_tabs: bool,
    }
}

pub struct View {
    _id:        Id,
    children:  ViewChildren,
    percent:   f32,
}

impl View {
    pub fn new(id: impl ToId, percent: f32) -> Self {
        Self {
            _id: id.to_id(),
            children: ViewChildren::None,
            percent:  percent,
        }
    }

    pub fn new_views(id: impl ToId, percent: f32, views: Vec<View>, direction: Direction) -> Self {
        Self {
            _id: id.to_id(),
            children: ViewChildren::Views{ views, direction },
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
                    hide_tabs: false,
                }
            },
            ViewChildren::Tabs { tabs, selected, .. } => {
                tabs.push(Box::new(tab));
                *selected = tabs.len() - 1;
            },
            _ => unreachable!()
        }
    }

    pub fn add_view(&mut self, view: View) {
        match &mut self.children {
            ViewChildren::None => {
                let mut views = Vec::with_capacity(1);
                views.push(view);
                self.children = ViewChildren::Views{
                    views: views,
                    direction: Direction::LeftToRight,
                }
            },
            ViewChildren::Views { views, .. } => {
                views.push(view);
            },
            _ => unreachable!()
        }
    }

    pub fn hide_tabs(&mut self, hide: bool)  {
        match &mut self.children {
            ViewChildren::Tabs { hide_tabs, .. } => {
                *hide_tabs = hide;
            },
            _ => unreachable!()
        }
    }
}

impl View {
    pub fn build(&mut self, builder: &mut Builder) {
        match &mut self.children {
            ViewChildren::None => {
                let mut layout_style = LayoutStyle::default();
                layout_style.width_sizing = Sizing::Fill;
                layout_style.height_sizing = Sizing::Fill;
                builder.scoped_style(layout_style, |builder| builder.label("Empty View"));
            },
            ViewChildren::Tabs { tabs, selected, hide_tabs } => {
                let mut layout_style: LayoutStyle = builder.style().get();
                layout_style.margin = Rect::default();
                layout_style.padding = (8.0, 5.0, 8.0, 5.0).into();
                if !*hide_tabs {
                    builder.style().push(layout_style);
    
                    let mut color: ColorStyle = builder.style().get();
                    color.focused_background = DARK.bg_s;
                    color.focused_foreground = DARK.fg;
                    builder.style().push(color);
    
                    let height = builder.label_height_with_padding();
                    let layout = Layout::left_to_right(builder.layout.push_size(Vector2::new(0.0, height)));
                    builder.layout(layout, |builder|{
                        for (index, it) in tabs.iter().enumerate() {
                            if TabLabel::new(it.name(), index == *selected).build(builder).clicked() {
                                *selected = index;
                            }
                        }
                    });
    
                    builder.style().pop::<LayoutStyle>();
                    builder.style().pop::<ColorStyle>();
                }

                layout_style.padding = (8.0, 8.0, 8.0, 8.0).into();
                builder.style().push(layout_style);

                let available_size = builder.available_rect().size();
                
                let bounds = builder.layout.push_size(available_size);
                let color: ColorStyle = builder.style().get();
                builder.painter.push_shape(Shape::solid_rect(bounds, color.inactive_background, 0.0));

                let mut color: ColorStyle = builder.style().get();
                color.inactive_background = DARK.bg_h;

                builder.scoped_style(color, |builder| {
                    let bounds = Rect::from_min_max(bounds.min + SPACING, bounds.max - SPACING);
                    builder.layout(Layout::up_to_down(bounds), |builder| {
                        tabs[*selected].build(builder);
                    });
                });

                builder.style().pop::<LayoutStyle>();
            }
            ViewChildren::Views { views, direction } => {
                let available_size = builder.available_rect().size();
                let bounds = builder.layout.push_size(available_size);
                
                let layout = Layout::new(bounds, *direction);

                builder.layout(layout, |builder| {
                    for it in views {
                        let size = builder.layout.bounds().size() * it.percent - SPACING / 2.0;
                        let bounds = builder.layout.push_size(size);

                        builder.layout(Layout::up_to_down(bounds), |builder| {
                            it.build(builder);
                        });
                        builder.add_spacing(SPACING);
                    }
                });
            },
        }
    }
}

pub trait Tab {
    fn name(&self) -> String;

    fn build(&mut self, builder: &mut Builder);
}

pub struct TestTab(pub i32);
impl Tab for TestTab {
    fn name(&self) -> String {
        format!("Test {}", self.0)
    }

    fn build(&mut self, builder: &mut Builder) {
        builder.label(self.name());
    }
}

pub struct TabLabel {
    id:   Id,
    name: String,
    selected: bool,
}

impl TabLabel {
    pub fn new(name: impl Into<String>, selected: bool) -> Self {
        let name = name.into();
        Self {
            id: name.to_id(),
            name: name,
            selected: selected
        }
    }

    pub fn build(self, builder: &mut Builder) -> ButtonResponse {
        let color: ColorStyle = builder.style().get();
        let text: TextStyle = builder.style().get();

        let label_rect = text.string_rect(&self.name, text.label_size, None).size();
        let bounds = builder.content_bounds(label_rect);
        
        let response = button_control(self.id, bounds, builder);
        
        let is_focused = self.selected;
        let is_hovered = builder.is_hovered(self.id);
        
        let (background_color, foreground_color) = {
            let background_color = if is_focused {
                color.focused_background
            } else if is_hovered {
                color.hovered_background
            } else {
                color.unhovered_background
            };

            let foreground_color = if is_focused {
                color.focused_foreground
            } else if is_hovered {
                color.hovered_foreground
            } else {
                color.unhovered_foreground
            };

            (background_color, foreground_color)
        };

        builder.painter.push_shape(Shape::solid_rect(bounds, background_color, 0.0));
        
        let at = Rect::from_pos_size(bounds.pos(), label_rect).top_left();
        builder.painter.push_shape(
            Shape::text(
                self.name, 
                at, 
                &text.font, 
                text.label_size, 
                builder.input().dpi, 
                foreground_color
            )
        );

        response
    }
}