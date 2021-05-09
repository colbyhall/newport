use newport_editor::{
    Tab,
    Builder,

    LayoutStyle,
    TextStyle,
    Sizing,

    TextEdit,
    Layout,
    Scrollbox,
    Direction,
    Alignment,

    SPACING,
};

use crate::{
    Game,

    engine::Engine,

    Named,
};

use std::cmp::Ordering;

pub struct Outliner {
    search_string: String,
}

impl Outliner {
    pub fn new() -> Self {
        Self{
            search_string: String::new(),
        }
    }
}

impl Tab for Outliner {
    fn name(&self) -> String {
        "Outliner".to_string()
    }

    fn build(&mut self, builder: &mut Builder) {
        let game = Engine::as_ref().module::<Game>().unwrap();
        let game_state = game.game_state.lock().unwrap();

        let mut layout_style: LayoutStyle = builder.style().get();
        layout_style.width_sizing = Sizing::Fill;
        builder.scoped_style(layout_style, |builder| {
            TextEdit::singleline(&mut self.search_string)
            .hint("Search")
            .build(builder);
        });

        builder.add_spacing(SPACING);

        let bounds = builder.layout.push_size(builder.layout.space_left());
        builder.layout(Layout::down_to_up(bounds), |builder| {
            let entities = game_state.world().entities();
            if entities.len() == 1 {
                builder.label(format!("{} entity in world", entities.len()));
            } else {
                builder.label(format!("{} entities in world", entities.len()));
            }

            let bounds = builder.layout.push_size(builder.layout.space_left());
            Scrollbox::new("entity_scrollbox", bounds, Direction::UpToDown).build(builder, |builder| {
                let mut entries = Vec::with_capacity(entities.len());

                entities.iter().enumerate().for_each(|(index, entity)| {
                    let label_string = match game_state.world().find::<Named>(*entity) {
                        Some(named) => {
                            format!("Entity {} - {}", index, named.name)
                        },
                        None => {
                            format!("Entity {}", index)
                        }
                    };

                    entries.push(label_string);
                });

                if self.search_string.len() > 0 {
                    entries.sort_by(|a, b| {
                        let a = a.contains(&self.search_string);
                        let b = b.contains(&self.search_string);
                        
                        if a && !b {
                            return Ordering::Greater
                        }
                        
                        if !a && b {
                            return Ordering::Less;
                        }
                        
                        return Ordering::Equal;
                    });
                }

                let mut text_style: TextStyle = builder.style().get();
                text_style.alignment = Alignment::Left;

                let mut layout_style: LayoutStyle = builder.style().get();
                layout_style.width_sizing = Sizing::Fill;

                builder.scoped_style(layout_style, |builder| {
                    builder.scoped_style(text_style, |builder| {
                        entries.drain(..).rev().for_each(|it| builder.label(it));
                    });
                });
            });
        });
    }
}