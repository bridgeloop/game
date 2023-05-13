// based on https://sotrh.github.io/learn-wgpu/

use {winit::{event as Event, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder}, std::{cmp::Ordering, process::ExitCode}};

mod state;
use state::State;

mod camera;
use camera::Camera;
use winit::event::{VirtualKeyCode, ElementState, KeyboardInput, WindowEvent};

#[derive(Debug)]
pub struct Input {
	amount_left: f32,
	amount_right: f32,
	amount_forward: f32,
	amount_backward: f32,
	amount_up: f32,
	amount_down: f32,
	rotate_horizontal: f32,
	rotate_vertical: f32,
	speed: f32,
	sensitivity: f32,
}

impl Input {
	pub fn new(speed: f32, sensitivity: f32) -> Self {
		Self {
			amount_left: 0.0,
			amount_right: 0.0,
			amount_forward: 0.0,
			amount_backward: 0.0,
			amount_up: 0.0,
			amount_down: 0.0,
			rotate_horizontal: 0.0,
			rotate_vertical: 0.0,
			speed,
			sensitivity,
		}
	}

	pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) -> bool{
		let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
		match key {
			VirtualKeyCode::I | VirtualKeyCode::Up => {
				self.amount_forward = amount;
				true
			}
			VirtualKeyCode::J | VirtualKeyCode::Left => {
				self.amount_left = amount;
				true
			}
			VirtualKeyCode::K | VirtualKeyCode::Down => {
				self.amount_backward = amount;
				true
			}
			VirtualKeyCode::L | VirtualKeyCode::Right => {
				self.amount_right = amount;
				true
			}
			VirtualKeyCode::Space => {
				self.amount_up = amount;
				true
			}
			VirtualKeyCode::Semicolon => {
				self.amount_down = amount;
				true
			}
			_ => false,
		}
	}

	pub fn process_mouse(&mut self, (dx, dy): (f64, f64)) {
		self.rotate_horizontal = dx as f32;
		self.rotate_vertical = dy as f32;
	}
}

fn real_main() -> Result<(), &'static str> {
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("game")
		.build(&(event_loop))
		.map_err(|_| "failed to create window")?;
	fn area(size: winit::dpi::PhysicalSize<u32>) -> u32 {
		size.width * size.height
	}
	let video_modes = window.current_monitor().unwrap().video_modes();
	let video_mode = video_modes.max_by(|x, y| {
		if area(x.size()) > area(y.size()) {
			return Ordering::Greater;
		} else if area(x.size()) < area(y.size()) {
			return Ordering::Less;
		}
		if x.refresh_rate_millihertz() > y.refresh_rate_millihertz() {
			return Ordering::Greater;
		} else {
			return Ordering::Less;
		}
	}).unwrap();
	window.set_fullscreen(
		Some(winit::window::Fullscreen::Exclusive(video_mode))
	);
	window.set_cursor_visible(false);

	let mut state = State::new(window)?;
	let mut input = Input::new(1.0, 0.088 * 31.4);
	let mut last_render_time = std::time::Instant::now();

	let mut timer = std::time::Instant::now();
	let mut frames = 0;

	event_loop.run(move |event, _, flow| {
		*flow = ControlFlow::Poll;
		let window = state.window();
		window.set_cursor_grab(winit::window::CursorGrabMode::Confined).expect("failed to lock cursor");
		match event {
			Event::Event::DeviceEvent {
				event: Event::DeviceEvent::MouseMotion { delta, },
				..
			} => {
				input.process_mouse(delta);
			}
			Event::Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							virtual_keycode: Some(key),
							state,
							..
						},
					..
				} => { input.process_keyboard(key, state); }
				WindowEvent::CloseRequested => {
					*flow = ControlFlow::Exit;
				},
				WindowEvent::Resized(physical_size) => {
					state.reconfigure(Some(physical_size));
				}
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					state.reconfigure(Some(*new_inner_size));
				}
				_ => (),
			}
			Event::Event::MainEventsCleared => {
				let now = std::time::Instant::now();
				let dt = now - last_render_time;
				last_render_time = now;
				state.update(&(input), dt);
				input.rotate_horizontal = 0.0;
				input.rotate_vertical = 0.0;
				match state.render() {
					Err(wgpu::SurfaceError::Lost) => state.reconfigure(None),
					Err(wgpu::SurfaceError::OutOfMemory) => *flow = ControlFlow::ExitWithCode(1),
					Err(e) => eprintln!("{:?}", e),

					_ => (),
				};

				frames += 1;

				let now = std::time::Instant::now();
				if now.duration_since(timer).as_millis() > 1000 {
					timer = now;
					println!("{:?}", frames);
					frames = 0;
				}
			}
			_ => (),
		};

		return;
	});
}

fn main() -> ExitCode {
	let r: Result<(), &'static str> = real_main();
	if let Err(e) = r {
		eprintln!("{e}");
		return ExitCode::FAILURE;
	}
	return ExitCode::SUCCESS;
}