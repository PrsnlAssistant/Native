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
            class: "h-screen h-dvh flex bg-bg-primary text-text-primary font-sans",

            // Sidebar - conversation list
            aside {
                class: "w-sidebar min-w-sidebar border-r border-border flex flex-col bg-bg-secondary",

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
                class: "flex-1 flex flex-col overflow-hidden bg-bg-primary",

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
            class: "shrink-0 p-4 border-b border-border flex justify-between items-center",

            h2 {
                class: "m-0 text-lg font-semibold text-text-white",
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
            class: "flex-1 flex flex-col items-center justify-center text-text-muted",

            // Placeholder icon
            div {
                class: "text-6xl mb-6 opacity-50",
                "💬"
            }

            p {
                class: "text-lg m-0 mb-2 text-text-secondary",
                "No conversation selected"
            }
            p {
                class: "text-sm m-0",
                "Select a conversation from the sidebar or create a new one"
            }
        }
    }
}
