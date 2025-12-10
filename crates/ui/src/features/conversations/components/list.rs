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
            class: "flex-1 overflow-y-auto",

            if loading {
                div {
                    class: "flex justify-center p-8 text-text-muted",
                    "Loading conversations..."
                }
            } else if conversations.is_empty() {
                div {
                    class: "flex flex-col items-center justify-center p-8 text-text-muted",
                    p { "No conversations yet" }
                    p {
                        class: "text-sm",
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

        // New chat button (FAB)
        button {
            onclick: move |_| on_new.call(()),
            class: "fixed bottom-6 right-6 w-14 h-14 rounded-full bg-accent text-text-white border-none text-2xl cursor-pointer shadow-lg flex items-center justify-center hover:bg-accent-hover transition-colors",
            "+"
        }
    }
}
