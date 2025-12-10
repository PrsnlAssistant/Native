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
    let button_style = "width: 44px; min-width: 44px; height: 44px; border-radius: 22px; border: none; cursor: pointer; display: flex; align-items: center; justify-content: center; flex-shrink: 0;";

    rsx! {
        div {
            style: "flex-shrink: 0; padding: 12px 16px; background: #1a1a2e; border-top: 1px solid #2d2d44; display: flex; gap: 8px; align-items: center;",

            // Media upload button
            button {
                onclick: move |_| on_media_select.call(()),
                style: "{button_style} background: #2d2d44; color: white; font-size: 20px;",
                "+"
            }

            // Text input - use min-width: 0 to allow flex shrinking properly
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
                style: "flex: 1; min-width: 0; height: 44px; padding: 0 16px; border: none; border-radius: 22px; background: #2d2d44; color: white; font-size: 1rem; outline: none; box-sizing: border-box;",
            }

            // Send button
            button {
                onclick: move |_| on_send.call(()),
                disabled: value.trim().is_empty(),
                style: "{button_style} background: #1e88e5; color: white;",
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
