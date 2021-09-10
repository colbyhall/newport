use asset::AssetRef;
use ecs::ComponentVariant;
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

#[derive(Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
#[serde(crate = "self::serde")]
pub struct Transform {
	pub location: Vector3,
	pub rotation: Quaternion,
	pub scale: Vector3,
}

impl Transform {
	pub fn matrix4(self) -> Matrix4 {
		//TODO: Do this without mat4 multiplication
		Matrix4::translate(self.location)
			* Matrix4::rotate(self.rotation)
			* Matrix4::scale(self.scale)
	}
}

// #[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
// #[serde(crate = "self::serde")]
// pub struct MeshRender {
// 	pub mesh: AssetRef<Mesh>,
// }
