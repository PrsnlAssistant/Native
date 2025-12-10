//! Chat screen container component

use dioxus::prelude::*;
use prsnl_core::ConnectionStatus;
use crate::features::media::{SelectedMedia, MediaPreview, pick_image};
use super::{ChatHeader, MessageList, MessageInput, TypingIndicator};
use crate::features::chat::hooks::{use_messages_for, use_typing_indicator, use_send_message};

/// Chat screen container
#[component]
pub fn ChatScreen(
    conv_id: String,
    title: String,
    status: ConnectionStatus,
    on_back: EventHandler<()>,
    on_status_tap: EventHandler<()>,
) -> Element {
    // Local state for input and media
    let mut input_text = use_signal(|| String::new());
    let mut pending_media = use_signal(|| Option::<SelectedMedia>::None);

    // Get messages and typing state from hooks
    let messages = use_messages_for(&conv_id);
    let is_typing = use_typing_indicator();
    let send_message = use_send_message();

    // Handlers
    let on_send = {
        let send_message = send_message.clone();
        move |_| {
            let text = input_text.read().clone();
            let media = pending_media.read().clone();

            if !text.trim().is_empty() || media.is_some() {
                send_message(text, media);
                input_text.set(String::new());
                pending_media.set(None);
            }
        }
    };

    let on_media_select = move |_| {
        spawn(async move {
            if let Some(selected) = pick_image().await {
                pending_media.set(Some(selected));
            }
        });
    };

    let on_media_remove = move |_| {
        pending_media.set(None);
    };

    rsx! {
        div {
            class: "flex flex-col h-screen h-dvh min-h-full font-sans bg-bg-primary",

            // Header
            ChatHeader {
                title,
                status,
                on_back,
                on_status_tap,
            }

            // Messages area
            div {
                class: "flex-1 overflow-y-auto p-4 bg-bg-primary min-h-0",
                id: "chat-container",

                if messages.is_empty() {
                    div {
                        class: "flex flex-col items-center justify-center h-full text-text-muted",
                        p { "Start a conversation" }
                        p {
                            class: "text-sm",
                            "Type a message below"
                        }
                    }
                } else {
                    MessageList { messages }
                }

                if is_typing {
                    TypingIndicator {}
                }
            }

            // Media preview (if pending)
            if let Some(media) = pending_media.read().clone() {
                MediaPreview {
                    media,
                    on_remove: on_media_remove,
                }
            }

            // Input area
            MessageInput {
                value: input_text.read().clone(),
                on_change: move |new_value: String| input_text.set(new_value),
                on_send,
                on_media_select,
            }
        }
    }
}
