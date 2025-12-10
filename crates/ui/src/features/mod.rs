//! Feature modules for the UI crate
//!
//! Each feature encapsulates its own state, services, and components.

pub mod chat;
pub mod conversations;
pub mod settings;
pub mod media;

// Re-export commonly used types
pub use chat::{ChatService, ChatState, provide_chat_feature};
pub use chat::components::{ChatScreen, ChatHeader, MessageList, MessageBubble, MessageInput, TypingIndicator};
pub use conversations::{ConversationsService, ConversationsState, ViewState, provide_conversations_feature};
pub use conversations::components::{ConversationList, ConversationItem};
pub use settings::{SettingsService, SettingsState, provide_settings_feature};
pub use settings::components::ServerUrlModal;
pub use media::{MediaPreview, SelectedMedia, pick_image};
