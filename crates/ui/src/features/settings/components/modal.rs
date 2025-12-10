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
            class: "fixed inset-0 bg-black/70 flex items-center justify-center z-[1000]",

            // Modal content
            div {
                onclick: move |e| e.stop_propagation(),
                class: "bg-bg-secondary rounded-2xl p-6 w-[90%] max-w-[400px] shadow-2xl",

                // Header
                h2 {
                    class: "text-text-white m-0 mb-4 text-xl",
                    "Server Settings"
                }

                // URL input
                div {
                    class: "mb-4",
                    label {
                        class: "block text-text-muted text-sm mb-2",
                        "WebSocket URL"
                    }
                    input {
                        r#type: "text",
                        value: "{url_input}",
                        oninput: move |e| url_input.set(e.value()),
                        placeholder: "ws://hostname:port/ws",
                        class: "w-full p-3 border border-border rounded-lg bg-bg-primary text-text-white text-base box-border outline-none focus:border-accent",
                    }
                }

                // Help text
                p {
                    class: "text-text-muted text-xs mb-6",
                    "Enter the WebSocket server address. Changes will trigger a reconnection."
                }

                // Buttons
                div {
                    class: "flex gap-3 justify-end",

                    button {
                        onclick: move |_| on_close.call(()),
                        class: "py-3 px-6 border border-border rounded-lg bg-transparent text-text-white cursor-pointer text-base hover:bg-bg-hover transition-colors",
                        "Cancel"
                    }

                    button {
                        onclick: handle_save,
                        class: "py-3 px-6 border-none rounded-lg bg-accent text-text-white cursor-pointer text-base hover:bg-accent-hover transition-colors",
                        "Save"
                    }
                }
            }
        }
    }
}
