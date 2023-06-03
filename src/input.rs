use winit::{event::ElementState, keyboard::KeyCode};

#[derive(Debug)]
pub struct Input {
	pub amount_left: f32,
	pub amount_right: f32,
	pub amount_forward: f32,
	pub amount_backward: f32,
	pub amount_up: f32,
	pub amount_down: f32,
	pub mouse_moved: (f32, f32),
	pub speed: f32,
	pub dots_per_deg: f32,
}

impl Input {
	pub fn new(speed: f32, dots_per_360deg: f32) -> Self {
		return Self {
			amount_left: 0.0,
			amount_right: 0.0,
			amount_forward: 0.0,
			amount_backward: 0.0,
			amount_up: 0.0,
			amount_down: 0.0,
			mouse_moved: (0.0, 0.0),
			speed,
			dots_per_deg: dots_per_360deg / 360.0,
		};
	}

	pub fn process_key(&mut self, key: KeyCode, state: ElementState) {
		let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
		match key {
			KeyCode::KeyI => {
				self.amount_forward = amount;
			}
			KeyCode::KeyJ => {
				self.amount_left = amount;
			}
			KeyCode::KeyK => {
				self.amount_backward = amount;
			}
			KeyCode::KeyL => {
				self.amount_right = amount;
			}
			KeyCode::Space => {
				self.amount_up = amount;
			}
			KeyCode::Semicolon => {
				self.amount_down = amount;
			}
			_ => (),
		};
		return;
	}

	pub fn set_mouse_motion(&mut self, (dx, dy): (f64, f64)) {
		self.mouse_moved = (dx as f32, dy as f32);
		return;
	}
}