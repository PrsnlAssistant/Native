//! Platform abstraction traits
//!
//! These traits define the interface for platform-specific implementations
//! of transport and event bus functionality.

use crate::events::AppEvent;
use crate::protocol::ImagePayload;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

// Conditional Send bounds based on target platform
// Native targets require Send for multi-threaded async runtimes
// WASM is single-threaded and cannot use Send
#[cfg(not(target_arch = "wasm32"))]
mod bounds {
    use super::*;

    /// Result type for async transport operations
    pub type TransportResult<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send>>;

    /// Result type for async transport operations (no return value)
    pub type TransportResultVoid = Pin<Box<dyn Future<Output = Result<(), String>> + Send>>;

    /// Stream of application events
    pub type EventStream = Pin<Box<dyn futures::Stream<Item = AppEvent> + Send>>;
}

#[cfg(target_arch = "wasm32")]
mod bounds {
    use super::*;

    /// Result type for async transport operations (WASM - no Send required)
    pub type TransportResult<T> = Pin<Box<dyn Future<Output = Result<T, String>>>>;

    /// Result type for async transport operations (no return value, WASM - no Send required)
    pub type TransportResultVoid = Pin<Box<dyn Future<Output = Result<(), String>>>>;

    /// Stream of application events (WASM - no Send required)
    pub type EventStream = Pin<Box<dyn futures::Stream<Item = AppEvent>>>;
}

pub use bounds::*;

/// Platform-agnostic transport for server communication
///
/// Implemented by platform-native (tokio-tungstenite) and platform-web (web-sys::WebSocket)
///
/// Note: Methods return boxed futures to avoid async_trait lifetime issues while
/// supporting both Send (native) and !Send (web) implementations.
pub trait Transport: Send + Sync + 'static {
    /// Connect to the server at the given URL
    fn connect(&self, url: String, event_bus: Arc<dyn EventBus>) -> TransportResultVoid;

    /// Disconnect from the server
    fn disconnect(&self) -> TransportResultVoid;

    /// Send a chat message
    fn send_chat(
        &self,
        conv_id: String,
        text: String,
        image: Option<ImagePayload>,
    ) -> TransportResult<String>;

    /// Request the list of conversations
    fn send_list_conversations(&self) -> TransportResultVoid;

    /// Request message history for a conversation
    fn send_get_history(&self, conv_id: String, limit: Option<u32>) -> TransportResultVoid;

    /// Create a new conversation
    fn send_create_conversation(&self, title: Option<String>) -> TransportResultVoid;

    /// Delete a conversation
    fn send_delete_conversation(&self, conv_id: String) -> TransportResultVoid;

    /// Check if currently connected
    fn is_connected(&self) -> bool;
}

/// Platform-agnostic event bus for cross-feature communication
///
/// Implemented by platform-native (tokio::sync::broadcast) and platform-web (futures-channel)
pub trait EventBus: Send + Sync + 'static {
    /// Publish an event to all subscribers
    fn publish(&self, event: AppEvent);

    /// Subscribe to events, returning a stream of events
    fn subscribe(&self) -> EventStream;
}

/// Shared transport handle
pub type SharedTransport = Arc<dyn Transport>;

/// Shared event bus handle
pub type SharedEventBus = Arc<dyn EventBus>;
