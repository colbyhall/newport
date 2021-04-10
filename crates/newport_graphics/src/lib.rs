use newport_engine::{ ModuleCompileTime, ModuleRuntime, Engine, EngineBuilder };
use newport_gpu::{ Instance, Device };
use newport_asset::AssetManager;

pub mod font;

pub struct Graphics {
    instance: Instance,
    device:   Option<Device>,
}

impl Graphics {
    pub fn device(&self) -> &Device {
        self.device.as_ref().unwrap()
    }
}

impl ModuleCompileTime for Graphics {
    fn new() -> Self {
        Self {
            instance: Instance::new().unwrap(),
            device:   None,
        }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<AssetManager>()
    }
}

impl ModuleRuntime for Graphics {
    fn post_init(&mut self, engine: &mut Engine) {
        self.device = self.instance.create_device(Some(engine.window().handle())).ok();

        let asset_manager = engine.module_mut::<AssetManager>().unwrap();
        asset_manager
            .register_variant::<font::FontCollection>();
    }
}