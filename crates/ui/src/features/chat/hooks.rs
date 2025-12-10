//! Custom hooks for the chat feature

use dioxus::prelude::*;
use prsnl_core::Message;
use crate::features::media::SelectedMedia;
use super::{ChatState, ChatService};

/// Hook to get messages for the current conversation (reactive)
///
/// Returns a reactive memo that updates when messages change.
pub fn use_chat_messages() -> Memo<Vec<Message>> {
    let state = use_context::<ChatState>();
    use_memo(move || state.current_messages())
}

/// Hook to get messages for a specific conversation (reactive)
///
/// Returns a reactive memo that updates when messages for this conversation change.
pub fn use_messages_for(conv_id: &str) -> Memo<Vec<Message>> {
    let state = use_context::<ChatState>();
    let conv_id = conv_id.to_string();
    use_memo(move || state.messages_for(&conv_id))
}

/// Hook to check if assistant is typing (reactive)
///
/// Returns a reactive memo that updates when typing state changes.
pub fn use_typing_indicator() -> Memo<bool> {
    let state = use_context::<ChatState>();
    use_memo(move || state.is_typing())
}

/// Hook to get a send message function
pub fn use_send_message() -> impl Fn(String, Option<SelectedMedia>) + Clone {
    let service = use_context::<ChatService>();

    move |text: String, media: Option<SelectedMedia>| {
        service.send_message(text, media);
    }
}

/// Hook to get current conversation ID (reactive)
///
/// Returns a reactive memo that updates when the current conversation changes.
pub fn use_current_conversation_id() -> Memo<Option<String>> {
    let state = use_context::<ChatState>();
    use_memo(move || state.current_conv_id())
}
