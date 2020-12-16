use crate::{
    camera::{Camera, CameraMovement},
    config::InputConfig,
};
use cgmath::Point2;
use serde::Deserialize;
use std::{
    collections::{HashMap, HashSet},
    iter,
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

impl InputAction {
    /// Should this action be applied repeatedly while its bound key is being
    /// held, or should it only apply on initial key-down?
    fn apply_on_repeat(&self) -> bool {
        match self {
            Self::CameraForward
            | Self::CameraBackward
            | Self::CameraLeft
            | Self::CameraRight
            | Self::CameraUp
            | Self::CameraDown
            | Self::CameraPan => true,
        }
    }
}

/// All the keys on the keyboard
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash, Deserialize)]
pub enum Key {
    // Keys
    A,
    B,
    C,
    D,
    E,
    F,
    G,
    H,
    I,
    J,
    K,
    L,
    M,
    N,
    O,
    P,
    Q,
    R,
    S,
    T,
    U,
    V,
    W,
    X,
    Y,
    Z,
    Num1,
    Num2,
    Num3,
    Num4,
    Num5,
    Num6,
    Num7,
    Num8,
    Num9,
    Num0,
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
    Scroll { up: bool },
    Blur,
}

#[derive(Copy, Clone, Debug)]
struct KeyPress {
    key: Key,
    repeat: bool,
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
    held_keys: HashSet<Key>,
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
            held_keys: HashSet::new(),
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
        // Pull all events out of the queue, convert each one to a Rust event,
        // then process that event
        let mut pressed_keys: Vec<Key> = Vec::new();
        for event in self.receiver.try_iter() {
            match event {
                InputEvent::KeyDown { key, repeat } => {
                    // Browser's logic for key repeats is trash so we ignore
                    // those and implement our own repeating
                    if !repeat {
                        pressed_keys.push(key);
                    }
                }
                InputEvent::KeyUp { key } => {
                    self.held_keys.remove(&key);
                }
                InputEvent::MouseDown { x, y } => {
                    // Start panning the mouse
                    pressed_keys.push(Key::Mouse1);
                    self.mouse_pos = Point2::new(x, y);
                }
                InputEvent::MouseUp { x, y } => {
                    self.held_keys.remove(&Key::Mouse1);
                    self.mouse_pos = Point2::new(x, y);
                }
                InputEvent::MouseMove { x, y } => {
                    self.mouse_pos = Point2::new(x, y);
                }
                InputEvent::Scroll { up } => {
                    camera.zoom_camera(up, 5.0);
                }
                // When we lose focus, clear all key states
                InputEvent::Blur => self.held_keys.clear(),
            }
        }

        // For each keypress, look up the bound action and apply it
        let actions = iter::empty() // symmetry!
            .chain(
                pressed_keys
                    .iter()
                    .filter_map(|key| self.get_bound_action(key)),
            )
            .chain(
                // For keys that are being held down, only apply actions that
                // are designated as "apply_on_repeat"
                self.held_keys
                    .iter()
                    .filter_map(|key| self.get_bound_action(key))
                    .filter(|action| action.apply_on_repeat()),
            );
        for action in actions {
            self.process_key_press(camera, action);
        }

        // After applying all actions, update internal state
        self.held_keys.extend(pressed_keys);
        self.last_mouse_pan_pos = self.mouse_pos;

        Ok(())
    }

    fn get_bound_action(&self, key: &Key) -> Option<InputAction> {
        self.config.bindings.0.get(&key).copied()
    }

    fn process_key_press(&self, camera: &mut Camera, action: InputAction) {
        let move_speed = 1.0;
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
                let mouse_delta = self.mouse_pos - self.last_mouse_pan_pos;
                camera.pan_camera(mouse_delta);
            }
        }
    }
}
