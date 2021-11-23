use super::ENABLED_LAYER_NAMES;
use crate::InstanceCreateError;

use ash::extensions::ext::DebugUtils;
use ash::version::EntryV1_0;
use ash::vk;

use std::ffi;
use std::sync::Arc;

use engine::{
	define_log_category,
	error,
	info,
	warn,
};

define_log_category!(Vulkan, VULKAN_CATEGORY);

pub struct Instance {
	pub entry: ash::Entry,
	pub instance: ash::Instance,

	_debug_utils: DebugUtils,
}

impl Instance {
	pub fn new() -> Result<Arc<Self>, InstanceCreateError> {
		let entry = unsafe {
			let entry = ash::Entry::new();
			if entry.is_err() {
				return Err(InstanceCreateError::FailedToLoadLibrary);
			}
			entry.unwrap()
		};

		let app_info = vk::ApplicationInfo {
			api_version: vk::make_version(1, 2, 0),
			..Default::default()
		};

		#[cfg(target_os = "windows")]
		let enabled_extension_names = [
			b"VK_KHR_surface\0".as_ptr() as *const i8,
			b"VK_KHR_win32_surface\0".as_ptr() as *const i8,
			b"VK_KHR_get_physical_device_properties2\0".as_ptr() as *const i8,
			// b"VK_KHR_maintenance3\0".as_ptr() as *const i8,
			// b"VK_EXT_descriptor_indexing\0".as_ptr() as *const i8,
			b"VK_EXT_debug_utils\0".as_ptr() as *const i8,
		];

		unsafe extern "system" fn debug_callback(
			message_severity: vk::DebugUtilsMessageSeverityFlagsEXT,
			_message_types: vk::DebugUtilsMessageTypeFlagsEXT,
			p_callback_data: *const vk::DebugUtilsMessengerCallbackDataEXT,
			_p_user_data: *mut ffi::c_void,
		) -> vk::Bool32 {
			let name = ffi::CStr::from_ptr((*p_callback_data).p_message_id_name);
			let message = ffi::CStr::from_ptr((*p_callback_data).p_message);

			let output = format!("{:?} -> {:?}", name, message);

			match message_severity {
				vk::DebugUtilsMessageSeverityFlagsEXT::INFO
				| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE => {
					info!(VULKAN_CATEGORY, "{}", output);
				}
				vk::DebugUtilsMessageSeverityFlagsEXT::ERROR => {
					error!(VULKAN_CATEGORY, "{}", output);
				}
				vk::DebugUtilsMessageSeverityFlagsEXT::WARNING => {
					warn!(VULKAN_CATEGORY, "{}", output);
				}
				_ => unreachable!(),
			}
			vk::FALSE
		}

		let mut debug_create_info = vk::DebugUtilsMessengerCreateInfoEXT::builder()
			.message_severity(
				vk::DebugUtilsMessageSeverityFlagsEXT::INFO
					| vk::DebugUtilsMessageSeverityFlagsEXT::VERBOSE
					| vk::DebugUtilsMessageSeverityFlagsEXT::WARNING
					| vk::DebugUtilsMessageSeverityFlagsEXT::ERROR,
			)
			.message_type(vk::DebugUtilsMessageTypeFlagsEXT::VALIDATION)
			.pfn_user_callback(Some(debug_callback))
			.build();

		let create_info = vk::InstanceCreateInfo::builder()
			.application_info(&app_info)
			.enabled_extension_names(&enabled_extension_names)
			.enabled_layer_names(&ENABLED_LAYER_NAMES)
			.push_next(&mut debug_create_info);

		let instance = unsafe {
			let instance = entry.create_instance(&create_info, None);
			if instance.is_err() {
				let err = instance.err().unwrap();
				match err {
					ash::InstanceError::LoadError(_err) => {
						return Err(InstanceCreateError::FailedToLoadLibrary)
					}
					ash::InstanceError::VkError(err) => match err {
						vk::Result::ERROR_INCOMPATIBLE_DRIVER => {
							return Err(InstanceCreateError::IncompatibleDriver)
						}
						_ => return Err(InstanceCreateError::Unknown),
					},
				}
			}
			instance.unwrap()
		};

		let debug_utils = unsafe {
			let debug_utils = DebugUtils::new(&entry, &instance);

			debug_utils
				.create_debug_utils_messenger(&debug_create_info, None)
				.unwrap();

			debug_utils
		};

		// aftermath::enable_gpu_crash_dumps().unwrap();

		Ok(Arc::new(Self {
			entry,
			instance,
			_debug_utils: debug_utils,
		}))
	}
}
