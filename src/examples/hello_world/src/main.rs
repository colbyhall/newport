use {
	egui::*,
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
};

struct HelloWorld {
	name: String,
	age: i32,

	color: [f32; 3],
}

impl Module for HelloWorld {
	fn new() -> Self {
		Self {
			name: String::default(),
			age: 0,
			color: [0.0; 3],
		}
	}

	fn depends_on(builder: Builder) -> Builder {
		builder.module::<Egui>().register(EguiScope::new(|ctx| {
			let hello_world: &mut HelloWorld = unsafe { Engine::module_mut().unwrap() };
			let HelloWorld { name, age, color } = hello_world;

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
				ui.color_edit_button_rgb(color);
			});
		}))
	}
}

define_run_module!(HelloWorld, "Hello World");
