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
            style: "flex-shrink: 0; padding: 12px 16px; background: #1a1a2e; color: white; display: flex; align-items: center; gap: 12px; border-bottom: 1px solid #2d2d44;",

            // Back button
            button {
                onclick: move |_| on_back.call(()),
                style: "background: none; border: none; color: white; cursor: pointer; padding: 8px; margin: -8px;",
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
                style: "margin: 0; font-size: 1.125rem; flex: 1; overflow: hidden; text-overflow: ellipsis; white-space: nowrap;",
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
