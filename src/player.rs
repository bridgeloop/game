use cgmath::{Point3, Deg, Vector3, Rad, InnerSpace};
use crate::input::Input;

pub struct Player {
	pub position: Point3<f32>,
	pub rot_x: Deg<f32>,

	pub buffer: wgpu::Buffer,
}

impl Player {
	pub fn sin_cos(&self) -> (f32, f32) {
		return Rad::from(self.rot_x).0.sin_cos();
	}
	
	pub fn forward_right(&self) -> (Vector3<f32>, Vector3<f32>) {
		let (sin, cos) = self.sin_cos();
		let forward = Vector3::new(cos, 0.0, sin).normalize();
		let right = Vector3::new(-sin, 0.0, cos).normalize();

		return (forward, right);
	}

	pub fn update_pos(&mut self, input: &Input, dt: f32) {
		let (forward, right) = self.forward_right();
		self.position += forward * (input.amount_forward - input.amount_backward) * (input.speed * dt);
		self.position += right * (input.amount_right - input.amount_left) * (input.speed * dt);

		self.position.y += (input.amount_up - input.amount_down) * (input.speed * dt);

		return;
	}
	pub fn update_rot(&mut self, input: &Input, sf: f32) {
		// doesn't need dt because the input is not continuous.
		let (dx, _) = input.mouse_moved;
		self.rot_x += Deg((dx / input.dots_per_deg) * sf);

		return;
	}
}