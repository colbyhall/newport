use crate::{
    engine,
    asset,
    
    gpu::Gpu,
    Texture,
    FontCollection,
    Mesh,
};

use engine::{ 
    Module, 
    Builder
};

use asset::{  
    AssetVariant 
};

pub struct Graphics;

impl Module for Graphics {
    fn new() -> Self { Self }

    fn depends_on(builder: Builder) -> Builder {
        builder
            .module::<Gpu>()
            .register(AssetVariant::new::<Texture>(&["texture", "tex"]))
            .register(AssetVariant::new::<FontCollection>(&["font"]))
            .register(AssetVariant::new::<Mesh>(&["mesh"]))
    }
}