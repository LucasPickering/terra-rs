use crate::camera::{Camera, CameraAction};
use anyhow::{anyhow, Context};
use gloo::events::EventListener;
use std::{
    collections::HashSet,
    convert::{TryFrom, TryInto},
    str::FromStr,
    sync::mpsc,
};
use wasm_bindgen::{JsCast, UnwrapThrowExt};
use web_sys::{Event, EventTarget, KeyboardEvent};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Key {
    W,
    A,
    S,
    D,
    UpArrow,
    DownArrow,
    LeftArrow,
    RightArrow,
    Space,
    LeftShift,
}

impl FromStr for Key {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "w" | "W" => Ok(Self::W),
            "s" | "S" => Ok(Self::S),
            "a" | "A" => Ok(Self::A),
            "d" | "D" => Ok(Self::D),
            "ArrowUp" => Ok(Self::UpArrow),
            "ArrowDown" => Ok(Self::DownArrow),
            "ArrowLeft" => Ok(Self::LeftArrow),
            "ArrowRight" => Ok(Self::RightArrow),
            " " => Ok(Self::Space),
            "Shift" => Ok(Self::LeftShift),
            _ => Err(anyhow!("Unknown key code: {}", s)),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum InputEvent {
    KeyDown { key: Key, repeat: bool },
    KeyUp { key: Key },
}

// Convert a DOM event into a pure Rust type, to make handling logic much easier
impl TryFrom<Event> for InputEvent {
    type Error = anyhow::Error;

    fn try_from(event: Event) -> Result<Self, Self::Error> {
        match event.type_().as_str() {
            "keydown" => {
                let event: &KeyboardEvent = event.dyn_ref().unwrap_throw();
                Ok(InputEvent::KeyDown {
                    key: event.key().parse()?,
                    repeat: event.repeat(),
                })
            }
            "keyup" => {
                let event: &KeyboardEvent = event.dyn_ref().unwrap_throw();
                Ok(InputEvent::KeyUp {
                    key: event.key().parse()?,
                })
            }
            other => Err(anyhow!("Unknown event type: {}", other)),
        }
    }
}

/// A handler for all input events. Input listeners are registered in the
/// constructor, and we use an MPSC channel to collect events from all
/// listeners. Call [Self::process_events] on each frame to process events from
/// the queue.
pub struct InputHandler {
    /// Receiver for the channel where events are pushed
    receiver: mpsc::Receiver<Event>,
    /// Track which keys are currently held down. A key should be added to the
    /// sent on key-down, and removed on key-up. The browser's logic for
    /// repeating keypresses sucks so we have to implement that ourselves.
    pressed_keys: HashSet<Key>,
    /// We need to hold onto the event listeners, because they get unregistered
    /// when they're dropped
    _listeners: [EventListener; 2],
}

impl InputHandler {
    pub fn new(canvas: &EventTarget) -> Self {
        let (sender, receiver) = mpsc::channel();

        // This closure only captures `sender` which is cheap to clone, so we
        // can clone the closure freely
        let callback = move |event: &Event| {
            sender.send(event.clone()).unwrap_throw();
        };
        let listeners = [
            EventListener::new(&canvas, "keydown", callback.clone()),
            EventListener::new(&canvas, "keyup", callback),
        ];

        Self {
            receiver,
            pressed_keys: HashSet::new(),
            _listeners: listeners,
        }
    }

    /// Process all available input events in the MPSC channel. This will apply
    /// each action, to update whatever game state needs to be updated.
    pub fn process_events(
        &mut self,
        camera: &mut Camera,
    ) -> anyhow::Result<()> {
        // Pull all events out of the queue, convert each one to a Rust event,
        // then process that event
        for event in self.receiver.try_iter() {
            let event: InputEvent = event
                .try_into()
                .context("Failed to convert DOM event to Rust")?;

            match event {
                InputEvent::KeyDown { key, repeat } => {
                    if !repeat {
                        self.pressed_keys.insert(key);
                    }
                }
                InputEvent::KeyUp { key } => {
                    self.pressed_keys.remove(&key);
                }
            }
        }

        self.process_held_keys(camera);

        Ok(())
    }

    /// Apply actions according to which keys are currently being held
    fn process_held_keys(&self, camera: &mut Camera) {
        for key in &self.pressed_keys {
            let cam_action = match key {
                Key::W => Some(CameraAction::MoveForward),
                Key::S => Some(CameraAction::MoveBackward),
                Key::A => Some(CameraAction::MoveLeft),
                Key::D => Some(CameraAction::MoveRight),
                Key::UpArrow => Some(CameraAction::RotateUp),
                Key::DownArrow => Some(CameraAction::RotateDown),
                Key::LeftArrow => Some(CameraAction::RotateLeft),
                Key::RightArrow => Some(CameraAction::RotateRight),
                Key::Space => Some(CameraAction::MoveUp),
                Key::LeftShift => Some(CameraAction::MoveDown),
            };
            if let Some(cam_action) = cam_action {
                camera.apply_action(cam_action, 0.1);
            }
        }
    }
}
