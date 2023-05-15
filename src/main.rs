// based on https://sotrh.github.io/learn-wgpu/

use {winit::{event as Event, event_loop::{ControlFlow, EventLoop}, window::WindowBuilder}, std::{cmp::Ordering, process::ExitCode}};

mod state;
use std::time::Instant;

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
	mouse_moved: (f32, f32),
	speed: f32,
	sens: f32,
}

impl Input {
	pub fn new(speed: f32, dots_multiplier: f32, dots_per_degree: f32) -> Self {
		Self {
			amount_left: 0.0,
			amount_right: 0.0,
			amount_forward: 0.0,
			amount_backward: 0.0,
			amount_up: 0.0,
			amount_down: 0.0,
			mouse_moved: (0.0, 0.0),
			speed,
			sens: dots_multiplier / dots_per_degree,
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
}

fn set_fullscreen(to: bool, window: &Window, cursor_locked: &mut bool) -> bool {
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

	*cursor_locked = to;
	return to;
}

fn real_main() -> Result<(), &'static str> {
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("game")
		.build(&(event_loop))
		.map_err(|_| "failed to create window")?;

	let mut cursor_locked = false;
	let mut fullscreen = set_fullscreen(false, &(window), &mut(cursor_locked));

	let mut state = State::new(window)?;
	let mut input = Input::new(
		1.0,
		8.8 / 100.0,
		// i don't want the sensitivity to be tied to the resolution, though.
		1920.0 / 360.0
	);

	let mut total_elapsed = 0.0;
	let mut timer = Instant::now();
	let mut frames = 0;

	let mut prev_render = Instant::now();

	event_loop.run(move |event, _, flow| {
		let window = state.window();
		*flow = ControlFlow::Poll;

		match event {
			Event::Event::WindowEvent { window_id, event } if window_id == window.id() => match event {
				WindowEvent::MouseInput { state, button, .. } => {
					if button == MouseButton::Left && state == ElementState::Pressed {
						if !fullscreen {
							fullscreen = set_fullscreen(true, &(window), &mut(cursor_locked));
						} else if !cursor_locked {
							window.set_cursor_grab(winit::window::CursorGrabMode::Confined).expect("failed to lock cursor");
							window.set_cursor_visible(false);
							cursor_locked = true;
						}
					}
				}
				WindowEvent::CursorLeft { .. } if fullscreen => {
					window.set_cursor_grab(winit::window::CursorGrabMode::None).expect("failed to unlock cursor");
					window.set_cursor_visible(true);
					cursor_locked = false;
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
				} => match key {
					VirtualKeyCode::Escape if state == ElementState::Pressed => {
						fullscreen = set_fullscreen(!fullscreen, &(window), &mut(cursor_locked));
					}
					_ => input.process_keyboard(key, state),
				}

				_ => ()
			}
			Event::Event::DeviceEvent {
				event: Event::DeviceEvent::MouseMotion { delta, }, ..
			} if cursor_locked => {
				input.mouse_moved.0 += delta.0 as f32;
				input.mouse_moved.1 += delta.1 as f32;
			}

			Event::Event::MainEventsCleared => {
				let mut elapsed = prev_render.elapsed().as_secs_f32();
				total_elapsed += elapsed;
                prev_render = Instant::now();

                const TIMESTEP: f32 = 1.0 / 60.0;

                while elapsed >= TIMESTEP {
                	state.camera.update(&mut(input), TIMESTEP);
                	elapsed -= TIMESTEP;
                }
                state.camera.update(&mut(input), elapsed);

	            state.window().request_redraw();
			}
			Event::Event::RedrawRequested(_) => {
				state.update();
				match state.render() {
					Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.reconfigure(None),
					Err(wgpu::SurfaceError::OutOfMemory) => *flow = ControlFlow::ExitWithCode(1),
					Err(e) => eprintln!("{:?}", e),

					_ => ()
				};

				frames += 1;
				let time = timer.elapsed().as_millis();
				if time > 1000 {
					timer = Instant::now();
					println!("frames in the past {time}ms: {:?}. {total_elapsed}", frames);
					frames = 0;
					total_elapsed = 0.0;
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