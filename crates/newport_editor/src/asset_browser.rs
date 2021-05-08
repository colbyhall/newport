use crate::{
    Tab,
    Builder,

    Layout,
    Scrollbox,
    Direction,
    LayoutStyle,

    ToId,
    Retained,
    Id,

    ButtonResponse,
    button_control,

    TextStyle,
    ColorStyle,

    Shape,
    Sizing,

    engine::Engine,
    asset::AssetManager,
    math::{ Rect, Vector2 },
};

use std::{
    path::Path,
};

#[derive(Debug)]
enum BrowserEntry {
    Directory{
        path:    String,
        entries: Vec<BrowserEntry>,
        id: u64,
    },
    Asset(String),
}

impl BrowserEntry {
    fn insert(&mut self, path: &Path, id: &mut u64) {
        let root = match path.iter().next() {
            Some(root) => root,
            None => return,
        }.to_str().unwrap();

        match self {
            BrowserEntry::Directory { entries, .. } => {
                let found = entries.iter_mut().find(|it| {
                    match it {
                        BrowserEntry::Directory { path, .. } => &root == path,
                        BrowserEntry::Asset(_) => false,
                    }
                });

                let is_file = root.contains('.');

                match found {
                    Some(found) => {
                        if !is_file {
                            found.insert(Path::new(path.strip_prefix(root).unwrap()), id);
                        }
                    },
                    None => {
                        if is_file {
                            entries.push(BrowserEntry::Asset(root.to_string()));
                        } else {
                            entries.push(BrowserEntry::Directory{ path: root.to_string(), entries: Vec::new(), id: *id });
                            *id += 1;
                            entries.last_mut().unwrap().insert(Path::new(path.strip_prefix(root).unwrap()), id);
                        }
                    },
                }
            },
            BrowserEntry::Asset(_) => {
                return;
            }
        }
    }
}

pub struct AssetBrowser {
    entries: BrowserEntry,
    selected_entry: u64,
}

impl AssetBrowser {
    pub fn new() -> Self {
        let asset_manager = Engine::as_ref().module::<AssetManager>().unwrap();

        let mut entries = BrowserEntry::Directory{
            path: "Assets".into(),
            entries: Vec::new(),
            id: 0,
        };

        let assets = asset_manager.assets();
        let mut id = 0;
        for entry in assets.iter() {
            let path = &entry.path;
            entries.insert(path, &mut id);
        }

        Self {
            entries: entries,
            selected_entry: 0,
        }
    }
}

impl Tab for AssetBrowser {
    fn name(&self) -> String {
        "Asset Browser".to_string()
    }

    fn build(&mut self, builder: &mut Builder) {
        builder.layout(Layout::left_to_right(builder.layout.bounds()), |builder| {
            let bounds = builder.layout.bounds();

            let size = (300.0, bounds.height()).into();
            Scrollbox::new("test", builder.layout.push_size(size), Direction::UpToDown)
            .build(builder, |builder| {
                let mut layout_style: LayoutStyle = builder.style().get();
                layout_style.width_sizing = Sizing::Fill;
                builder.scoped_style(layout_style, |builder| {
                    fn build_entry(builder: &mut Builder, entry: &BrowserEntry, selected: &mut u64) {
                        match entry {
                            BrowserEntry::Directory{ path, entries, id } => {
                                let entry = SelectableCollapsingEntry::new(id, path, *id == *selected);

                                if entry.build(builder, entries.len() > 0, |builder| {
                                    for entry in entries.iter() {
                                        build_entry(builder, entry, selected);
                                    }
                                }).clicked() {
                                    *selected = *id;
                                }
                            },
                            _ => return
                        }
                    }

                    match &self.entries {
                        BrowserEntry::Directory{ entries, .. } => {
                            for entry in entries.iter() {
                                build_entry(builder, entry, &mut self.selected_entry);
                            }
                        },
                        _ => {}
                    }

                });
            });
        });
    }
}

#[derive(Clone)]
struct SelectableCollapsingRetained {
    is_open: bool,
}

impl Default for SelectableCollapsingRetained {
    fn default() -> Self {
        Self{
            is_open: true,
        }
    }
}

impl Retained for SelectableCollapsingRetained { }

struct SelectableCollapsingEntry {
    id:       Id,
    label:    String,
    selected: bool,
}

impl SelectableCollapsingEntry {
    fn new(id: impl ToId, label: impl Into<String>, selected: bool) -> Self {
        Self {
            id: id.to_id(),
            label: label.into(),
            selected,
        }
    }
}

impl SelectableCollapsingEntry {
    fn build(self, builder: &mut Builder, has_contents: bool, contents: impl FnOnce(&mut Builder)) -> ButtonResponse {
        let mut retained = builder.retained::<SelectableCollapsingRetained>(self.id);

        let layout_style: LayoutStyle = builder.style().get();
        let color: ColorStyle = builder.style().get();
        let text: TextStyle = builder.style().get();

        let label_rect = text.string_rect(&self.label, text.label_size, None).size();
        let bounds = builder.content_bounds(label_rect);

        let collapse_button_size: Vector2 = (10.0, 10.0).into();

        let mut cursor_x = bounds.min.x + layout_style.padding.min.x;

        let collapse_button_bounds = if has_contents {
            let min = Vector2::new(cursor_x, bounds.min.y + bounds.height() / 2.0 - collapse_button_size.y / 2.0);
            let max = min + collapse_button_size;
            cursor_x += collapse_button_size.x + 5.0;
            Rect::from_min_max(min, max)
        } else {
            Rect::default()
        };

        let toggle_id = (self.id, builder as *mut Builder).to_id();
        if button_control(toggle_id, collapse_button_bounds, builder).clicked() && has_contents {
            retained.is_open = !retained.is_open;
            builder.set_retained(self.id, retained.clone());
        }

        let response = button_control(self.id, bounds, builder);

        let is_focused = builder.is_focused(self.id);
        let is_hovered = builder.is_hovered(self.id);
        
        let (background_color, foreground_color) = {
            let background_color = if self.selected {
                color.selected_background
            } else if is_focused {
                color.focused_background
            } else if is_hovered {
                color.hovered_background
            } else {
                color.unhovered_background
            };

            let foreground_color = if self.selected {
                color.selected_foreground
            } else if is_focused {
                color.focused_foreground
            } else if is_hovered {
                color.hovered_foreground
            } else {
                color.unhovered_foreground
            };

            (background_color, foreground_color)
        };

        builder.painter.push_shape(Shape::solid_rect(bounds, background_color, 0.0));

        let roundness = if retained.is_open {
            0.0
        } else {
            3.0
        };

        if has_contents {
            builder.painter.push_shape(Shape::solid_rect(collapse_button_bounds, color.inactive_foreground, roundness));
        }

        let at = Vector2::new(cursor_x, Rect::from_pos_size(bounds.pos(), label_rect).top_left().y);
        builder.painter.push_shape(
            Shape::text(
                self.label, 
                at, 
                &text.font, 
                text.label_size, 
                builder.input().dpi, 
                foreground_color
            )
        );

        if retained.is_open && has_contents {
            let mut layout_style: LayoutStyle = builder.style().get();
            layout_style.padding.min.x += 10.0;
            builder.scoped_style(layout_style, contents);
        }

        response
    }
}