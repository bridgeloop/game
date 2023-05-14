// based on https://sotrh.github.io/learn-wgpu/

use {winit::{event as Event, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder}, std::{cmp::Ordering, process::ExitCode}};

mod state;
use state::State;

mod camera;
use camera::Camera;
use winit::{event::{VirtualKeyCode, ElementState, KeyboardInput, WindowEvent, MouseButton}, window::Window};

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

	pub fn process_keyboard(&mut self, key: VirtualKeyCode, state: ElementState) {
		let amount = if state == ElementState::Pressed { 1.0 } else { 0.0 };
		match key {
			VirtualKeyCode::I | VirtualKeyCode::Up => {
				self.amount_forward = amount;
			}
			VirtualKeyCode::J | VirtualKeyCode::Left => {
				self.amount_left = amount;
			}
			VirtualKeyCode::K | VirtualKeyCode::Down => {
				self.amount_backward = amount;
			}
			VirtualKeyCode::L | VirtualKeyCode::Right => {
				self.amount_right = amount;
			}
			VirtualKeyCode::Space => {
				self.amount_up = amount;
			}
			VirtualKeyCode::Semicolon => {
				self.amount_down = amount;
			}
			_ => ()
		};
	}

	pub fn process_mouse(&mut self, (dx, dy): (f64, f64)) {
		self.rotate_horizontal = dx as f32;
		self.rotate_vertical = dy as f32;
	}
}

fn set_fullscreen(to: bool, window: &Window) -> bool {
	if to {
		fn area(size: winit::dpi::PhysicalSize<u32>) -> u32 {
			size.width * size.height
		}
		let video_modes = window.current_monitor().expect("no monitor detected").video_modes();
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
		}).expect("no video modes");
		window.set_fullscreen(
			Some(winit::window::Fullscreen::Exclusive(video_mode.clone()))
		);
		window.set_cursor_visible(false);
		window.set_cursor_grab(winit::window::CursorGrabMode::Confined).expect("failed to lock cursor");
	} else {
		window.set_fullscreen(None);
		window.set_cursor_visible(true);
		window.set_cursor_grab(winit::window::CursorGrabMode::None).expect("failed to unlock cursor");
	}

	return to;
}

fn real_main() -> Result<(), &'static str> {
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("game")
		.build(&(event_loop))
		.map_err(|_| "failed to create window")?;

	let mut fullscreen = set_fullscreen(false, &(window));

	let mut state = State::new(window)?;
	let mut input = Input::new(1.0, 0.088 * 31.4);
	let mut last_render_time = std::time::Instant::now();

	let mut timer = std::time::Instant::now();
	let mut frames = 0;

	event_loop.run(move |event, _, flow| {
		*flow = ControlFlow::Poll;
		let window = state.window();

		match event {
			Event::Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
				WindowEvent::MouseInput { state, button, .. } => {
					if button == MouseButton::Left && state == ElementState::Pressed && !fullscreen {
						fullscreen = set_fullscreen(true, &(window));
					}
				}
				WindowEvent::CloseRequested => {
					*flow = ControlFlow::Exit;
				}
				WindowEvent::Resized(physical_size) => {
					state.reconfigure(Some(physical_size));
				}
				WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
					state.reconfigure(Some(*new_inner_size));
				}

				WindowEvent::KeyboardInput {
					input:
						KeyboardInput {
							virtual_keycode: Some(key),
							state,
							..
						},
					..
				} if fullscreen => match key {
					VirtualKeyCode::Escape if state == ElementState::Pressed => {
						fullscreen = set_fullscreen(false, &(window));
					}
					_ => input.process_keyboard(key, state),
				}

				_ => ()
			}
			Event::Event::DeviceEvent {
				event: Event::DeviceEvent::MouseMotion { delta, }, ..
			} if fullscreen => {
				input.process_mouse(delta);
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
				let time = now.duration_since(timer).as_millis();
				if time > 1000 {
					timer = now;
					println!("frames in the past {time}ms: {:?}", frames);
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