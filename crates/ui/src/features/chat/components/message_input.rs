//! Message input component

use dioxus::prelude::*;

/// Message input with send and media buttons
#[component]
pub fn MessageInput(
    value: String,
    on_change: EventHandler<String>,
    on_send: EventHandler<()>,
    on_media_select: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            class: "shrink-0 py-3 px-4 bg-bg-secondary border-t border-border flex gap-2 items-center",

            // Media upload button
            button {
                onclick: move |_| on_media_select.call(()),
                class: "w-11 min-w-11 h-11 rounded-full border-none cursor-pointer flex items-center justify-center shrink-0 bg-bg-tertiary text-text-white text-xl",
                "+"
            }

            // Text input - use min-w-0 to allow flex shrinking properly
            input {
                r#type: "text",
                value: "{value}",
                placeholder: "Type a message...",
                oninput: move |e| on_change.call(e.value()),
                onkeypress: move |e| {
                    if e.key() == Key::Enter {
                        on_send.call(());
                    }
                },
                class: "flex-1 min-w-0 h-11 px-4 border-none rounded-full bg-bg-tertiary text-text-white text-base outline-none box-border",
            }

            // Send button
            button {
                onclick: move |_| on_send.call(()),
                disabled: value.trim().is_empty(),
                class: "w-11 min-w-11 h-11 rounded-full border-none cursor-pointer flex items-center justify-center shrink-0 bg-accent text-text-white disabled:opacity-50",
                svg {
                    width: "24",
                    height: "24",
                    view_box: "0 0 24 24",
                    fill: "currentColor",
                    path {
                        d: "M2.01 21L23 12 2.01 3 2 10l15 2-15 2z"
                    }
                }
            }
        }
    }
}
