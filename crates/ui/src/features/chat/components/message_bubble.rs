//! Message bubble component

use dioxus::prelude::*;
use prsnl_core::{Message, MessageSender, MessageStatus};

/// A single message bubble
#[component]
pub fn MessageBubble(message: Message) -> Element {
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

    let max_width = if is_system { "90%" } else { "80%" };

    // Format timestamp
    let time = message.timestamp.format("%H:%M").to_string();

    // Status indicator for user messages
    let status_icon = if is_user {
        match &message.status {
            MessageStatus::Sending => Some("..."),
            MessageStatus::Sent => Some("v"),
            MessageStatus::Delivered => Some("vv"),
            MessageStatus::Error(_) => Some("!"),
        }
    } else {
        None
    };

    rsx! {
        div {
            class: "message-bubble",
            style: "display: flex; justify-content: {align}; margin-bottom: 12px;",

            div {
                style: "max-width: {max_width}; background: {bg_color}; padding: 12px 16px; border-radius: 16px; color: white;",

                // Image if present
                if let Some(ref image) = message.image {
                    {
                        let img_src = format!("data:{};base64,{}", image.mimetype, image.data);
                        rsx! {
                            div {
                                style: "margin-bottom: 8px;",
                                img {
                                    src: "{img_src}",
                                    style: "max-width: 100%; max-height: 200px; border-radius: 8px;",
                                }
                            }
                        }
                    }
                }

                // Message body
                if !message.body.is_empty() {
                    p {
                        style: "margin: 0; white-space: pre-wrap; word-break: break-word;",
                        "{message.body}"
                    }
                }

                // Footer with time and status
                div {
                    style: "display: flex; justify-content: flex-end; align-items: center; gap: 4px; margin-top: 4px;",

                    span {
                        style: "font-size: 0.7rem; color: rgba(255,255,255,0.6);",
                        "{time}"
                    }

                    if let Some(icon) = status_icon {
                        {
                            let status_color = match &message.status {
                                MessageStatus::Error(_) => "#f44336",
                                MessageStatus::Delivered => "#4caf50",
                                _ => "rgba(255,255,255,0.6)",
                            };
                            rsx! {
                                span {
                                    style: "font-size: 0.7rem; color: {status_color};",
                                    "{icon}"
                                }
                            }
                        }
                    }
                }

                // Error message if present
                if let MessageStatus::Error(ref err) = message.status {
                    div {
                        style: "font-size: 0.75rem; color: #f44336; margin-top: 4px;",
                        "{err}"
                    }
                }
            }
        }
    }
}
