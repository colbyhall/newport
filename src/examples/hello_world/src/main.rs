use {
	egui::*,
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	resources::*,
};

struct HelloWorld {
	name: String,
	age: i32,
}

impl Module for HelloWorld {
	fn new() -> Self {
		Self {
			name: String::default(),
			age: 0,
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.module::<Egui>().register(EguiScope::new(|ctx| {
			let hello_world: &mut HelloWorld = unsafe { Engine::module_mut().unwrap() };
			let HelloWorld { name, age } = hello_world;

			Window::new("Hello World").show(ctx, |ui| {
				ui.heading("My egui Application");
				ui.horizontal(|ui| {
					ui.label("Your name: ");
					ui.text_edit_singleline(name);
				});
				ui.add(Slider::new(age, 0..=120).text("age"));
				if ui.button("Click each year").clicked() {
					*age += 1;
				}
				ui.label(format!("Hello '{}', age {}", name, age));
			});
		}))
	}
}

define_run_module!(HelloWorld, "Hello World");
