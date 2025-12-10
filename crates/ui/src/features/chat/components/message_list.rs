//! Message list component

use dioxus::prelude::*;
use prsnl_core::Message;
use super::message_bubble::MessageBubble;

/// List of messages in a chat
#[component]
pub fn MessageList(messages: Vec<Message>) -> Element {
    rsx! {
        div {
            for message in messages {
                MessageBubble {
                    key: "{message.id}",
                    message,
                }
            }
        }
    }
}
