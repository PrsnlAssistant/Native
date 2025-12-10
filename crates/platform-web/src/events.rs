//! Web event bus using futures-channel

use futures_channel::mpsc;
use prsnl_core::{AppEvent, EventBus, EventStream};
use std::sync::Mutex;

/// Web event bus implementation using futures-channel
///
/// Uses unbounded channels since we're in a single-threaded WASM environment
/// and don't need backpressure.
pub struct WebEventBus {
    senders: Mutex<Vec<mpsc::UnboundedSender<AppEvent>>>,
}

impl WebEventBus {
    pub fn new() -> Self {
        Self {
            senders: Mutex::new(Vec::new()),
        }
    }
}

impl Default for WebEventBus {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: WebEventBus will only be used from the main browser thread
// WASM is single-threaded, so these markers are safe
unsafe impl Send for WebEventBus {}
unsafe impl Sync for WebEventBus {}

impl EventBus for WebEventBus {
    fn publish(&self, event: AppEvent) {
        let mut senders = self.senders.lock().unwrap();
        // Remove closed senders and send to remaining ones
        senders.retain(|sender| sender.unbounded_send(event.clone()).is_ok());
    }

    fn subscribe(&self) -> EventStream {
        let (tx, rx) = mpsc::unbounded();
        self.senders.lock().unwrap().push(tx);
        // In WASM, EventStream doesn't require Send, so we can return the receiver directly
        Box::pin(rx)
    }
}
