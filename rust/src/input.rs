use crate::{
    camera::{Camera, CameraMovement},
    config::InputConfig,
};
use cgmath::Point2;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    sync::mpsc,
};

/// Actions that can be initiated by user input. This defines the semantic
/// meanings of the user's input, and is mapped to [InputEvent] via bindings in
/// the config.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum InputAction {
    CameraForward,
    CameraBackward,
    CameraLeft,
    CameraRight,
    CameraUp,
    CameraDown,
    CameraPan,
}

/// All the keys on the keyboard
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Key {
    // Keys
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

    // Mouse buttons
    Mouse1,

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
    MouseDown { x: isize, y: isize },
    MouseUp { x: isize, y: isize },
    MouseMove { x: isize, y: isize },
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
    /// Current location of the mouse pointer, updated every time we receive a
    /// mouse event.
    mouse_pos: Point2<isize>,
    /// The last mouse position that was used for camera panning. Updated only
    /// while the camera is panning.
    last_mouse_pan_pos: Point2<isize>,
}

impl InputHandler {
    pub fn new(config: InputConfig) -> Self {
        let (sender, receiver) = mpsc::channel();
        Self {
            config,
            sender,
            receiver,
            pressed_keys: HashSet::new(),
            last_mouse_pan_pos: Point2::new(0, 0),
            mouse_pos: Point2::new(0, 0),
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
        // Shitty but we have to pull the iter into a vec so that we can access
        // &mut self from within the loop
        let events: Vec<InputEvent> = self.receiver.try_iter().collect();

        // Pull all events out of the queue, convert each one to a Rust event,
        // then process that event
        for event in events {
            match event {
                InputEvent::KeyDown { key, repeat } => {
                    if !repeat {
                        self.pressed_keys.insert(key);
                    }
                }
                InputEvent::KeyUp { key } => {
                    self.pressed_keys.remove(&key);
                }
                InputEvent::MouseDown { x, y } => {
                    // Start panning the mouse
                    self.pressed_keys.insert(Key::Mouse1);
                    self.mouse_pos = Point2::new(x, y);
                }
                InputEvent::MouseUp { x, y } => {
                    self.pressed_keys.remove(&Key::Mouse1);
                    self.mouse_pos = Point2::new(x, y);
                }
                InputEvent::MouseMove { x, y } => {
                    self.mouse_pos = Point2::new(x, y);
                }
                // When we lose focus, clear all key states
                InputEvent::Blur => self.pressed_keys.clear(),
            }
        }

        // Right now we only care about apply-while-held actions. At some point
        // we can add on-down or on-up actions when we need them
        self.process_held_keys(camera);
        self.last_mouse_pan_pos = self.mouse_pos;

        Ok(())
    }

    /// Apply actions according to which keys are currently being held
    fn process_held_keys(&mut self, camera: &mut Camera) {
        let move_speed = 1.0;
        for key in &self.pressed_keys {
            if let Some(action) = self.config.bindings.0.get(key) {
                match action {
                    InputAction::CameraForward => {
                        camera.move_camera(CameraMovement::Forward, move_speed)
                    }
                    InputAction::CameraBackward => {
                        camera.move_camera(CameraMovement::Backward, move_speed)
                    }
                    InputAction::CameraLeft => {
                        camera.move_camera(CameraMovement::Left, move_speed)
                    }
                    InputAction::CameraRight => {
                        camera.move_camera(CameraMovement::Right, move_speed)
                    }
                    InputAction::CameraUp => {
                        camera.move_camera(CameraMovement::Up, move_speed)
                    }
                    InputAction::CameraDown => {
                        camera.move_camera(CameraMovement::Down, move_speed)
                    }
                    InputAction::CameraPan => {
                        let mouse_delta =
                            self.mouse_pos - self.last_mouse_pan_pos;
                        camera.pan_camera(mouse_delta);
                    }
                }
            }
        }
    }
}
