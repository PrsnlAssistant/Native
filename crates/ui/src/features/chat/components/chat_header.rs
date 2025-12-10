//! Chat header component

use dioxus::prelude::*;
use prsnl_core::ConnectionStatus;
use crate::shared::ConnectionIndicator;

/// Chat header with back button and title
#[component]
pub fn ChatHeader(
    title: String,
    status: ConnectionStatus,
    on_back: EventHandler<()>,
    on_status_tap: EventHandler<()>,
) -> Element {
    rsx! {
        header {
            class: "shrink-0 py-3 px-4 bg-bg-secondary text-text-white flex items-center gap-3 border-b border-border",

            // Back button
            button {
                onclick: move |_| on_back.call(()),
                class: "bg-transparent border-none text-text-white cursor-pointer p-2 -m-2",
                svg {
                    width: "24",
                    height: "24",
                    view_box: "0 0 24 24",
                    fill: "currentColor",
                    path {
                        d: "M20 11H7.83l5.59-5.59L12 4l-8 8 8 8 1.41-1.41L7.83 13H20v-2z"
                    }
                }
            }

            // Title
            h1 {
                class: "m-0 text-lg flex-1 overflow-hidden text-ellipsis whitespace-nowrap",
                "{title}"
            }

            // Connection status
            ConnectionIndicator {
                status,
                on_tap: on_status_tap,
            }
        }
    }
}
