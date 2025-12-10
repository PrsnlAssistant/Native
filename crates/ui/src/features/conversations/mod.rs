//! Conversations feature module
//!
//! This feature manages the list of conversations and navigation between them.

mod state;
mod service;
pub mod components;

pub use state::{ConversationsState, ViewState};
pub use service::ConversationsService;

use std::sync::Arc;
use prsnl_core::{EventBus, Transport};

/// Initialize the conversations feature
pub fn provide_conversations_feature(
    event_bus: Arc<dyn EventBus>,
    transport: Arc<dyn Transport>,
) -> (ConversationsState, ConversationsService) {
    let state = ConversationsState::new();
    let service = ConversationsService::new(state, event_bus, transport);
    (state, service)
}
