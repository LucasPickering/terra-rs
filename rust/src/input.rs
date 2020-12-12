use crate::{
    camera::{Camera, CameraAction},
    config::InputConfig,
};
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc,
};

/// The different kinds of actions that a user can perform.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
#[serde(untagged)]
pub enum InputAction {
    Camera(CameraAction),
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
///
/// This type **needs** to match the InputEvent in `input.ts`.
#[derive(Copy, Clone, Debug, Deserialize)]
pub enum InputEvent {
    KeyDown { key: Key, repeat: bool },
    KeyUp { key: Key },
    Blur,
}

/// A handler for all input events. Input listeners are registered in the
/// constructor, and we use an MPSC channel to collect events from all
/// listeners. Call [Self::process_events] on each frame to process events from
/// the queue.
pub struct InputHandler {
    /// User's personal input configuration
    config: InputConfig,
    /// Sender for the channel where events are pushed. This is pushed to by
    /// a function called from Wasm.
    sender: mpsc::Sender<InputEvent>,
    /// Receiver for the channel where events are pushed. This is pulled from
    /// during the render loop.
    receiver: mpsc::Receiver<InputEvent>,
    /// Track which keys are currently held down. A key should be added to the
    /// sent on key-down, and removed on key-up. The browser's logic for
    /// repeating keypresses sucks so we have to implement that ourselves.
    pressed_keys: HashSet<Key>,
}

impl InputHandler {
    pub fn new(config: InputConfig) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            config,
            sender,
            receiver,
            pressed_keys: HashSet::new(),
        }
    }

    /// TODO get a better handle and standardize it
    pub fn ingest(&self, event: InputEvent) -> anyhow::Result<()> {
        self.sender.send(event)?;
        Ok(())
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
                    InputAction::Camera(cam_action) => {
                        camera.apply_action(*cam_action, 0.1)
                    }
                }
            }
        }
    }
}
