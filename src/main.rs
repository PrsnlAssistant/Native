//! Personal Assistant Native App
//!
//! A Dioxus-based mobile application that connects to the PrsnlAssistant
//! backend via WebSocket for chat functionality with multiple conversations.

mod websocket;
mod components;
mod state;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use state::{AppState, Message, ConnectionStatus, ViewState};
use components::{ChatView, ChatHeader, ConversationList, ConnectionIndicator, MessageInput, TypingIndicator};

fn main() {
    // Initialize logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    info!("Starting Personal Assistant app");

    // Launch the app
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // Global app state
    let mut app_state = use_signal(|| AppState::new());
    let mut input_text = use_signal(|| String::new());

    // WebSocket connection effect
    use_effect(move || {
        let mut state = app_state.clone();
        spawn(async move {
            // Try to connect to WebSocket server
            let server_url = "ws://10.0.0.1:8765/ws"; // Update with your VPN IP
            info!("Connecting to WebSocket server: {}", server_url);

            match websocket::connect(server_url).await {
                Ok(ws) => {
                    state.write().connection_status = ConnectionStatus::Connected;
                    info!("Connected to server");

                    // Start message handling loop
                    websocket::handle_messages(ws, state).await;
                }
                Err(e) => {
                    info!("Failed to connect: {:?}", e);
                    state.write().connection_status = ConnectionStatus::Disconnected;
                    state.write().loading_conversations = false;
                }
            }
        });
    });

    // Handler for selecting a conversation
    let on_select_conversation = move |conv_id: String| {
        info!("Opening conversation: {}", conv_id);
        app_state.write().open_conversation(&conv_id);

        // Request history for this conversation
        let id = conv_id.clone();
        spawn(async move {
            if let Err(e) = websocket::send_get_history(&id, Some(50)).await {
                info!("Failed to get history: {:?}", e);
            }
        });
    };

    // Handler for creating new conversation
    let on_new_conversation = move |_| {
        info!("Creating new conversation");
        spawn(async move {
            if let Err(e) = websocket::send_create_conversation(None).await {
                info!("Failed to create conversation: {:?}", e);
            }
        });
    };

    // Handler for going back to conversation list
    let on_back = move |_| {
        app_state.write().go_to_list();
        input_text.set(String::new());
    };

    // Send message handler
    let send_message = move |_| {
        let text = input_text.read().clone();
        if text.trim().is_empty() {
            return;
        }

        let state_read = app_state.read();
        let conv_id = match state_read.current_conversation_id() {
            Some(id) => id.to_string(),
            None => return,
        };
        drop(state_read);

        // Add user message to state
        let msg = Message::new_user(text.clone());
        {
            let mut state_write = app_state.write();
            if let Some(conv) = state_write.conversations.get_mut(&conv_id) {
                conv.add_user_message(msg.clone());
            }
        }

        // Clear input
        input_text.set(String::new());

        // Send via WebSocket
        spawn(async move {
            if let Err(e) = websocket::send_message(&conv_id, &text).await {
                info!("Failed to send message: {:?}", e);
            }
        });
    };

    // Get current view state
    let view_state = app_state.read().view.clone();

    match view_state {
        ViewState::ConversationList => {
            let conversations = app_state.read().sorted_conversations()
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();
            let loading = app_state.read().loading_conversations;
            let status = app_state.read().connection_status.clone();

            rsx! {
                div {
                    class: "app-container",
                    style: "display: flex; flex-direction: column; height: 100vh; font-family: system-ui, -apple-system, sans-serif; background: #0f0f23;",

                    // Header
                    header {
                        style: "padding: 16px; background: #1a1a2e; color: white; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #2d2d44;",
                        h1 {
                            style: "margin: 0; font-size: 1.25rem;",
                            "PrsnlAssistant"
                        }
                        ConnectionIndicator { status }
                    }

                    // Conversation list
                    ConversationList {
                        conversations,
                        loading,
                        on_select: on_select_conversation,
                        on_new: on_new_conversation,
                    }
                }
            }
        }
        ViewState::Chat(conv_id) => {
            let state_read = app_state.read();
            let conv = state_read.conversations.get(&conv_id);
            let title = conv.map(|c| c.title.clone()).unwrap_or_else(|| "Chat".to_string());
            let messages = conv.map(|c| c.messages.clone()).unwrap_or_default();
            let status = state_read.connection_status.clone();
            let is_typing = state_read.is_typing;
            drop(state_read);

            rsx! {
                div {
                    class: "app-container",
                    style: "display: flex; flex-direction: column; height: 100vh; font-family: system-ui, -apple-system, sans-serif; background: #0f0f23;",

                    // Chat header with back button
                    ChatHeader {
                        title,
                        on_back,
                        status,
                    }

                    // Chat messages area
                    div {
                        style: "flex: 1; overflow-y: auto; padding: 16px; background: #0f0f23;",
                        id: "chat-container",

                        if messages.is_empty() {
                            div {
                                style: "display: flex; flex-direction: column; align-items: center; justify-content: center; height: 100%; color: #888;",
                                p { "Start a conversation" }
                                p {
                                    style: "font-size: 0.875rem;",
                                    "Type a message below"
                                }
                            }
                        } else {
                            ChatView { messages }
                        }

                        if is_typing {
                            TypingIndicator {}
                        }
                    }

                    // Message input area
                    MessageInput {
                        value: input_text.read().clone(),
                        on_change: move |new_value: String| input_text.set(new_value),
                        on_send: send_message,
                    }
                }
            }
        }
    }
}
