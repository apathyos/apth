use serde::Serialize;

pub trait StreamEventData {
    fn get_event_type(&self) -> &'static str;
    fn get_payload(&self) -> impl Serialize;
}
