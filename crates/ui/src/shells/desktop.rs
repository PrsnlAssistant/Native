//! Desktop shell with side-by-side layout
//!
//! Shows conversation list and chat panel simultaneously in a flexbox layout.
//! Optimized for larger screens (768px and above) with mouse interaction.
//!
//! Layout:
//! ```text
//! +------------+-------------------------+
//! |  Sidebar   |     Chat Panel          |
//! |  300px     |     flex: 1             |
//! |            |                         |
//! |  Chat 1    |  Header                 |
//! |  Chat 2    |  Messages...            |
//! |  [+]       |  Input                  |
//! +------------+-------------------------+
//! ```

use dioxus::prelude::*;
use prsnl_core::ConnectionStatus;

use crate::features::{
    ConversationList, ConversationsService, ConversationsState,
    ChatScreen,
};

/// Sidebar width in pixels
const SIDEBAR_WIDTH: &str = "300px";

/// Desktop shell with sidebar and main content area
///
/// This component renders a side-by-side layout with:
/// - A fixed-width sidebar (300px) showing the conversation list
/// - A flexible main content area showing the active chat
///
/// # Example
///
/// ```rust,ignore
/// use dioxus::prelude::*;
/// use prsnl_ui::DesktopShell;
///
/// fn App() -> Element {
///     rsx! { DesktopShell {} }
/// }
/// ```
#[component]
pub fn DesktopShell() -> Element {
    // Get state and services from context
    let conv_state: ConversationsState = use_context();
    let conv_service: ConversationsService = use_context();
    let connection_status: Signal<ConnectionStatus> = use_context();

    // Get current conversation ID (if any)
    let current_conv_id = conv_state.current_conversation_id();

    // Navigation callbacks
    let on_select = {
        let conv_service = conv_service.clone();
        move |conv_id: String| {
            conv_service.select_conversation(&conv_id);
        }
    };

    let on_new = {
        let conv_service = conv_service.clone();
        move |_| {
            conv_service.create_conversation(None);
        }
    };

    rsx! {
        div {
            class: "desktop-shell",
            style: "height: 100vh; display: flex; background-color: #0f0f23; color: #e0e0e0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;",

            // Sidebar - conversation list
            aside {
                class: "desktop-sidebar",
                style: "width: {SIDEBAR_WIDTH}; min-width: {SIDEBAR_WIDTH}; border-right: 1px solid #2d2d44; display: flex; flex-direction: column; background-color: #1a1a2e;",

                // Header with title
                SidebarHeader {}

                // Real conversation list
                ConversationList {
                    conversations: conv_state.sorted_conversations(),
                    loading: conv_state.is_loading(),
                    on_select: on_select,
                    on_new: on_new,
                }
            }

            // Main content - chat panel
            main {
                class: "desktop-main",
                style: "flex: 1; display: flex; flex-direction: column; overflow: hidden; background-color: #0f0f23;",

                if let Some(conv_id) = current_conv_id {
                    // Get conversation title
                    {
                        let title = conv_state
                            .get_conversation(&conv_id)
                            .map(|c| if c.title.is_empty() { "New Chat".to_string() } else { c.title.clone() })
                            .unwrap_or_else(|| "Chat".to_string());

                        rsx! {
                            ChatScreen {
                                conv_id: conv_id.clone(),
                                title: title,
                                status: connection_status.read().clone(),
                                on_back: move |_| {
                                    // On desktop, back just deselects (no navigation needed)
                                    tracing::info!("Back pressed on desktop (no-op)");
                                },
                                on_status_tap: move |_| {
                                    tracing::info!("Status indicator tapped");
                                },
                            }
                        }
                    }
                } else {
                    // No conversation selected - show placeholder
                    EmptyState {}
                }
            }
        }
    }
}

/// Sidebar header with title
#[component]
fn SidebarHeader() -> Element {
    rsx! {
        div {
            class: "sidebar-header",
            style: "flex-shrink: 0; padding: 16px; border-bottom: 1px solid #2d2d44; display: flex; justify-content: space-between; align-items: center;",

            h2 {
                style: "margin: 0; font-size: 18px; font-weight: 600; color: #ffffff;",
                "Conversations"
            }
        }
    }
}

/// Empty state when no conversation is selected
#[component]
fn EmptyState() -> Element {
    rsx! {
        div {
            class: "empty-state",
            style: "flex: 1; display: flex; flex-direction: column; align-items: center; justify-content: center; color: #6b6b8a;",

            // Placeholder icon
            div {
                style: "font-size: 64px; margin-bottom: 24px; opacity: 0.5;",
                "💬"
            }

            p {
                style: "font-size: 18px; margin: 0 0 8px; color: #8888a8;",
                "No conversation selected"
            }
            p {
                style: "font-size: 14px; margin: 0;",
                "Select a conversation from the sidebar or create a new one"
            }
        }
    }
}
