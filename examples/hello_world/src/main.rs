use {
	engine::{
		define_run_module,
		Builder,
		Engine,
		Module,
	},
	resources::*,
	serde::{
		Deserialize,
		Serialize,
	},
};

struct HelloWorld;

impl Module for HelloWorld {
	fn new() -> Self {
		let foo: Handle<Foo> =
			Handle::find_or_load("{A6D46364-14C8-4322-BAC9-859002D5687F}").unwrap();

		Self
	}

	fn depends_on(builder: Builder) -> Builder {
		builder
			.module::<ResourceManager>()
			.register(Foo::variant())
			.register(Bar::variant())
			.register(NativeImporter::<Foo>::variant(&["foo"]))
			.register(NativeImporter::<Bar>::variant(&["bar"]))
	}
}

define_run_module!(HelloWorld, "Hello World");

#[derive(Resource, Serialize, Deserialize)]
struct Foo {
	bar: Handle<Bar>,
	a: i32,
	c: f32,
	d: String,
}

#[derive(Resource, Serialize, Deserialize)]
struct Bar {
	e: Option<u32>,
}
