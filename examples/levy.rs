use newport::{
    engine,
    graphics,
    asset,
    math,
    gpu,
};

use engine::{ 
    Module, 
    Engine, 
    EngineBuilder
};


use graphics::{
    Graphics,
    Pipeline,
};

use asset::{
    AssetManager,
    AssetRef,
};

use math::{
    Color,
};

struct LevyExample {
    _test: AssetRef<Pipeline>,
}

// Implement the module trait
impl Module for LevyExample {
    fn new() -> Self {
        let engine = Engine::as_ref();
        let asset_manager = engine.module::<AssetManager>().unwrap();

        let test = asset_manager.find("{62b4ffa0-9510-4818-a6f2-7645ec304d8e}").unwrap();

        Self{
            _test: test,
        }
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .tick(|engine: &Engine, _dt: f32| {
                let graphics = engine.module::<Graphics>().unwrap();
                let device = graphics.device();

                let backbuffer = device.acquire_backbuffer();

                let mut gfx = device.create_graphics_context().unwrap();
                gfx.begin();
                {
                    gfx.begin_render_pass(&graphics.backbuffer_render_pass(), &[&backbuffer]);
                    gfx.clear(Color::GREEN);
                    gfx.end_render_pass();

                    gfx.resource_barrier_texture(&backbuffer, gpu::Layout::ColorAttachment, gpu::Layout::Present);
                }
                gfx.end();

                let receipt = device.submit_graphics(vec![gfx], &[]);
                device.display(&[receipt]);
                device.wait_for_idle();
            })
            .module::<Graphics>()
            .module::<AssetManager>()
    }
}

// Start the app runner
fn main() {
    let builder = EngineBuilder::new()
        .module::<LevyExample>()
        .name("Game Example");
    Engine::run(builder);
}