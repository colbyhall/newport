use engine::{
	define_run_module,
	Module,
};

struct HelloWorld;
impl Module for HelloWorld {
	fn new() -> Self {
		Self
	}
}

define_run_module!(HelloWorld, "Hello World");
