use crate::{event_stream::StreamEventData, inputs::Keyboard};

pub struct KeyboardLayoutChanged<'a> {
    payload: &'a Keyboard,
}

impl<'a> KeyboardLayoutChanged<'a> {
    pub fn from(payload: &'a Keyboard) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for KeyboardLayoutChanged<'a> {
    fn get_event_type(&self) -> &'static str {
        "keyboard.layout.changed"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}
