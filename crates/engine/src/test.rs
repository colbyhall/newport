use crate::*;

struct TestModule;

impl ModuleCompileTime for TestModule {
    fn new() -> Result<Self, String> {
        Ok(TestModule)
    }
}

impl ModuleRuntime for TestModule {
    fn as_any(&self) -> &dyn Any { self }
}

#[test]
fn modules() {
    let builder = Builder::new()
        .name("test".to_string())
        .module::<TestModule>();
    
    Engine::run(builder).unwrap();
}