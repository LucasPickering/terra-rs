use gloo::events::EventListener;
use std::{ops::Deref, sync::mpsc};
use web_sys::{Event, EventTarget};

pub struct InputHandler {
    // receiver: mpsc::UnboundedReceiver<Event>,
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
}

impl Deref for InputHandler {
    type Target = mpsc::Receiver<Event>;

    fn deref(&self) -> &Self::Target {
        &self.receiver
    }
}
