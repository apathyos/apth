use crate::{event_stream::StreamEventData, outputs::{OutputsMap, module::Output}};

pub struct OutputsChanged<'a> {
    payload: &'a OutputsMap,
}

impl<'a> OutputsChanged<'a> {
    pub fn from(payload: &'a OutputsMap) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for OutputsChanged<'a> {
    fn get_event_type(&self) -> &'static str {
        "outputs.changed"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}

pub struct OutputCreated<'a> {
    payload: &'a Output,
}

impl<'a> OutputCreated<'a> {
    pub fn from(payload: &'a Output) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for OutputCreated<'a> {
    fn get_event_type(&self) -> &'static str {
        "output.created"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}

pub struct OutputChanged<'a> {
    payload: &'a Output,
}

impl<'a> OutputChanged<'a> {
    pub fn from(payload: &'a Output) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for OutputChanged<'a> {
    fn get_event_type(&self) -> &'static str {
        "output.changed"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}

pub struct OutputRemoved<'a> {
    payload: &'a Output,
}

impl<'a> OutputRemoved<'a> {
    pub fn from(payload: &'a Output) -> Self {
        Self { payload }
    }
}

impl<'a> StreamEventData for OutputRemoved<'a> {
    fn get_event_type(&self) -> &'static str {
        "output.removed"
    }

    fn get_payload(&self) -> impl serde::Serialize {
        self.payload
    }
}
