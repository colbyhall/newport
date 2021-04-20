use newport_engine::{ Module, Engine, EngineBuilder };
use newport_gpu as gpu;
use gpu::{ Instance, Device, RenderPass, Format };
use newport_asset::AssetManager;

mod font;
pub use font::*;

mod texture;
pub use texture::*;

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

        let asset_manager = engine.module::<AssetManager>().unwrap();
        asset_manager
            .register_variant::<FontCollection>()
            .register_variant::<Texture>();

        let backbuffer_render_pass = device.create_render_pass(vec![Format::BGR_U8_SRGB], None).unwrap();

        Self { device, backbuffer_render_pass }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<AssetManager>()
    }
}

