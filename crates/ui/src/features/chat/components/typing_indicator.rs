//! Typing indicator component

use dioxus::prelude::*;

/// Animated typing indicator
#[component]
pub fn TypingIndicator() -> Element {
    rsx! {
        div {
            class: "flex items-center py-2 px-4 mb-3",

            div {
                class: "bg-bg-tertiary py-3 px-4 rounded-2xl flex gap-1",

                span {
                    class: "w-2 h-2 bg-text-muted rounded-full animate-bounce-dot",
                }
                span {
                    class: "w-2 h-2 bg-text-muted rounded-full animate-bounce-dot animation-delay-150",
                }
                span {
                    class: "w-2 h-2 bg-text-muted rounded-full animate-bounce-dot animation-delay-300",
                }
            }
        }
    }
}
