use asset::AssetRef;
use ecs::ComponentVariant;
use engine::Builder;
use gpu::GraphicsPipeline;
use graphics::Mesh;
use math::{
	Matrix4,
	Quaternion,
	Vector3,
};
use serde::{
	self,
	Deserialize,
	Serialize,
};

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize, Default)]
pub struct Transform {
	pub location: Vector3,
	pub rotation: Quaternion,
	pub scale: Vector3,
}

impl Transform {
	pub fn to_matrix4(self) -> Matrix4 {
		// TODO: Do this without mat4 multiplication
		Matrix4::translate(self.location)
			* Matrix4::rotate(self.rotation)
			* Matrix4::scale(self.scale)
	}
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
pub struct MeshRender {
	pub mesh: AssetRef<Mesh>,
	pub pipeline: AssetRef<GraphicsPipeline>,
}

#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Camera {
	pub fov: f32,
}

impl Default for Camera {
	fn default() -> Self {
		Self { fov: 90.0 }
	}
}

#[derive(Default, Clone, Serialize, Deserialize)]
pub struct CameraController {
	pub pitch: f32,
	pub yaw: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Spinner {
	pub speed: f32,
}

#[derive(Clone, Copy, Serialize, Deserialize)]
pub struct Scaler {
	pub speed: f32,
	pub max: f32,
	pub time: f32,
}

pub(crate) fn register_components(builder: Builder) -> Builder {
	builder
		.register(ComponentVariant::new::<Transform>())
		.register(ComponentVariant::new::<MeshRender>())
		.register(ComponentVariant::new::<Camera>())
		.register(ComponentVariant::new::<CameraController>())
		.register(ComponentVariant::new::<Spinner>())
		.register(ComponentVariant::new::<Scaler>())
}
