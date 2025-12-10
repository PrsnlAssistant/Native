//! Application events for cross-feature communication
//!
//! This module defines the event types only. Platform-specific implementations
//! of the event bus are provided by platform-native and platform-web crates.

use crate::types::{ConnectionStatus, Conversation, Message};

/// Application-wide events for cross-feature communication
#[derive(Debug, Clone)]
pub enum AppEvent {
    // Connection events
    ConnectionChanged(ConnectionStatus),

    // Conversation events
    ConversationSelected(String),
    ConversationCreated { id: String, title: Option<String> },
    ConversationDeleted(String),
    ConversationsLoaded(Vec<Conversation>),

    // Chat events
    MessageSent { conv_id: String, message: Message },
    MessageReceived { conv_id: String, message: Message },
    MessageError { conv_id: String, msg_id: String, error: String },
    TypingChanged { conv_id: String, is_typing: bool },
    HistoryLoaded { conv_id: String, messages: Vec<Message> },

    // Settings events
    ServerUrlChanged(String),
    SettingsModalToggled(bool),

    // Navigation events
    NavigateToList,
    NavigateToChat(String),
}
