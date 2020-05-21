
use super::AppEvent;
use crate::glutin::{
    event::{self, ElementState, WindowEvent, ModifiersState},
    event_loop,
    platform::desktop::EventLoopExtDesktop,
};

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::mpsc::Receiver;

use crate::renderer::Window;
use super::{App, Application};
use crate::core::Update;


pub enum ClickState {
    None,
    Single,
    Double,
    Triple,
}

// pub struct Mouse {
//     pub x: f32,
//     pub y: f32,
//     pub left_button: ElementState,
//     pub right_button: ElementState,
//     pub middle_button: ElementState,
//     pub click_state: ClickState,
// }

// impl Default for Mouse {
//     fn default() -> Self {
//         Self {
//             x: 0.0,
//             y: 0.0,
//             left_button: ElementState::Released,
//             right_button: ElementState::Released,
//             middle_button: ElementState::Released,
//             click_state: ClickState::None,
//         }
//     }
// }

pub struct EventHandler {
    pub elp: event_loop::EventLoopProxy<AppEvent>,
    pub modifiers: event::ModifiersState,
    // pub mouse: Mouse,
}

impl EventHandler {

    pub fn new(elp: event_loop::EventLoopProxy<AppEvent>) -> Self {
        Self {
            elp,
            modifiers: event::ModifiersState::empty(),
        }
    }

    fn skip_event(event: &event::Event<AppEvent>) -> bool {
        match event {
            event::Event::WindowEvent { event, .. } => {
                use glutin::event::WindowEvent::*;
                match event {
                    KeyboardInput {
                        is_synthetic: true, ..
                    }
                    | TouchpadPressure { .. }
                    | CursorEntered { .. }
                    | AxisMotion { .. }
                    | HoveredFileCancelled
                    | Destroyed
                    | HoveredFile(_)
                    | Touch(_)
                    | Moved(_) => true,
                    _ => false,
                }
            }
            event::Event::NewEvents { .. } => false,
            event::Event::Suspended { .. }
            | event::Event::MainEventsCleared
            | event::Event::LoopDestroyed => true,
            event::Event::DeviceEvent { .. } => true,
            _ => false,
        }
    }

    fn handle_event(event: event::Event<AppEvent>, handler: &mut Self, app: &mut Application) {
        use event::WindowEvent::*;
        match event {
            event::Event::WindowEvent {
                window_id: _,
                event,
            } => match event {
                Resized(size) => {
                    app.update_size(size.width, size.height);
                }
                Moved(pos) => {}
                CloseRequested => {
                    handler.elp.send_event(AppEvent::Exit).ok();
                }
                Focused(focus) => {}
                ReceivedCharacter(ch) => {
                    // app.handle_character(ch, &handler.modifiers);
                }
                KeyboardInput {
                    device_id,
                    input,
                    is_synthetic,
                } => {
                    // app.handle_keyboard_input(input, &handler.modifiers)
                }
                CursorMoved {
                    device_id,
                    position,
                    ..
                } => {
                    // let pos = position.cast::<f32>();
                    // handler.mouse.x = pos.x;
                    // handler.mouse.y = pos.y;
                }
                CursorEntered { device_id } => {}
                CursorLeft { device_id } => {}
                MouseWheel {
                    device_id,
                    delta,
                    phase,
                    ..
                } => {
                    // app.handle_mouse_wheel(delta, phase, &handler.modifiers);
                }
                MouseInput {
                    device_id,
                    state,
                    button,
                    ..
                } => {
                    // match button {
                    //     glutin::event::MouseButton::Left => {
                    //         handler.mouse.left_button = state;
                    //     }
                    //     glutin::event::MouseButton::Right => {
                    //         handler.mouse.right_button = state;
                    //     }
                    //     glutin::event::MouseButton::Middle => {
                    //         handler.mouse.middle_button = state;
                    //     }
                    //     glutin::event::MouseButton::Other(i) => {
                    //         // other mouse buttons are not supported at the moment.
                    //     }
                    // }
                    // app.handle_mouse_input(&handler.mouse,&modifiers);
                }
                ModifiersChanged(mods) => {
                    handler.modifiers = mods;
                }
                ThemeChanged(theme) => {}
                // Destroyed => {},
                // i might what to handle this eventually.
                // DroppedFile(path) => {},
                // HoveredFile(path) => {},
                // HoveredFileCancelled => {},
                _ => {}
            },
            event::Event::NewEvents(start_cause) => {
                use event::StartCause::*;
                match start_cause {
                    Init => app.on_init(),
                    _ => {}
                }
            }
            _ => (),
        }
    }

    pub fn run(&mut self, app: App, mut event_loop: event_loop::EventLoop<AppEvent>, receiver: Receiver<Update>) {
        let mut event_queue = Vec::new();

        event_loop.run_return(|event, el, cf| {
            if Self::skip_event(&event) {
                return;
            }

            match event {
                event::Event::UserEvent(e) => match e {
                    AppEvent::Exit => *cf = event_loop::ControlFlow::Exit,
                },
                event::Event::WindowEvent {
                    event:
                        WindowEvent::ScaleFactorChanged {
                            scale_factor: _,
                            new_inner_size: _,
                        },
                    ..
                } => {}
                event => unsafe {
                    *cf = event_loop::ControlFlow::Poll;
                    event_queue.push(std::mem::transmute(event));
                },
            }

            let mut guard = app.inner();
            for event in event_queue.drain(..) {
                Self::handle_event(event, self, &mut guard);
            }

            guard.maybe_render();
        });
    }
}
