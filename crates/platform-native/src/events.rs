//! Native event bus using tokio::sync::broadcast

use futures::StreamExt;
use prsnl_core::{AppEvent, EventBus, EventStream};
use tokio::sync::broadcast;
use tokio_stream::wrappers::BroadcastStream;

/// Native event bus implementation using tokio broadcast channels
pub struct NativeEventBus {
    tx: broadcast::Sender<AppEvent>,
}

impl NativeEventBus {
    pub fn new() -> Self {
        let (tx, _) = broadcast::channel(256);
        Self { tx }
    }
}

impl Default for NativeEventBus {
    fn default() -> Self {
        Self::new()
    }
}

impl EventBus for NativeEventBus {
    fn publish(&self, event: AppEvent) {
        let _ = self.tx.send(event);
    }

    fn subscribe(&self) -> EventStream {
        let rx = self.tx.subscribe();
        Box::pin(BroadcastStream::new(rx).filter_map(|r| async { r.ok() }))
    }
}
