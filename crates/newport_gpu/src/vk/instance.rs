use crate::InstanceCreateError;
use super::ENABLED_LAYER_NAMES;

use ash::vk;
use ash::version::EntryV1_0;

use std::sync::Arc;

pub struct Instance {
    pub entry:    ash::Entry,
    pub instance: ash::Instance,
}

impl Instance {
    pub fn new() -> Result<Arc<Self>, InstanceCreateError> {
        let entry = unsafe{ 
            let entry = ash::Entry::new();
            if entry.is_err() {
                return Err(InstanceCreateError::FailedToLoadLibrary);
            }
            entry.unwrap()
        };

        let app_info = vk::ApplicationInfo{
            api_version: vk::make_version(1, 2, 0),
            ..Default::default()
        };

        #[cfg(target_os = "windows")]
        let enabled_extension_names = [
            b"VK_KHR_surface\0".as_ptr() as *const i8,
            b"VK_KHR_win32_surface\0".as_ptr() as *const i8
        ];

        let create_info = vk::InstanceCreateInfo::builder()
            .application_info(&app_info)
            .enabled_extension_names(&enabled_extension_names)
            .enabled_layer_names(&ENABLED_LAYER_NAMES);
        let instance = unsafe{ 
            let instance = entry.create_instance(&create_info, None);
            if instance.is_err() {
                let err = instance.err().unwrap();
                match err {
                    ash::InstanceError::LoadError(_err) => return Err(InstanceCreateError::FailedToLoadLibrary),
                    ash::InstanceError::VkError(err) => {
                        match err {
                            vk::Result::ERROR_INCOMPATIBLE_DRIVER => return Err(InstanceCreateError::IncompatibleDriver),
                            _ => return Err(InstanceCreateError::Unknown),
                        }
                    }
                }
            }
            instance.unwrap()
        };

        Ok(Arc::new(Self {
            entry:    entry,
            instance: instance,
        }))
    }
}
