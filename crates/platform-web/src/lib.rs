//! Web platform adapter for PrsnlAssistant
//!
//! Provides Transport and EventBus implementations using web-sys and futures-channel.

pub mod events;
pub mod transport;

pub use events::WebEventBus;
pub use transport::WebTransport;
