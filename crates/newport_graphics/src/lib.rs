use newport_engine::*;
use newport_gpu::*;

pub struct Graphics {
    instance: Arc<Instance>,
    device:   Option<Arc<Device>>,
}

impl Graphics {
    pub fn device(&self) -> &Arc<Device> {
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
        let device = DeviceBuilder::new(self.instance.clone())
            .present_to(engine.window().handle())
            .spawn().unwrap();
        
        self.device = Some(device);
    }
}