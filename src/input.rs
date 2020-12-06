use crate::camera::{Camera, CameraAction};
use log::trace;
use std::sync::mpsc;
use wasm_bindgen::JsCast;
use web_sys::{Event, KeyboardEvent};

// TODO create an InputEvent enum to clean up some of the inputs from JS

#[derive(Clone, Debug)]
pub enum InputEvent {
    KeyDown { event: KeyboardEvent },
    KeyUp { event: KeyboardEvent },
}

/// A handler for all input events. **Input listeners should be registered in
/// JS, then calls propagated to [Self::handle_event].** This uses an MPSC
/// channel to collect input event so that they can be processed during the main
/// game loop.
pub struct InputHandler {
    /// Sender for the channel we will send all events on
    sender: mpsc::Sender<InputEvent>,
    /// Sender for the channel we will send all events on
    receiver: mpsc::Receiver<InputEvent>,
}

impl InputHandler {
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::channel();
        Self { sender, receiver }
    }

    /// Handle an incoming event from the DOM. This should be called from each
    /// event listener on the canvas element. The listeners should be registered
    /// in JS.
    pub fn handle_event(&self, event: Event) {
        let input_event = match event.type_().as_str() {
            "keydown" => InputEvent::KeyDown {
                event: event.dyn_into().unwrap(),
            },
            "keyup" => InputEvent::KeyUp {
                event: event.dyn_into().unwrap(),
            },
            other => panic!("Unknown event type: {}", other),
        };
        self.sender.send(input_event).unwrap();
    }

    /// Apply the actions for a single input event
    fn process_event(&self, camera: &mut Camera, event: InputEvent) {
        match event {
            InputEvent::KeyDown { event } => {
                let cam_action = match event.key().as_str() {
                    "w" | "W" => Some(CameraAction::MoveForward),
                    "s" | "S" => Some(CameraAction::MoveBackward),
                    "a" | "A" => Some(CameraAction::MoveLeft),
                    "d" | "D" => Some(CameraAction::MoveRight),
                    "ArrowUp" => Some(CameraAction::RotateUp),
                    "ArrowDown" => Some(CameraAction::RotateDown),
                    "ArrowLeft" => Some(CameraAction::RotateLeft),
                    "ArrowRight" => Some(CameraAction::RotateRight),
                    " " => Some(CameraAction::MoveUp),
                    "Shift" => Some(CameraAction::MoveDown),
                    other => {
                        trace!("Unknown key code: {}", other);
                        None
                    }
                };
                if let Some(cam_action) = cam_action {
                    camera.apply_action(cam_action, 0.1);
                }
            }
            InputEvent::KeyUp { event } => todo!(),
        }
    }

    /// Process all available input events in the MPSC channel. This will apply
    /// each action, to update whatever game state needs to be updated.
    pub fn process_events(&mut self, camera: &mut Camera) {
        for event in self.receiver.try_iter() {
            self.process_event(camera, event);
        }
    }
}
