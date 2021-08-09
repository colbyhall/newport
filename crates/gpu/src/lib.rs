//! This crate is the HAL for the GPU. Currently Vulkan is the only back end available. The design and architecture
//! was originally concepted after reading http://alextardif.com/RenderingAbstractionLayers.html
//!
//! # Warning
//!
//! This package is still in a very early state. The API is currently super volatile. I would not
//! recommend using this package if you don't plan on handling the unknown future changes.
//!
//! # Goals
//!
//! * Abstraction layer should be as lightweight as possible. As many API layer specfic concepts should be
//!    hidden from the user
//!
//! * Abstraction layer should be as simple as possible. There will be code complexity that is unavoidable but
//!   they should be rare. If the user ends up spending too much time debugging just to get to the meat of
//!   their calls then we have failed
//!
//! * Abstraction layer should be easy to maintain and add on. The hope is that the above points aid this goal
//!
//! # Needs
//!
//! * Ability to create multiple devices to allow multiple GPU work if desired
//! * Create, upload, and destroy resources (buffers, textures, shaders, pipelines, etc)
//! * Gather, submit, and wait on command work from various passes, in a multicore-compatible way
//! * Automatic device memory management

#![feature(in_band_lifetimes)]

use math::{
	Color,
	Rect,
};

use serde::{
	self,
	Deserialize,
	Serialize,
};

use std::convert::Into;

#[cfg(feature = "vulkan")]
mod vk;

#[cfg(feature = "vulkan")]
use vk as api;

mod buffer;
mod command_buffer;
mod device;
mod gpu;
mod graphics_pipeline;
mod instance;
mod receipt;
mod render_pass;
mod shader;
mod texture;

pub use {
	buffer::*,
	command_buffer::*,
	device::*,
	gpu::*,
	graphics_pipeline::*,
	instance::*,
	receipt::*,
	render_pass::*,
	shader::*,
	texture::*,
};

/// Type of memory allocations that buffers or textures can be allocated from
#[derive(Copy, Clone, Debug)]
pub enum MemoryType {
	/// Able to be uploaded to by mapping memory. Slower to access. Faster to write to
	HostVisible,
	/// Able to be uploaded to by using commands. Faster to access. Slower to write to
	DeviceLocal,
}

#[derive(Copy, Clone, Debug)]
pub enum ResourceCreateError {
	Unknown,
	OutOfMemory,
}

#[derive(Serialize, Deserialize, Copy, Clone)]
#[serde(crate = "self::serde")]
pub struct SamplerDescription {
	min_filter: Filter,
	mag_filter: Filter,
	address_u: Wrap,
	address_v: Wrap,
	address_w: Wrap,
}

impl Default for SamplerDescription {
	fn default() -> Self {
		Self {
			min_filter: Filter::Linear,
			mag_filter: Filter::Linear,

			address_u: Wrap::Clamp,
			address_v: Wrap::Clamp,
			address_w: Wrap::Clamp,
		}
	}
}
