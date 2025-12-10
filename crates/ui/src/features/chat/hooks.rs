//! Custom hooks for the chat feature

use dioxus::prelude::*;
use prsnl_core::Message;
use crate::features::media::SelectedMedia;
use super::{ChatState, ChatService};

/// Hook to get messages for the current conversation
pub fn use_chat_messages() -> Vec<Message> {
    let state = use_context::<ChatState>();
    state.current_messages()
}

/// Hook to get messages for a specific conversation
pub fn use_messages_for(conv_id: &str) -> Vec<Message> {
    let state = use_context::<ChatState>();
    state.messages_for(conv_id)
}

/// Hook to check if assistant is typing
pub fn use_typing_indicator() -> bool {
    let state = use_context::<ChatState>();
    state.is_typing()
}

/// Hook to get a send message function
pub fn use_send_message() -> impl Fn(String, Option<SelectedMedia>) + Clone {
    let service = use_context::<ChatService>();

    move |text: String, media: Option<SelectedMedia>| {
        service.send_message(text, media);
    }
}

/// Hook to get current conversation ID
pub fn use_current_conversation_id() -> Option<String> {
    let state = use_context::<ChatState>();
    state.current_conv_id()
}
