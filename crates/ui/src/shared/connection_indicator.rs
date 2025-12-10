//! Connection status indicator component

use dioxus::prelude::*;
use prsnl_core::ConnectionStatus;

/// Connection indicator that shows current WebSocket status
#[component]
pub fn ConnectionIndicator(
    status: ConnectionStatus,
    on_tap: EventHandler<()>,
) -> Element {
    let (color, text, class) = match status {
        ConnectionStatus::Connected => ("#4caf50", "Connected", ""),
        ConnectionStatus::Connecting => ("#ff9800", "Connecting...", "status-connecting"),
        ConnectionStatus::Reconnecting => ("#ff9800", "Reconnecting...", "status-reconnecting"),
        ConnectionStatus::Disconnected => ("#f44336", "Disconnected", ""),
    };

    rsx! {
        button {
            onclick: move |_| on_tap.call(()),
            class: "{class}",
            style: "background: none; border: none; cursor: pointer; display: flex; align-items: center; gap: 6px; padding: 8px;",
            span {
                style: "width: 8px; height: 8px; border-radius: 50%; background: {color};",
            }
            span {
                style: "color: #888; font-size: 0.75rem;",
                "{text}"
            }
        }
    }
}
