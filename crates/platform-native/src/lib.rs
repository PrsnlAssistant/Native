//! Native platform adapter for PrsnlAssistant
//!
//! Provides Transport and EventBus implementations using tokio and tungstenite.

pub mod events;
pub mod transport;

pub use events::NativeEventBus;
pub use transport::NativeTransport;
