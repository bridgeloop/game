// based on https://sotrh.github.io/learn-wgpu/

/*
quality aiden messages:

i fucking hate everything
but i mean whatever
mouse input is not continuous so i don't think that delta time should be a factor in rotating the camera
(whereas keydown input is continuous)
rn i have two factors going into sensitivty:
- dots multiplier
- dots per degree
(dots are the unit of mouse input) 
so like i have 8.8% dots multiplier
and dots per degree at 1920/360
this is where the problem is
sens depends on resolution
im not sure what fortnite does about this
maybe they use a fixed amount of dots per degree?
also im accumulating winit mousemotion 
i've seen a tutorial doing mouse_x = mouse_delta.x
but im doing mouse_x += mouse_delta.x
im not sure which one is correct
but mousemotion can be fired multiple times per frame
so it's not like they're doing the same thing
i think i'll actually have to read winit's code for this one
-----
also i want to add functionality to acmec to either (try to?) wait for dns updates, or actually add the dns txt record itself
i think it should hit the wait though
i think updating the dns shit should be done by a separate program/shell script
*/

/*
https://gamingsmart.com/mouse-sensitivity-converter/fortnite/
game sens: 8.8%

var dpi = 800;
var inches_per_360dg = 9.21;
var inches_per_1dg = inches_per_360dg / 360;

var dots_per_1dg = inches_per_1dg * dpi;
*/

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
	dots_per_deg: f32,
}

impl Input {
	pub fn new(speed: f32, dots_per_360deg: f32) -> Self {
		Self {
			amount_left: 0.0,
			amount_right: 0.0,
			amount_forward: 0.0,
			amount_backward: 0.0,
			amount_up: 0.0,
			amount_down: 0.0,
			mouse_moved: (0.0, 0.0),
			speed,
			dots_per_deg: dots_per_360deg / 360.0,
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
	let mut input = Input::new(1.0, 7368.0);

	let mut total_elapsed = 0.0;
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
				input.mouse_moved.0 = delta.0 as f32;
				input.mouse_moved.1 = delta.1 as f32;
			}

			Event::Event::MainEventsCleared => {
				let mut elapsed = prev_render.elapsed().as_secs_f32();
				total_elapsed += elapsed;
                prev_render = Instant::now();

                const TIMESTEP: f32 = 1.0 / 60.0;

                let mut interpolate = 1.0;
                let sf = interpolate / (elapsed / TIMESTEP);

                while elapsed >= TIMESTEP {
                	state.camera.update_pos(&(input), TIMESTEP);
                	state.camera.update_rot(&(input), sf);
                	elapsed -= TIMESTEP;
                	interpolate -= sf;
                }
                state.camera.update_pos(&(input), elapsed);
                state.camera.update_rot(&(input), interpolate);
                input.mouse_moved = (0.0, 0.0);

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
				if total_elapsed >= 1.0 {
					println!("frames in the past {total_elapsed}: {frames:?}");
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