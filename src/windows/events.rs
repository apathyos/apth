use crate::{event_stream::StreamEventData, windows::module::WindowsMap};

pub struct WindowsChanged<'a> {
    payload: &'a WindowsMap,
}

impl<'a> WindowsChanged<'a> {
    pub fn from(payload: &'a WindowsMap) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for WindowsChanged<'a> {
    fn get_event_type(&self) -> &'static str {
        "windows.changed"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}
