use serde::Serialize;

pub struct Output;

impl Output {
    pub fn serialize(snapshot: impl Serialize, pretty: &bool) -> String {
        if pretty.clone() {
            serde_json::to_string_pretty(&snapshot).unwrap()
        } else {
            serde_json::to_string(&snapshot).unwrap()
        }
    }
}
