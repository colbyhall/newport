use {
	crate::{
		PhysicsManager,
		Shape,
	},
	ecs::Entity,
	math::{
		Point3,
		Quat,
		Vec3,
	},
	rapier3d::{
		na::{
			Quaternion,
			UnitQuaternion,
		},
		prelude::*,
	},
};

pub trait Query {
	type Hit;
	fn single(self, filter: Filter, manager: &PhysicsManager) -> Option<Self::Hit>;
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct RayCast {
	pub origin: Point3,
	pub direction: Vec3,
	pub distance: f32,
}

impl RayCast {
	pub fn new(origin: impl Into<Point3>, direction: impl Into<Vec3>, distance: f32) -> Self {
		Self {
			origin: origin.into(),
			direction: direction.into(),
			distance,
		}
	}
}

impl Query for RayCast {
	type Hit = RayCastHit;
	fn single(self, filter: Filter, manager: &PhysicsManager) -> Option<Self::Hit> {
		let ray = Ray::new(
			point![self.origin.x, self.origin.y, self.origin.z],
			vector![self.direction.x, self.direction.y, self.direction.z],
		);

		let filter = |collider_handle: ColliderHandle| {
			let collider = manager.collider_set.get(collider_handle).unwrap();
			let entity: Entity = collider.user_data.into();

			let passes = !filter.ignore.iter().any(|e| *e == entity);
			passes
		};
		let filter: Option<&dyn Fn(ColliderHandle) -> bool> = Some(&filter);

		let (collider, intersection) = manager.query_pipeline.cast_ray_and_get_normal(
			&manager.collider_set,
			&ray,
			self.distance,
			true,
			InteractionGroups::all(),
			filter,
		)?;

		let collider = manager
			.collider_set
			.get(collider)
			.expect("Failed to find hit collider in collider set");
		let entity = collider.user_data.into();

		let impact = self.origin + self.direction * intersection.toi;
		let normal = Vec3::new(
			intersection.normal.x,
			intersection.normal.y,
			intersection.normal.z,
		);

		Some(Self::Hit {
			entity,

			impact,
			normal,
		})
	}
}

#[derive(Debug, Clone)]
pub struct RayCastHit {
	pub entity: Entity,

	pub impact: Point3,
	pub normal: Vec3,
}

#[derive(Debug, Clone, Copy)]
pub struct ShapeCast {
	pub start: Point3,
	pub end: Point3,

	pub rotation: Quat,
	pub shape: Shape,
}

impl ShapeCast {
	pub fn new(
		start: impl Into<Point3>,
		end: impl Into<Point3>,
		rotation: Quat,
		shape: Shape,
	) -> Self {
		Self {
			start: start.into(),
			end: end.into(),
			rotation,
			shape,
		}
	}
}

impl Default for ShapeCast {
	fn default() -> Self {
		Self::new(0.0, 0.0, Quat::IDENTITY, Shape::cube(0.5))
	}
}

impl Query for ShapeCast {
	type Hit = ShapeCastHit;
	fn single(self, filter: Filter, manager: &PhysicsManager) -> Option<Self::Hit> {
		let mut shape_pos = Isometry::translation(self.start.x, self.start.y, self.start.z);
		shape_pos.rotation = UnitQuaternion::from_quaternion(Quaternion::new(
			self.rotation.w,
			self.rotation.x,
			self.rotation.y,
			self.rotation.z,
		));

		let start_to_end = self.end - self.start;
		let direction = start_to_end.norm().unwrap_or_default();

		let filter = |collider_handle: ColliderHandle| {
			let collider = manager.collider_set.get(collider_handle).unwrap();
			let entity: Entity = collider.user_data.into();

			let passes = !filter.ignore.iter().any(|e| *e == entity);
			passes
		};

		// FIXME: Should there be some centralized way of making rapier3d shapes
		//        To do this we couldn't have just a `to_rapier3d` method as each
		// 		  shape is their own unique type
		let (collider, hit) = {
			let direction = vector![direction.x, direction.y, direction.z];
			let distance = start_to_end.len();
			let groups = InteractionGroups::all();
			let filter: Option<&dyn Fn(ColliderHandle) -> bool> = Some(&filter);
			match self.shape {
				Shape::Cube { half_extents } => {
					let shape = rapier3d::geometry::Cuboid::new(vector![
						half_extents.x,
						half_extents.y,
						half_extents.z
					]);
					manager.query_pipeline.cast_shape(
						&manager.collider_set,
						&shape_pos,
						&direction,
						&shape,
						distance,
						groups,
						filter,
					)
				}
				Shape::Capsule {
					half_height,
					radius,
				} => {
					let shape = rapier3d::geometry::Capsule::new_z(half_height, radius);
					manager.query_pipeline.cast_shape(
						&manager.collider_set,
						&shape_pos,
						&direction,
						&shape,
						distance,
						groups,
						filter,
					)
				}
				_ => unimplemented!(),
			}
		}?;

		let collider = manager
			.collider_set
			.get(collider)
			.expect("Failed to find hit collider in collider set");
		let entity: Entity = collider.user_data.into();

		if hit.status == rapier3d::parry::query::TOIStatus::Penetrating {
			Some(Self::Hit {
				entity,
				status: ShapeCastStatus::Penetrating,
			})
		} else {
			Some(Self::Hit {
				entity,
				status: ShapeCastStatus::Success {
					origin_at_impact: self.start + direction * (hit.toi - 0.001),
					witnesses: [
						ShapeCastWitness {
							impact: Vec3::new(hit.witness1[0], hit.witness1[1], hit.witness1[2]),
							normal: Vec3::new(hit.normal1[0], hit.normal1[1], hit.normal1[2]),
						},
						// FIXME: Convert all of this to world space as it is apparently local
						ShapeCastWitness {
							impact: Vec3::new(hit.witness2[0], hit.witness2[1], hit.witness2[2]),
							normal: Vec3::new(hit.normal2[0], hit.normal2[1], hit.normal2[2]),
						},
					],
				},
			})
		}
	}
}

#[derive(Debug, Clone)]
pub struct ShapeCastWitness {
	pub impact: Point3,
	pub normal: Point3,
}

#[derive(Debug, Clone)]
pub enum ShapeCastStatus {
	Penetrating,
	Success {
		origin_at_impact: Point3,
		witnesses: [ShapeCastWitness; 2],
	},
}

#[derive(Debug, Clone)]
pub struct ShapeCastHit {
	pub entity: Entity,
	pub status: ShapeCastStatus,
}

#[derive(Default, Debug, Clone)]
pub struct Filter {
	pub ignore: Vec<Entity>,
}
