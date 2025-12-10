//! Server URL settings modal

use dioxus::prelude::*;

/// Modal for editing server URL
#[component]
pub fn ServerUrlModal(
    current_url: String,
    on_save: EventHandler<String>,
    on_close: EventHandler<()>,
) -> Element {
    let mut url_input = use_signal(|| current_url.clone());

    let handle_save = move |_| {
        on_save.call(url_input.read().clone());
    };

    rsx! {
        // Backdrop
        div {
            onclick: move |_| on_close.call(()),
            style: "position: fixed; inset: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",

            // Modal content
            div {
                onclick: move |e| e.stop_propagation(),
                style: "background: #1a1a2e; border-radius: 16px; padding: 24px; width: 90%; max-width: 400px; box-shadow: 0 8px 32px rgba(0,0,0,0.5);",

                // Header
                h2 {
                    style: "color: white; margin: 0 0 16px 0; font-size: 1.25rem;",
                    "Server Settings"
                }

                // URL input
                div {
                    style: "margin-bottom: 16px;",
                    label {
                        style: "display: block; color: #888; font-size: 0.875rem; margin-bottom: 8px;",
                        "WebSocket URL"
                    }
                    input {
                        r#type: "text",
                        value: "{url_input}",
                        oninput: move |e| url_input.set(e.value()),
                        placeholder: "ws://hostname:port/ws",
                        style: "width: 100%; padding: 12px; border: 1px solid #2d2d44; border-radius: 8px; background: #0f0f23; color: white; font-size: 1rem; box-sizing: border-box;",
                    }
                }

                // Help text
                p {
                    style: "color: #666; font-size: 0.75rem; margin-bottom: 24px;",
                    "Enter the WebSocket server address. Changes will trigger a reconnection."
                }

                // Buttons
                div {
                    style: "display: flex; gap: 12px; justify-content: flex-end;",

                    button {
                        onclick: move |_| on_close.call(()),
                        style: "padding: 12px 24px; border: 1px solid #2d2d44; border-radius: 8px; background: transparent; color: white; cursor: pointer; font-size: 1rem;",
                        "Cancel"
                    }

                    button {
                        onclick: handle_save,
                        style: "padding: 12px 24px; border: none; border-radius: 8px; background: #1e88e5; color: white; cursor: pointer; font-size: 1rem;",
                        "Save"
                    }
                }
            }
        }
    }
}
