//! UI Components for the chat interface

use dioxus::prelude::*;
use crate::state::{ConnectionStatus, Conversation, Message, MessageSender, MessageStatus};

/// Connection status indicator (tappable to change server URL)
#[component]
pub fn ConnectionIndicator(
    status: ConnectionStatus,
    on_tap: EventHandler<()>,
) -> Element {
    let (color, text) = match status {
        ConnectionStatus::Connecting => ("#ffc107", "Connecting..."),
        ConnectionStatus::Connected => ("#28a745", "Connected"),
        ConnectionStatus::Disconnected => ("#dc3545", "Disconnected"),
        ConnectionStatus::Reconnecting => ("#fd7e14", "Reconnecting..."),
    };

    rsx! {
        div {
            style: "display: flex; align-items: center; gap: 8px; cursor: pointer; padding: 4px 8px; border-radius: 8px; background: rgba(255,255,255,0.1);",
            onclick: move |_| on_tap.call(()),
            span {
                style: "width: 10px; height: 10px; border-radius: 50%; background: {color};",
            }
            span {
                style: "font-size: 0.875rem;",
                "{text}"
            }
        }
    }
}

/// Server URL settings modal
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

    let handle_input = move |evt: Event<FormData>| {
        url_input.set(evt.value().clone());
    };

    rsx! {
        // Backdrop
        div {
            style: "position: fixed; top: 0; left: 0; right: 0; bottom: 0; background: rgba(0,0,0,0.7); display: flex; align-items: center; justify-content: center; z-index: 1000;",
            onclick: move |_| on_close.call(()),

            // Modal
            div {
                style: "background: #1a1a2e; border-radius: 16px; padding: 24px; margin: 16px; max-width: 400px; width: 100%;",
                onclick: move |evt| evt.stop_propagation(),

                h2 {
                    style: "margin: 0 0 16px 0; color: #fff; font-size: 1.25rem;",
                    "Server Settings"
                }

                label {
                    style: "display: block; color: #888; font-size: 0.875rem; margin-bottom: 8px;",
                    "WebSocket Server URL"
                }

                input {
                    style: "width: 100%; padding: 12px; border-radius: 8px; border: 1px solid #3d3d5c; background: #0f0f23; color: #fff; font-size: 1rem; box-sizing: border-box;",
                    r#type: "text",
                    value: "{url_input}",
                    placeholder: "ws://10.8.0.8:8765/ws",
                    oninput: handle_input,
                }

                p {
                    style: "margin: 8px 0 16px 0; color: #666; font-size: 0.75rem;",
                    "Example: ws://192.168.1.100:8765/ws"
                }

                div {
                    style: "display: flex; gap: 12px; justify-content: flex-end;",

                    button {
                        style: "padding: 10px 20px; border-radius: 8px; border: 1px solid #3d3d5c; background: transparent; color: #888; cursor: pointer;",
                        onclick: move |_| on_close.call(()),
                        "Cancel"
                    }

                    button {
                        style: "padding: 10px 20px; border-radius: 8px; border: none; background: #1e88e5; color: white; font-weight: 600; cursor: pointer;",
                        onclick: handle_save,
                        "Save & Reconnect"
                    }
                }
            }
        }
    }
}

/// Conversation list view (home screen)
#[component]
pub fn ConversationList(
    conversations: Vec<Conversation>,
    loading: bool,
    on_select: EventHandler<String>,
    on_new: EventHandler<()>,
) -> Element {
    rsx! {
        div {
            style: "flex: 1; overflow-y: auto; background: #0f0f23;",

            // New chat button
            div {
                style: "padding: 16px; border-bottom: 1px solid #2d2d44;",
                button {
                    style: "width: 100%; padding: 12px 20px; border-radius: 12px; border: 2px dashed #3d3d5c; background: transparent; color: #888; font-size: 1rem; cursor: pointer; display: flex; align-items: center; justify-content: center; gap: 8px;",
                    onclick: move |_| on_new.call(()),
                    span { "+ New Chat" }
                }
            }

            if loading {
                div {
                    style: "padding: 20px; text-align: center; color: #888;",
                    "Loading conversations..."
                }
            } else if conversations.is_empty() {
                div {
                    style: "padding: 40px 20px; text-align: center; color: #888;",
                    p { "No conversations yet" }
                    p {
                        style: "font-size: 0.875rem; margin-top: 8px;",
                        "Tap '+ New Chat' to start"
                    }
                }
            } else {
                for conv in conversations.iter() {
                    ConversationItem {
                        conversation: conv.clone(),
                        on_click: move |id: String| on_select.call(id),
                    }
                }
            }
        }
    }
}

/// Individual conversation item in the list
#[component]
fn ConversationItem(
    conversation: Conversation,
    on_click: EventHandler<String>,
) -> Element {
    let conv_id = conversation.id.clone();
    let preview = conversation.last_message_preview
        .clone()
        .unwrap_or_else(|| "No messages yet".to_string());
    let preview_truncated = if preview.len() > 50 {
        format!("{}...", &preview[..50])
    } else {
        preview
    };

    let time_str = conversation.last_message_time
        .map(|t| t.format("%H:%M").to_string())
        .unwrap_or_default();

    rsx! {
        div {
            style: "padding: 16px; border-bottom: 1px solid #2d2d44; cursor: pointer; transition: background 0.2s;",
            onclick: move |_| on_click.call(conv_id.clone()),

            div {
                style: "display: flex; justify-content: space-between; align-items: flex-start;",

                div {
                    style: "flex: 1; min-width: 0;",

                    h3 {
                        style: "margin: 0 0 4px 0; font-size: 1rem; color: #fff; font-weight: 600;",
                        "{conversation.title}"
                    }

                    p {
                        style: "margin: 0; font-size: 0.875rem; color: #888; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{preview_truncated}"
                    }
                }

                div {
                    style: "margin-left: 12px; text-align: right;",

                    span {
                        style: "font-size: 0.75rem; color: #666;",
                        "{time_str}"
                    }

                    if conversation.message_count > 0 {
                        div {
                            style: "margin-top: 4px; font-size: 0.75rem; color: #888;",
                            "{conversation.message_count} messages"
                        }
                    }
                }
            }
        }
    }
}

/// Chat header with back button
#[component]
pub fn ChatHeader(
    title: String,
    on_back: EventHandler<()>,
    status: ConnectionStatus,
    on_status_tap: EventHandler<()>,
) -> Element {
    rsx! {
        header {
            style: "padding: 12px 16px; background: #1a1a2e; color: white; display: flex; align-items: center; gap: 12px; border-bottom: 1px solid #2d2d44;",

            // Back button
            button {
                style: "padding: 8px; border: none; background: transparent; color: white; cursor: pointer; font-size: 1.25rem;",
                onclick: move |_| on_back.call(()),
                "<"
            }

            // Title
            div {
                style: "flex: 1;",
                h1 {
                    style: "margin: 0; font-size: 1.125rem; font-weight: 600;",
                    "{title}"
                }
            }

            // Connection status
            ConnectionIndicator { status, on_tap: on_status_tap }
        }
    }
}

/// Chat messages view
#[component]
pub fn ChatView(messages: Vec<Message>) -> Element {
    rsx! {
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
                for msg in messages.iter() {
                    MessageBubble { message: msg.clone() }
                }
            }
        }
    }
}

/// Individual message bubble
#[component]
fn MessageBubble(message: Message) -> Element {
    let is_user = message.sender == MessageSender::User;
    let is_system = message.sender == MessageSender::System;

    let bg_color = if is_system {
        "#2d2d44"
    } else if is_user {
        "#1e88e5"
    } else {
        "#2d2d44"
    };

    let align = if is_system {
        "center"
    } else if is_user {
        "flex-end"
    } else {
        "flex-start"
    };

    let text_color = if is_system {
        "#888"
    } else if is_user {
        "#fff"
    } else {
        "#e0e0e0"
    };

    let status_icon = match &message.status {
        MessageStatus::Sending => "...",
        MessageStatus::Sent => "",
        MessageStatus::Delivered => "",
        MessageStatus::Error(_) => "!",
    };

    let max_width = if is_system { "100%" } else { "80%" };
    let time_str = message.timestamp.format("%H:%M").to_string();

    // Build image src if present
    let image_src = message.image.as_ref().map(|img| {
        format!("data:{};base64,{}", img.mimetype, img.data)
    });

    rsx! {
        div {
            style: "display: flex; justify-content: {align}; margin-bottom: 12px;",

            div {
                style: "max-width: {max_width}; padding: 12px 16px; border-radius: 16px; background: {bg_color}; color: {text_color};",

                p {
                    style: "margin: 0; white-space: pre-wrap; word-break: break-word;",
                    "{message.body}"
                }

                if let Some(src) = image_src {
                    img {
                        style: "max-width: 100%; border-radius: 8px; margin-top: 8px;",
                        src: "{src}",
                    }
                }

                div {
                    style: "display: flex; justify-content: flex-end; align-items: center; gap: 4px; margin-top: 4px; font-size: 0.75rem; opacity: 0.7;",

                    span { "{time_str}" }

                    if !is_system && is_user {
                        span { "{status_icon}" }
                    }
                }
            }
        }
    }
}

/// Message input component
#[component]
pub fn MessageInput(
    value: String,
    on_change: EventHandler<String>,
    on_send: EventHandler<()>,
    on_media_select: EventHandler<()>,
) -> Element {
    let handle_input = move |evt: Event<FormData>| {
        on_change.call(evt.value().clone());
    };

    let handle_keypress = move |evt: Event<KeyboardData>| {
        if evt.key() == Key::Enter && !evt.modifiers().shift() {
            evt.prevent_default();
            on_send.call(());
        }
    };

    let handle_submit = move |evt: Event<FormData>| {
        evt.prevent_default();
        on_send.call(());
    };

    let is_empty = value.trim().is_empty();
    let send_bg = if is_empty { "#3d3d5c" } else { "#1e88e5" };

    rsx! {
        div {
            style: "padding: 12px 16px; background: #1a1a2e; border-top: 1px solid #2d2d44;",

            form {
                style: "display: flex; gap: 8px; align-items: center;",
                onsubmit: handle_submit,

                // Media upload button
                button {
                    style: "width: 44px; height: 44px; border-radius: 50%; border: none; background: #2d2d44; color: #888; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 1.25rem; flex-shrink: 0;",
                    r#type: "button",
                    onclick: move |_| on_media_select.call(()),
                    "+"
                }

                // Text input - using input instead of textarea for single line
                input {
                    style: "flex: 1; height: 44px; padding: 0 16px; border-radius: 22px; border: 1px solid #3d3d5c; background: #0f0f23; color: #fff; font-family: inherit; font-size: 1rem; box-sizing: border-box;",
                    r#type: "text",
                    placeholder: "Type a message...",
                    value: "{value}",
                    oninput: handle_input,
                    onkeypress: handle_keypress,
                }

                // Send button
                button {
                    style: "height: 44px; padding: 0 20px; border-radius: 22px; border: none; background: {send_bg}; color: white; font-weight: 600; cursor: pointer; flex-shrink: 0;",
                    r#type: "submit",
                    disabled: is_empty,
                    "Send"
                }
            }
        }
    }
}

/// Typing indicator
#[component]
pub fn TypingIndicator() -> Element {
    rsx! {
        div {
            style: "display: flex; gap: 4px; padding: 8px 12px; background: #2d2d44; border-radius: 16px; width: fit-content; margin-bottom: 12px;",

            span {
                style: "width: 8px; height: 8px; border-radius: 50%; background: #888;",
            }
            span {
                style: "width: 8px; height: 8px; border-radius: 50%; background: #888;",
            }
            span {
                style: "width: 8px; height: 8px; border-radius: 50%; background: #888;",
            }
        }
    }
}

/// Media preview component for pending attachment
#[component]
pub fn MediaPreview(
    media: crate::media::SelectedMedia,
    on_remove: EventHandler<()>,
) -> Element {
    let src = format!("data:{};base64,{}", media.mimetype, media.data);

    rsx! {
        div {
            style: "padding: 8px 16px; background: #1a1a2e; border-top: 1px solid #2d2d44;",

            div {
                style: "display: flex; align-items: center; gap: 12px; padding: 8px; background: #2d2d44; border-radius: 8px;",

                // Thumbnail
                img {
                    style: "width: 60px; height: 60px; object-fit: cover; border-radius: 4px;",
                    src: "{src}",
                }

                // Filename
                div {
                    style: "flex: 1; min-width: 0;",
                    p {
                        style: "margin: 0; color: #fff; font-size: 0.875rem; white-space: nowrap; overflow: hidden; text-overflow: ellipsis;",
                        "{media.filename}"
                    }
                    p {
                        style: "margin: 4px 0 0 0; color: #888; font-size: 0.75rem;",
                        "Tap to remove"
                    }
                }

                // Remove button
                button {
                    style: "width: 32px; height: 32px; border-radius: 50%; border: none; background: #dc3545; color: white; cursor: pointer; display: flex; align-items: center; justify-content: center; font-size: 1rem; flex-shrink: 0;",
                    r#type: "button",
                    onclick: move |_| on_remove.call(()),
                    "×"
                }
            }
        }
    }
}
