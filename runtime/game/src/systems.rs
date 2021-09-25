use crate::components::*;
use crate::ecs::{
	Query,
	System,
	World,
};
use engine::Engine;
use math::{
	Quaternion,
	Vector3,
};
use platform::input::*;
use sync::async_trait;

#[derive(Clone)]
pub struct CameraDriver;

#[async_trait]
impl System for CameraDriver {
	async fn run(&self, world: &World, dt: f32) {
		let game3d: &mut crate::Game3d = unsafe { Engine::module_mut().unwrap() };

		let mut query = Query::builder()
			.read::<Camera>()
			.write::<Transform>()
			.write::<CameraController>()
			.execute(world);

		for it in query.iter() {
			const SPEED: f32 = 6.0;

			let transform: &mut Transform = it.get_mut().unwrap();
			let controller: &mut CameraController = it.get_mut().unwrap();

			if game3d.input_state.mouse_locked {
				controller.yaw += game3d.input_state.mouse_delta.x / 10.0;
				controller.pitch -= game3d.input_state.mouse_delta.y / 10.0;
			}

			transform.rotation =
				Quaternion::from_euler(Vector3::new(controller.pitch, controller.yaw, 0.0));

			let rotation = transform.rotation;
			let forward = rotation.forward();
			let right = rotation.right();

			if game3d.input_state.is_key_down(KEY_W) {
				transform.position += forward * dt * SPEED;
			}
			if game3d.input_state.is_key_down(KEY_S) {
				transform.position += -forward * dt * SPEED;
			}

			if game3d.input_state.is_key_down(KEY_D) {
				transform.position += right * dt * SPEED;
			}
			if game3d.input_state.is_key_down(KEY_A) {
				transform.position += -right * dt * SPEED;
			}

			if game3d.input_state.is_key_down(KEY_SPACE) {
				transform.position += Vector3::UP * dt * SPEED;
			}

			if game3d.input_state.is_key_down(KEY_LCTRL) {
				transform.position += -Vector3::UP * dt * SPEED;
			}

			if game3d.input_state.was_key_pressed(KEY_L)
				|| game3d.input_state.was_key_pressed(KEY_ESCAPE)
			{
				game3d.input_state.mouse_locked = !game3d.input_state.mouse_locked;

				let window = Engine::window().unwrap();

				if game3d.input_state.mouse_locked {
					window.set_cursor_grab(true).unwrap();
					window.set_cursor_visible(false);

					let size = window.outer_size();

					window
						.set_cursor_position(platform::winit::dpi::PhysicalPosition::new(
							size.width / 2,
							size.height / 2,
						))
						.unwrap();
				} else {
					window.set_cursor_grab(false).unwrap();
					window.set_cursor_visible(true);
				}
			}
		}
	}
}

#[derive(Clone)]
pub struct SpinDriver;

#[async_trait]
impl System for SpinDriver {
	async fn run(&self, world: &World, dt: f32) {
		let mut query = Query::builder()
			.write::<Transform>()
			.read::<Spinner>()
			.execute(world);

		for it in query.iter() {
			let transform: &mut Transform = it.get_mut().unwrap();
			let spinner: &Spinner = it.get().unwrap();

			let rotation =
				Quaternion::from_euler([spinner.speed * dt, 0.0, spinner.speed * dt * 0.5]);

			transform.rotation = transform.rotation * rotation;
		}
	}
}

#[derive(Clone)]
pub struct ScaleDriver;

#[async_trait]
impl System for ScaleDriver {
	async fn run(&self, world: &World, dt: f32) {
		let mut query = Query::builder()
			.write::<Transform>()
			.write::<Scaler>()
			.execute(world);

		for it in query.iter() {
			let transform: &mut Transform = it.get_mut().unwrap();
			let scaler: &mut Scaler = it.get_mut().unwrap();

			scaler.time += dt * scaler.speed;

			let scale = 1.0 + scaler.time.sin().abs() * scaler.max + scaler.min;
			transform.scale = [scale, scale, scale].into();
		}
	}
}
