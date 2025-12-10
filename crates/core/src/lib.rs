//! Core types and traits for PrsnlAssistant
//!
//! This crate provides platform-agnostic types, protocol definitions,
//! and trait abstractions used by all platform implementations.

pub mod events;
pub mod protocol;
pub mod traits;
pub mod types;

// Re-export commonly used types at crate root
pub use events::AppEvent;
pub use protocol::{
    ConversationInfo, HistoryMessage, ImagePayload, WSClientMessage, WSServerMessage,
};
pub use traits::{
    EventBus, EventStream, SharedEventBus, SharedTransport, Transport, TransportResult,
    TransportResultVoid,
};
pub use types::{ConnectionStatus, Conversation, ImageData, Message, MessageSender, MessageStatus};
