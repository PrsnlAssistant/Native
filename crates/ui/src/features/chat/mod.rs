//! Chat feature module
//!
//! This feature manages chat messages, sending/receiving, and chat UI.

mod state;
mod service;
pub mod hooks;
pub mod components;

pub use state::ChatState;
pub use service::ChatService;

use prsnl_core::{SharedEventBus, SharedTransport};

/// Initialize the chat feature
pub fn provide_chat_feature(
    event_bus: SharedEventBus,
    transport: SharedTransport,
) -> (ChatState, ChatService) {
    let state = ChatState::new();
    let service = ChatService::new(state.clone(), event_bus, transport);
    (state, service)
}
