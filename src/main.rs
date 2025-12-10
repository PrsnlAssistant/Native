//! Personal Assistant Native App
//!
//! A Dioxus-based mobile application that connects to the PrsnlAssistant
//! backend via WebSocket for chat functionality with multiple conversations.

mod websocket;
mod components;
mod state;
mod media;

use dioxus::prelude::*;
use dioxus_logger::tracing::{info, Level};
use state::{AppState, Message, ImageData, ConnectionStatus, ViewState};
use components::{ChatView, ChatHeader, ConversationList, ConnectionIndicator, MessageInput, TypingIndicator, ServerUrlModal, MediaPreview};

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

    // Server URL and settings modal state
    let mut server_url = use_signal(|| "ws://10.8.0.8:8765/ws".to_string());
    let mut show_settings_modal = use_signal(|| false);
    let mut reconnect_trigger = use_signal(|| 0u32); // Increment to trigger reconnect

    // Pending media attachment
    let mut pending_media = use_signal(|| Option::<media::SelectedMedia>::None);

    // WebSocket connection effect - re-runs when reconnect_trigger changes
    use_effect(move || {
        let _trigger = reconnect_trigger.read(); // Subscribe to reconnect_trigger
        let url = server_url.read().clone();
        let mut state = app_state.clone();

        spawn(async move {
            // Set connecting status
            state.write().connection_status = ConnectionStatus::Connecting;

            info!("Connecting to WebSocket server: {}", url);

            match websocket::connect(&url).await {
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

    // Handler for opening settings modal
    let on_settings_tap = move |_| {
        show_settings_modal.set(true);
    };

    // Handler for closing settings modal
    let on_settings_close = move |_| {
        show_settings_modal.set(false);
    };

    // Handler for saving new server URL
    let on_settings_save = move |new_url: String| {
        info!("Changing server URL to: {}", new_url);
        server_url.set(new_url);
        show_settings_modal.set(false);
        // Trigger reconnection
        let current = *reconnect_trigger.read();
        reconnect_trigger.set(current.wrapping_add(1));
    };

    // Send message handler
    let send_message = move |_| {
        let text = input_text.read().clone();
        let media = pending_media.read().clone();

        // Allow sending if there's text or media
        if text.trim().is_empty() && media.is_none() {
            return;
        }

        let state_read = app_state.read();
        let conv_id = match state_read.current_conversation_id() {
            Some(id) => id.to_string(),
            None => return,
        };
        drop(state_read);

        // Create message with or without image
        let msg = if let Some(ref m) = media {
            Message::new_user_with_image(
                text.clone(),
                ImageData {
                    data: m.data.clone(),
                    mimetype: m.mimetype.clone(),
                },
            )
        } else {
            Message::new_user(text.clone())
        };

        // Add user message to state
        {
            let mut state_write = app_state.write();
            if let Some(conv) = state_write.conversations.get_mut(&conv_id) {
                conv.add_user_message(msg.clone());
            }
        }

        // Clear input and pending media
        input_text.set(String::new());
        pending_media.set(None);

        // Prepare image payload for websocket
        let image_payload = media.map(|m| websocket::ImagePayload {
            data: m.data,
            mimetype: m.mimetype,
        });

        // Send via WebSocket
        spawn(async move {
            if let Err(e) = websocket::send_message_with_image(&conv_id, &text, image_payload).await {
                info!("Failed to send message: {:?}", e);
            }
        });
    };

    // Media picker handler
    let on_media_select = move |_| {
        spawn(async move {
            info!("Opening media picker...");
            if let Some(selected) = media::pick_image().await {
                info!("Selected image: {} ({})", selected.filename, selected.mimetype);
                pending_media.set(Some(selected));
            } else {
                info!("No image selected");
            }
        });
    };

    // Get current view state
    let view_state = app_state.read().view.clone();
    let show_modal = *show_settings_modal.read();
    let current_url = server_url.read().clone();

    // Main content based on view state
    let main_content = match view_state {
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
                    style: "display: flex; flex-direction: column; height: 100vh; height: 100dvh; min-height: 100%; font-family: system-ui, -apple-system, sans-serif; background: #0f0f23;",

                    // Header
                    header {
                        style: "flex-shrink: 0; padding: 16px; background: #1a1a2e; color: white; display: flex; justify-content: space-between; align-items: center; border-bottom: 1px solid #2d2d44;",
                        h1 {
                            style: "margin: 0; font-size: 1.25rem;",
                            "PrsnlAssistant"
                        }
                        ConnectionIndicator { status, on_tap: on_settings_tap }
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
                    style: "display: flex; flex-direction: column; height: 100vh; height: 100dvh; min-height: 100%; font-family: system-ui, -apple-system, sans-serif; background: #0f0f23;",

                    // Chat header with back button
                    ChatHeader {
                        title,
                        on_back,
                        status,
                        on_status_tap: on_settings_tap,
                    }

                    // Chat messages area
                    div {
                        style: "flex: 1; overflow-y: auto; padding: 16px; background: #0f0f23; min-height: 0;",
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

                    // Media preview (if pending)
                    if pending_media.read().is_some() {
                        MediaPreview {
                            media: pending_media.read().clone().unwrap(),
                            on_remove: move |_| pending_media.set(None),
                        }
                    }

                    // Message input area
                    MessageInput {
                        value: input_text.read().clone(),
                        on_change: move |new_value: String| input_text.set(new_value),
                        on_send: send_message,
                        on_media_select,
                    }
                }
            }
        }
    };

    // Render main content + modal overlay
    rsx! {
        {main_content}

        if show_modal {
            ServerUrlModal {
                current_url,
                on_save: on_settings_save,
                on_close: on_settings_close,
            }
        }
    }
}
