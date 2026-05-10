use std::sync::{Arc, Mutex, atomic::AtomicU64};

use serde::Serialize;
use serde_json;
use tokio::sync::broadcast;

use crate::{event_stream::StreamEventData, utils::time};

#[macro_export]
macro_rules! stream_emit {
    ($ctx:expr, $deffer:expr, $event_data:expr $(,)?) => {{
        $ctx.emit($deffer, $event_data);
    }};
}

#[macro_export]
macro_rules! stream_flush {
    ($ctx:expr $(,)?) => {{
        $ctx.flush();
    }};
}

#[derive(Clone, Serialize)]
pub struct StreamEvent {
    pub version: u32,
    pub seq: u64,
    pub timestamp: u128,
    #[serde(rename = "type")]
    pub event_type: &'static str,
    pub payload: serde_json::Value,
}

#[derive(Default, Clone)]
pub struct StreamContext {
    seq: Arc<AtomicU64>,
    tx: Option<broadcast::Sender<StreamEvent>>,
    deffered: Arc<Mutex<Vec<StreamEvent>>>,
}

impl StreamContext {
    pub fn new(capacity: usize) -> Self {
        let (tx, _) = broadcast::channel(capacity);

        Self {
            seq: Arc::new(AtomicU64::new(0)),
            tx: Some(tx),
            deffered: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn subscribe(&self) -> Option<broadcast::Receiver<StreamEvent>> {
        Some(self.tx.as_ref().unwrap().subscribe())
    }

    pub fn emit<T: StreamEventData>(&self, deffer: bool, event_data: T) {
        let event = StreamEvent {
            version: 1,
            seq: self.seq.fetch_add(1, std::sync::atomic::Ordering::Relaxed) + 1,
            timestamp: time::get_time_now(),
            event_type: event_data.get_event_type(),
            payload: serde_json::to_value(event_data.get_payload()).unwrap(),
        };

        if deffer {
            let mut deffered_mutex = self.deffered.lock().unwrap();
            deffered_mutex.push(event);
        } else {
            let _ = self.tx.as_ref().unwrap().send(event);
        }
    }

    pub fn flush(&self) {
        let mut buffer = self.deffered.lock().unwrap();

        for event in buffer.drain(..) {
            let _ = self.tx.as_ref().unwrap().send(event);
        }
    }
}
