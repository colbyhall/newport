use newport::*;
use math::*;

use engine::{ Engine, EngineBuilder, Module };
use editor::{ Editor, Page, CtxRef, SidePanel, ScrollArea, CollapsingHeader, Layout };
use ecs::{ World, Entity };

use std::sync::Mutex;

#[derive(Editable)]
struct Transform {
    position: Vector3,
    rotation: Quaternion,
    scale:    Vector3,
}

impl Default for Transform {
    fn default() -> Self {
        Self {
            position: Vector3::ZERO,
            rotation: Quaternion::default(),
            scale:    Vector3::ONE,           
        }
    }
}

#[derive(Editable)]
struct Named {
    name: String,
}

struct NonEditable {
    _foo: Vec<f32>,
}

#[derive(Editable)]
struct Door {
    openness: f32,
    color:    Color,
}

#[derive(Editable)]
struct PersonalData {
    age:    i32,
    weight: f32,
}

struct EditorExample {
    world: Mutex<World>,
}

struct WorldPage {
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
        let example = engine.module::<EditorExample>().unwrap();

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
                            let named = world.find::<Named>(*it);
                            
                            let label = if named.is_some() {
                                let named = named.unwrap();
                                format!("Entity {} - {}", index, named.name)
                            } else {
                                format!("Entity {}", index)
                            };
                            
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

impl Module for EditorExample {
    fn new() -> Self {
        let mut world = World::new();

        // This list of names is randomly generated :P
        let names = [
            "Arturo Wheeler",
            "Madilyn Simpson",
            "Jeffery Dixon",
            "Emily Casey",
            "Josh Figueroa",
            "Jerry Garrison",
            "Brynn Good",
            "Kianna Hanson",
            "Leo Singleton",
            "Ean Rhodes",
            "Selina Baker",
            "Karlie Macias",
            "Benjamin Brooks",
            "Griffin Cain",
            "Armani Massey",
            "Sullivan Greene",
            "Kason Church",
            "Hailey Mcintyre",
            "Noel Friedman",
            "Krish Doyle",
            "Drew Gordon",
            "Karina Wright",
            "Trenton Benton",
            "Ari Pearson",
            "Abbey Lane",
            "Jordin Santos",
            "Makhi Merritt",
            "Saniya Sutton",
            "Matilda Bowers",
            "Finley Snyder",
            "Jamarion Rowe",
            "Jocelynn Allison",
            "Rebekah Swanson",
            "Raphael Olson",
            "Kobe Hernandez",
            "Abbigail Mata",
            "Jermaine Moon",
            "Zara Moore",
            "Evelin Patton",
            "Mikayla Zuniga",
            "Jazlyn Maldonado",
            "Kyla Huerta",
            "Talia Williamson",
            "Edwin Rasmussen",
            "Destiny Ball",
            "Pedro Clay",
        ];

        for (index, name) in names.iter().enumerate() {
            world.create()
                .with(Transform::default())
                .with(Named{
                    name: name.to_string(),
                })
                .with(PersonalData{
                    age: (index % 37) as i32,
                    weight: 185.7,
                })
                .finish();
        }

        world.create()
            .with(Transform::default())
            .with(Named{
                name:   "Door".to_string(),
            })
            .with(Door{
                openness: 45.0,
                color: Color::WHITE,
            })
            .with(NonEditable{
                _foo: Vec::new(),
            })
            .finish();

        Self{
            world: Mutex::new(world),
        }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<Editor>()
            .post_init(|engine: &Engine| {
                let editor = engine.module::<Editor>().unwrap();
                editor.push_page(Box::new(WorldPage{
                    selected: None,
                }));
            })
    }
}

fn main() {
    let builder = EngineBuilder::new()
        .module::<EditorExample>()
        .name("Editor Example");
    Engine::run(builder);
}