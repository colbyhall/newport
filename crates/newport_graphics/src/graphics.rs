use crate::{
    engine,
    gpu,
    asset,

    Texture,
    FontCollection,
};

use engine::{ 
    Module, 
    Engine, 
    EngineBuilder
};

use gpu::{ 
    Instance, 
    Device, 
    RenderPass, 
    Format
};

use asset::{ 
    AssetManager, 
    AssetVariant 
};

pub struct Graphics {
    device:      Device,
    backbuffer_render_pass: RenderPass,
}

impl Graphics {
    pub fn device(&self) -> &Device {
        &self.device
    }

    pub fn backbuffer_render_pass(&self) -> &RenderPass {
        &self.backbuffer_render_pass
    }
}

impl Module for Graphics {
    fn new() -> Self {
        let engine = Engine::as_ref();

        let instance = Instance::new().unwrap();
        let device = instance.create_device(Some(engine.window().handle())).unwrap();

        let backbuffer_render_pass = device.create_render_pass(vec![Format::BGR_U8_SRGB], None).unwrap();

        Self { device, backbuffer_render_pass }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<AssetManager>()
            .register(AssetVariant::new::<Texture>())
            .register(AssetVariant::new::<FontCollection>())
    }
}