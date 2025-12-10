//! Connection status indicator component

use dioxus::prelude::*;
use prsnl_core::ConnectionStatus;

/// Connection indicator that shows current WebSocket status
#[component]
pub fn ConnectionIndicator(
    status: ConnectionStatus,
    on_tap: EventHandler<()>,
) -> Element {
    let (dot_class, text, btn_class) = match status {
        ConnectionStatus::Connected => ("bg-success", "Connected", ""),
        ConnectionStatus::Connecting => ("bg-warning animate-pulse-status", "Connecting...", ""),
        ConnectionStatus::Reconnecting => ("bg-warning animate-pulse-status", "Reconnecting...", ""),
        ConnectionStatus::Disconnected => ("bg-error", "Disconnected", ""),
    };

    rsx! {
        button {
            onclick: move |_| on_tap.call(()),
            class: "bg-transparent border-none cursor-pointer flex items-center gap-1.5 p-2 {btn_class}",
            span {
                class: "w-2 h-2 rounded-full {dot_class}",
            }
            span {
                class: "text-text-muted text-xs",
                "{text}"
            }
        }
    }
}
