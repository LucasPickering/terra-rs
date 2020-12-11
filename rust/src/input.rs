use crate::{camera::Camera, config::InputConfig};
use anyhow::{anyhow, Context};
use gloo::events::EventListener;
use log::warn;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    convert::{TryFrom, TryInto},
    str::FromStr,
    sync::mpsc,
};
use wasm_bindgen::{prelude::*, JsCast, UnwrapThrowExt};
use web_sys::{Event, EventTarget, KeyboardEvent};

/// The different kinds of actions that a user can perform.
#[wasm_bindgen]
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(untagged)]
pub enum InputAction {
    CameraMoveUp,
    CameraMoveDown,
    CameraMoveLeft,
    CameraMoveRight,
    CameraPan,
}

/// All the keys on the keyboard
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
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
    // TODO add more as needed
    Unknown,
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
            _ => {
                warn!("Unknown key code: {}", s);
                Ok(Self::Unknown)
            }
        }
    }
}

#[derive(Clone, Debug, Default)]
pub struct InputBindings(HashMap<Key, InputAction>);

impl<'de> Deserialize<'de> for InputBindings {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        // Parse the input as a map of action:key
        let map: HashMap<InputAction, Key> =
            Deserialize::deserialize(deserializer)?;

        // Flip the script (we need to do key->action lookups, not vice versa)
        // Right now we don't support one key binding to multiple actions, but
        // it wouldn't be that hard to add it here
        Ok(InputBindings(
            map.into_iter().map(|(k, v)| (v, k)).collect(),
        ))
    }
}

/// A Rustic version of [web_sys::Event]. This covers all the types of input
/// events that we may care about. This events should be produced by listeners
/// on the canvas that we attach to.
#[derive(Copy, Clone, Debug)]
pub enum InputEvent {
    KeyDown { key: Key, repeat: bool },
    KeyUp { key: Key },
    Blur,
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
            "blur" => Ok(InputEvent::Blur),
            other => Err(anyhow!("Unknown event type: {}", other)),
        }
    }
}

/// A handler for all input events. Input listeners are registered in the
/// constructor, and we use an MPSC channel to collect events from all
/// listeners. Call [Self::process_events] on each frame to process events from
/// the queue.
pub struct InputHandler {
    /// User's personal input configuration
    config: InputConfig,
    /// Receiver for the channel where events are pushed
    receiver: mpsc::Receiver<Event>,
    /// Track which keys are currently held down. A key should be added to the
    /// sent on key-down, and removed on key-up. The browser's logic for
    /// repeating keypresses sucks so we have to implement that ourselves.
    pressed_keys: HashSet<Key>,
    /// We need to hold onto the event listeners, because they get unregistered
    /// when they're dropped
    _listeners: [EventListener; 3],
}

impl InputHandler {
    pub fn new(config: InputConfig, canvas: &EventTarget) -> Self {
        let (sender, receiver) = mpsc::channel();

        // This closure only captures `sender` which is cheap to clone, so we
        // can clone the closure freely
        let callback = move |event: &Event| {
            sender.send(event.clone()).unwrap_throw();
        };
        let listeners = [
            EventListener::new(&canvas, "keydown", callback.clone()),
            EventListener::new(&canvas, "keyup", callback.clone()),
            EventListener::new(&canvas, "blur", callback),
        ];

        Self {
            config,
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
                // When we lose focus, clear all key states
                InputEvent::Blur => self.pressed_keys.clear(),
            }
        }

        // Right now we only care about apply-while-held actions. At some point
        // we can add on-down or on-up actions when we need them
        self.process_held_keys(camera);

        Ok(())
    }

    /// Apply actions according to which keys are currently being held
    fn process_held_keys(&self, camera: &mut Camera) {
        for key in &self.pressed_keys {
            if let Some(action) = self.config.bindings.0.get(key) {
                match action {
                    // InputAction::Camera(cam_action) => {
                    //     camera.apply_action(*cam_action, 0.1)
                    // }
                    _ => todo!(),
                }
            }
        }
    }
}
