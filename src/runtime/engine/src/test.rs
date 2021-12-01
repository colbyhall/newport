use crate::*;

#[test]
fn hello_world() {
	Engine::builder().test().unwrap();
}

#[test]
fn module() {
	struct TestModule;
	impl Module for TestModule {
		fn new() -> Self {
			Self
		}
	}
	Engine::builder().module::<TestModule>().test().unwrap();
}

#[test]
fn module_dependency() {
	struct ModuleA;
	impl Module for ModuleA {
		fn new() -> Self {
			Self
		}
	}
	struct ModuleB;
	impl Module for ModuleB {
		fn new() -> Self {
			Engine::module::<ModuleA>().unwrap();
			Self
		}

		fn depends_on(builder: Builder) -> Builder {
			builder.module::<ModuleA>()
		}
	}
	Engine::builder().module::<ModuleB>().test().unwrap();
}
