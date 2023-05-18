// based on https://sotrh.github.io/learn-wgpu/

/*
quality aiden messages:

i fucking hate everything
but i mean whatever
mouse input is not continuous so i don't think that delta time should be a factor in rotating the camera
(whereas keydown input is continuous)
i'm not sure whether mouse_x = mouse_delta.x or mouse_x += mouse_delta.x is correct
but mousemotion can be fired multiple times per frame
so it's not like they're doing the same thing
i think i'll actually have to read winit's code for this one
-----
several thousand models doesnt necessarily mean a lot of tris if i do it right
this should perform similarly to using low-detail models
i have to submit all of the vertices i want to draw anyway
this will just decide which ones to draw, and which tris to (quickly) change into one larger tri
there would be more data for the cpu to parse ofc
but if the data is structured well (my own file format) then that should not be a problem
also gonna use a bunch of caches to speed up subsequent similar renders
further away shit will have less tris
i feel like i might want some way to mark some tris as "important" though
-----
i wanna avoid submitting tris that the camera can't see (i.e. won't actually end up on the screen)
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

use {
	std::{process::ExitCode, time::Instant},
	winit::{
		event_loop::{ControlFlow, EventLoop},
		window::WindowBuilder,
		event::{VirtualKeyCode, ElementState, WindowEvent}
	},
};

mod state;
mod camera;
mod input;

use input::Input;
use state::State;

fn handle_window_event(state: &mut State, event: WindowEvent) -> ControlFlow {
	use WindowEvent::{*, KeyboardInput as KeyboardInputEvent};
	use winit::event::{KeyboardInput, MouseButton};

	match event {
		MouseInput { state: elem_state, button, .. } => {
			if button == MouseButton::Left && elem_state == ElementState::Pressed {
				if !state.is_fullscreen() {
					state.set_fullscreen(true);
				} else if !state.is_focused() {
					// is fullscreen but not focused
					state.set_focus(true);
				}
			}
		}
		CursorLeft { .. } if state.is_focused() => {
			state.set_focus(false);
		}
		CloseRequested => {
			return ControlFlow::Exit;
		}
		Resized(physical_size) => {
			state.reconfigure(Some(physical_size));
		}
		ScaleFactorChanged { new_inner_size, .. } => {
			state.reconfigure(Some(*new_inner_size));
		}

		KeyboardInputEvent {
			input:
				KeyboardInput {
					virtual_keycode: Some(key),
					state: elem_state,
					..
				},
			..
		} => match key {
			VirtualKeyCode::Escape if elem_state == ElementState::Pressed => {
				// toggle fullscreen
				state.set_fullscreen(!state.is_fullscreen());
			}
			_  => if state.is_focused() { state.process_key(key, elem_state); }
		}

		_ => (),
	}

	return ControlFlow::Poll;
}

fn real_main() -> Result<(), &'static str> {
	let event_loop = EventLoop::new();
	let window = WindowBuilder::new()
		.with_title("game")
		.with_fullscreen(None)
		.build(&(event_loop))
		.map_err(|_| "failed to create window")?;
	let window_id = window.id();

	let mut state = State::new(window, Input::new(1.0, 7368.0))?;

	let mut total_elapsed = 0.0;
	let mut frames = 0;

	let mut prev_render = Instant::now();

	event_loop.run(move |event, _, flow| {
		use winit::event::{Event::*, DeviceEvent::MouseMotion};
		*flow = ControlFlow::Poll;

		match event {
			WindowEvent { window_id: event_window_id, event: window_event } => {
				assert!(event_window_id == window_id);
				*flow = handle_window_event(&mut(state), window_event);
			}

			DeviceEvent {
				event: MouseMotion { delta, }, ..
			} if state.is_focused() => {
				state.process_mouse_motion(delta);
			}

			MainEventsCleared => {
				let mut elapsed = prev_render.elapsed().as_secs_f32();
				total_elapsed += elapsed;
				prev_render = Instant::now();

				const TIMESTEP: f32 = 1.0 / 60.0;

				let mut interpolate = 1.0;
				let sf = interpolate / (elapsed / TIMESTEP);

				while elapsed >= TIMESTEP {
					state.update_camera(TIMESTEP, sf);

					elapsed -= TIMESTEP;
					interpolate -= sf;
				}
				state.update_camera(elapsed, interpolate);
				state.process_mouse_motion((0.0, 0.0));

				state.window().request_redraw();
			}
			RedrawRequested(_) => {
				match state.render() {
					Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => state.reconfigure(None),
					Err(wgpu::SurfaceError::OutOfMemory) => *flow = ControlFlow::ExitWithCode(1),
					Err(e) => eprintln!("{:?}", e),

					_ => ()
				};

				frames += 1;
				if total_elapsed >= 1.0 {
					println!("frames in the past {total_elapsed}s: {frames:?}");
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