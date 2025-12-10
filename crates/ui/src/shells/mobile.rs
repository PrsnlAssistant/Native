//! Mobile shell with navigation-based layout
//!
//! Shows either conversation list OR chat view, not both.
//! Optimized for small screens (< 768px) with touch-friendly targets.
//!
//! Views:
//! - `ConversationList`: Full-screen list of conversations
//! - `Chat`: Full-screen chat view with back button to return to list

use dioxus::prelude::*;
use prsnl_core::ConnectionStatus;

use crate::features::{
    ConversationList, ConversationsService, ConversationsState,
    ChatScreen,
};

/// Mobile view state
#[derive(Clone, Debug, PartialEq)]
enum MobileView {
    /// Showing the conversation list
    ConversationList,
    /// Showing a chat conversation
    Chat { conversation_id: String },
}

impl Default for MobileView {
    fn default() -> Self {
        MobileView::ConversationList
    }
}

/// Mobile shell with navigation between views
///
/// This component manages navigation state and renders either:
/// - A full-screen conversation list
/// - A full-screen chat view with back navigation
///
/// Navigation is driven by:
/// - User interactions (tapping conversations, back button)
/// - ConversationsService (select_conversation, go_back)
///
/// # Example
///
/// ```rust,ignore
/// use dioxus::prelude::*;
/// use prsnl_ui::MobileShell;
///
/// fn App() -> Element {
///     rsx! { MobileShell {} }
/// }
/// ```
#[component]
pub fn MobileShell() -> Element {
    // Get state and services from context
    let conv_state: ConversationsState = use_context();
    let conv_service: ConversationsService = use_context();
    let connection_status: Signal<ConnectionStatus> = use_context();

    // Local view state for navigation (separate from ConversationsState.view for mobile-specific behavior)
    let mut view = use_signal(MobileView::default);

    // Navigation callbacks
    let on_select = {
        let conv_service = conv_service.clone();
        move |conv_id: String| {
            conv_service.select_conversation(&conv_id);
            view.set(MobileView::Chat { conversation_id: conv_id });
        }
    };

    let on_new = {
        let conv_service = conv_service.clone();
        move |_| {
            conv_service.create_conversation(None);
        }
    };

    let on_back = {
        let conv_service = conv_service.clone();
        move |_| {
            conv_service.go_back();
            view.set(MobileView::ConversationList);
        }
    };

    rsx! {
        div {
            class: "mobile-shell",
            style: "height: 100vh; height: 100dvh; display: flex; flex-direction: column; background-color: #0f0f23; color: #e0e0e0; font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;",

            match view.read().clone() {
                MobileView::ConversationList => rsx! {
                    // Header
                    MobileListHeader {}

                    // Real conversation list from features
                    ConversationList {
                        conversations: conv_state.sorted_conversations(),
                        loading: conv_state.is_loading(),
                        on_select: on_select,
                        on_new: on_new,
                    }
                },
                MobileView::Chat { conversation_id } => {
                    // Get conversation title
                    let title = conv_state
                        .get_conversation(&conversation_id)
                        .map(|c| if c.title.is_empty() { "New Chat".to_string() } else { c.title.clone() })
                        .unwrap_or_else(|| "Chat".to_string());

                    rsx! {
                        // Real chat screen from features
                        ChatScreen {
                            conv_id: conversation_id.clone(),
                            title: title,
                            status: connection_status.read().clone(),
                            on_back: on_back,
                            on_status_tap: move |_| {
                                tracing::info!("Status indicator tapped");
                            },
                        }
                    }
                },
            }
        }
    }
}

/// Header for mobile list view
#[component]
fn MobileListHeader() -> Element {
    rsx! {
        header {
            class: "mobile-list-header",
            style: "flex-shrink: 0; padding: 16px; border-bottom: 1px solid #2d2d44; display: flex; justify-content: space-between; align-items: center; background-color: #1a1a2e;",

            h1 {
                style: "margin: 0; font-size: 20px; font-weight: 600; color: #ffffff;",
                "Conversations"
            }

            // Connection indicator could go here
        }
    }
}
