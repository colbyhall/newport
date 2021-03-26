use newport_engine::{ ModuleCompileTime, ModuleRuntime, EngineBuilder, Engine, Any };
use newport_log::*;
use newport_asset::AssetManager;

use ash::{ vk, Entry, version::EntryV1_0 };

pub  struct GPU {
    instance: ash::Instance,
}

impl ModuleCompileTime for GPU {
    fn new() -> Result<Self, String> {
        let entry = unsafe{ Entry::new().unwrap() };

        let app_info = vk::ApplicationInfo{
            api_version: vk::make_version(1, 0, 0),
            ..Default::default()
        };

        let enabled_extension_names = [
            b"VK_KHR_surface\0".as_ptr() as *const i8,
            b"VK_KHR_win32_surface\0".as_ptr() as *const i8
        ];

        let enabled_layer_names = [
            b"VK_LAYER_KHRONOS_validation\0".as_ptr() as *const i8
        ];

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extension_names)
            .enabled_layer_names(&enabled_layer_names);
        let instance = unsafe{ entry.create_instance(&create_info, None).unwrap() };

        Ok(Self {
            instance: instance,
        })
    }

    fn depends_on(builder: EngineBuilder) -> EngineBuilder {
        builder
            .module::<AssetManager>()
    }
}

impl ModuleRuntime for GPU {
    fn as_any(&self) -> &dyn Any { self }
}