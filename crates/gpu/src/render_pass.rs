use crate::api;

use std::{
	fmt,

	sync::Arc,
};

#[derive(Clone)]
pub struct RenderPass(pub(crate) Arc<api::RenderPass>);

impl fmt::Debug for RenderPass {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
		fmt.debug_struct("RenderPass").finish()
	}
}
