use newport_engine::{ ModuleCompileTime, ModuleRuntime, Engine };
use newport_gpu::{ Instance, Device };

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
}

impl ModuleRuntime for Graphics {
    fn post_init(&mut self, engine: &mut Engine) {
        self.device = self.instance.create_device(Some(engine.window().handle())).ok();
    }
}