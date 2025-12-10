//! Conversation list component

use dioxus::prelude::*;
use prsnl_core::Conversation;
use super::item::ConversationItem;

/// List of conversations with new chat button
#[component]
pub fn ConversationList(
    conversations: Vec<Conversation>,
    loading: bool,
    on_select: EventHandler<String>,
    on_new: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "flex: 1; overflow-y: auto;",

            if loading {
                div {
                    style: "display: flex; justify-content: center; padding: 32px; color: #888;",
                    "Loading conversations..."
                }
            } else if conversations.is_empty() {
                div {
                    style: "display: flex; flex-direction: column; align-items: center; justify-content: center; padding: 32px; color: #888;",
                    p { "No conversations yet" }
                    p {
                        style: "font-size: 0.875rem;",
                        "Tap the button below to start"
                    }
                }
            } else {
                for conv in conversations {
                    ConversationItem {
                        key: "{conv.id}",
                        conversation: conv.clone(),
                        on_select,
                    }
                }
            }
        }

        // New chat button
        button {
            onclick: move |_| on_new.call(()),
            style: "position: fixed; bottom: 24px; right: 24px; width: 56px; height: 56px; border-radius: 28px; background: #1e88e5; color: white; border: none; font-size: 24px; cursor: pointer; box-shadow: 0 4px 12px rgba(0,0,0,0.3); display: flex; align-items: center; justify-content: center;",
            "+"
        }
    }
}
