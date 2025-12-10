//! UI components and shells for PrsnlAssistant
//!
//! This crate contains all Dioxus UI components and the responsive shell system.

pub mod features;
pub mod shared;
pub mod shells;

pub use shared::ConnectionIndicator;
pub use shells::{DesktopShell, MobileShell, ResponsiveApp};

// Re-export feature types
pub use features::{
    ChatScreen, ChatHeader, MessageList, MessageBubble, MessageInput, TypingIndicator,
    ChatService, ChatState, provide_chat_feature,
    ConversationItem, ConversationList, ConversationsService, ConversationsState,
    ViewState, provide_conversations_feature,
    MediaPreview, SelectedMedia, pick_image,
    ServerUrlModal, SettingsService, SettingsState, provide_settings_feature,
};
