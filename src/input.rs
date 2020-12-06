use crate::camera::{Camera, CameraAction};
use gloo::events::EventListener;
use log::error;
use std::sync::mpsc;
use wasm_bindgen::{prelude::*, JsCast};
use web_sys::{Event, EventTarget, KeyboardEvent};

pub struct InputHandler {
    /// A channel that we will send all events to
    receiver: mpsc::Receiver<Event>,
    // These all need to be hung onto, because they are de-registered when
    // they're dropped
    _on_key_press: EventListener,
}

impl InputHandler {
    pub fn new(canvas: &EventTarget) -> Self {
        let (sender, receiver) = mpsc::channel();

        // Attach an event listener
        let on_key_press =
            EventListener::new(canvas, "keydown", move |event| {
                sender.send(event.clone()).unwrap();
            });

        Self {
            receiver,
            _on_key_press: on_key_press,
        }
    }

    // Process a single input event
    fn process_event(&self, camera: &mut Camera, event: Event) {
        match event.type_().as_str() {
            "keydown" => {
                let event: &KeyboardEvent = event.dyn_ref().unwrap_throw();
                let cam_action = match event.key().as_str() {
                    "w" | "W" => Some(CameraAction::MoveForward),
                    "s" | "S" => Some(CameraAction::MoveBackward),
                    "a" | "A" => Some(CameraAction::MoveLeft),
                    "d" | "D" => Some(CameraAction::MoveRight),
                    "ArrowUp" => Some(CameraAction::RotateUp),
                    "ArrowDown" => Some(CameraAction::RotateDown),
                    "ArrowLeft" => Some(CameraAction::RotateLeft),
                    "ArrowRight" => Some(CameraAction::RotateRight),
                    _ => None,
                };
                if let Some(cam_action) = cam_action {
                    camera.apply_action(cam_action, 0.1);
                }
            }
            other => error!("Unhandled event type: {}", other),
        }
    }

    /// Process all available input events
    pub fn process_events(&mut self, camera: &mut Camera) {
        for event in self.receiver.try_iter() {
            self.process_event(camera, event);
        }
    }
}
