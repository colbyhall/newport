use newport_editor::{ Page, CtxRef, SidePanel, CollapsingHeader, ScrollArea, Layout };
use newport_ecs::Entity;
use newport_engine::Engine;

use crate::Game;

#[derive(Default)]
pub struct WorldPage {
    selected: Option<Entity>,
}

impl Page for WorldPage {
    fn can_close(&self) -> bool {
        false
    }

    fn name(&self) -> &str {
        "World"
    }

    fn show(&mut self, ctx: &CtxRef) {
        let engine = Engine::as_ref();
        let example = engine.module::<Game>().unwrap();

        let mut world = example.world.lock().unwrap();

        let width = 300.0;
        SidePanel::left("Entity Introspect", width).show(ctx, |ui| {
            ui.set_min_width(width);

            CollapsingHeader::new("Entities")
                .default_open(true)
                .show(ui, |ui| {
                    ui.set_min_height(100.0);
                    ui.set_max_height(200.0);

                    let entities = world.entities();
                    ScrollArea::auto_sized().show(ui, |ui| {
                        for (index, it) in entities.iter().enumerate() {
                            let label = format!("Entity {}", index);
                            
                            let is_me = if self.selected.is_some() {
                                let selected = self.selected.unwrap();
                                *it == selected
                            } else {
                                false
                            };

                            ui.with_layout(Layout::left_to_right().with_cross_justify(true), |ui| {
                                if ui.selectable_label(is_me, label).clicked() {
                                    if is_me {
                                        self.selected = None;
                                    } else {
                                        self.selected = Some(*it);
                                    }
                                }
                            });
                        }
                    })
                });

            if self.selected.is_some() {
                let entity = self.selected.unwrap();
                CollapsingHeader::new("Introspect")
                    .default_open(true)
                    .show(ui, |ui| {
                        ui.set_max_height(600.0);
                        ScrollArea::auto_sized().show(ui, |ui| {
                            world.edit(entity, ui);
                        });
                    });
            } else {
                ui.heading("No Entity Selected");
            }
        });
    }
}