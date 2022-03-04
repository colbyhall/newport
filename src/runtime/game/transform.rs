use {
	ecs::{
		Component,
		Entity,
		World,
		WriteStorage,
	},
	math::{
		Mat4,
		Point3,
		Quat,
		Vec3,
	},
	serde::{
		Deserialize,
		Serialize,
	},
};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transform {
	location: Point3,
	rotation: Quat,
	scale: Vec3,

	parent: Option<Entity>,
	children: Vec<Entity>,

	// Cached data
	changed: bool,
	local_to_world: Mat4,
	world_to_local: Mat4,
}

impl Transform {
	pub fn builder() -> TransformBuilder {
		TransformBuilder {
			transform: Transform::default(),
		}
	}

	pub fn children(&self) -> &[Entity] {
		&self.children
	}

	pub fn parent(&self) -> Option<Entity> {
		self.parent
	}

	pub fn local_mat4(&self) -> Mat4 {
		let mut result = Mat4::IDENTITY;
		result.x_column = (self.rotation.rotate(Vec3::FORWARD) * self.scale.x, 0.0).into();
		result.y_column = (self.rotation.rotate(Vec3::RIGHT) * self.scale.y, 0.0).into();
		result.z_column = (self.rotation.rotate(Vec3::UP) * self.scale.z, 0.0).into();
		result.w_column = (self.location, 1.0).into();

		result
	}

	pub fn world_mat4(&self) -> Mat4 {
		if self.parent.is_some() {
			self.local_to_world * self.local_mat4()
		} else {
			self.local_mat4()
		}
	}

	fn update_children_local_to_world(&self, storage: &WriteStorage<Self>) {
		let local_to_world = self.local_to_world * self.local_mat4();
		let world_to_local = local_to_world.inverse().unwrap();
		for child in self.children.iter().cloned() {
			let mut child = storage.get_mut(child).unwrap();
			child.local_to_world = local_to_world;
			child.world_to_local = world_to_local;
			child.update_children_local_to_world(storage);
		}
	}

	pub fn set_local_location_and_rotation(
		&mut self,
		location: impl Into<Vec3>,
		rotation: Quat,
		storage: &WriteStorage<Self>,
	) -> &mut Self {
		self.location = location.into();
		self.rotation = rotation;
		self.changed = true;

		self.update_children_local_to_world(storage);

		self
	}

	pub fn local_location(&self) -> Vec3 {
		self.location
	}

	pub fn local_rotation(&self) -> Quat {
		self.rotation
	}

	pub fn changed(&self) -> bool {
		self.changed
	}

	pub fn set_changed(&mut self, changed: bool) -> &mut Self {
		self.changed = changed;
		self
	}
}

impl Component for Transform {
	fn on_added(_world: &World, entity: Entity, storage: &mut WriteStorage<Self>) {
		let mut child = storage.get_mut(entity).unwrap();

		if let Some(parent) = child.parent {
			let mut parent = storage
				.get_mut(parent)
				.expect("Parent should have a transform");
			parent.children.push(entity);
			child.local_to_world = parent.local_to_world * parent.local_mat4();
			child.world_to_local = child.local_to_world.inverse().unwrap_or_default();
		}
	}
}

impl Default for Transform {
	fn default() -> Self {
		Self {
			location: Point3::ZERO,
			rotation: Quat::IDENTITY,
			scale: Vec3::ONE,

			parent: None,
			children: Vec::with_capacity(32),

			changed: false,
			local_to_world: Mat4::IDENTITY,
			world_to_local: Mat4::IDENTITY,
		}
	}
}

pub struct TransformBuilder {
	transform: Transform,
}

impl TransformBuilder {
	#[must_use]
	pub fn location(mut self, location: impl Into<Point3>) -> Self {
		self.transform.location = location.into();
		self
	}

	#[must_use]
	pub fn rotation(mut self, rotation: Quat) -> Self {
		self.transform.rotation = rotation;
		self
	}

	#[must_use]
	pub fn scale(mut self, scale: impl Into<Vec3>) -> Self {
		self.transform.scale = scale.into();
		self
	}

	#[must_use]
	pub fn parent(mut self, entity: Entity) -> Self {
		self.transform.parent = Some(entity);
		self
	}

	pub fn finish(self) -> Transform {
		self.transform
	}
}
