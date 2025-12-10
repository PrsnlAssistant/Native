//! Typing indicator component

use dioxus::prelude::*;

/// Animated typing indicator
#[component]
pub fn TypingIndicator() -> Element {
    rsx! {
        div {
            style: "display: flex; align-items: center; padding: 8px 16px; margin-bottom: 12px;",

            div {
                style: "background: #2d2d44; padding: 12px 16px; border-radius: 16px; display: flex; gap: 4px;",

                span {
                    class: "typing-dot",
                    style: "width: 8px; height: 8px; background: #888; border-radius: 50%; animation: bounce 1.4s infinite ease-in-out both;",
                }
                span {
                    class: "typing-dot",
                    style: "width: 8px; height: 8px; background: #888; border-radius: 50%; animation: bounce 1.4s infinite ease-in-out both;",
                }
                span {
                    class: "typing-dot",
                    style: "width: 8px; height: 8px; background: #888; border-radius: 50%; animation: bounce 1.4s infinite ease-in-out both;",
                }
            }
        }
    }
}
