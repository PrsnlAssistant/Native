//! Message bubble component

use dioxus::prelude::*;
use prsnl_core::{Message, MessageSender, MessageStatus};

/// A single message bubble
#[component]
pub fn MessageBubble(message: Message) -> Element {
    let is_user = message.sender == MessageSender::User;
    let is_system = message.sender == MessageSender::System;

    // Use Tailwind component classes with conditional variants
    let container_class = if is_system {
        "flex justify-center mb-3"
    } else if is_user {
        "flex justify-end mb-3"
    } else {
        "flex justify-start mb-3"
    };

    let bubble_class = if is_system {
        "message-bubble max-w-[90%] bg-bg-tertiary"
    } else if is_user {
        "message-bubble message-bubble-user"
    } else {
        "message-bubble message-bubble-assistant"
    };

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
            class: "{container_class}",

            div {
                class: "{bubble_class}",

                // Image if present
                if let Some(ref image) = message.image {
                    {
                        let img_src = format!("data:{};base64,{}", image.mimetype, image.data);
                        rsx! {
                            div {
                                class: "mb-2",
                                img {
                                    src: "{img_src}",
                                    class: "max-w-full max-h-[200px] rounded-lg",
                                }
                            }
                        }
                    }
                }

                // Message body
                if !message.body.is_empty() {
                    p {
                        class: "m-0 whitespace-pre-wrap break-words",
                        "{message.body}"
                    }
                }

                // Footer with time and status
                div {
                    class: "flex justify-end items-center gap-1 mt-1",

                    span {
                        class: "text-[0.7rem] text-white/60",
                        "{time}"
                    }

                    if let Some(icon) = status_icon {
                        {
                            let status_class = match &message.status {
                                MessageStatus::Error(_) => "text-[0.7rem] text-error",
                                MessageStatus::Delivered => "text-[0.7rem] text-success",
                                _ => "text-[0.7rem] text-white/60",
                            };
                            rsx! {
                                span {
                                    class: "{status_class}",
                                    "{icon}"
                                }
                            }
                        }
                    }
                }

                // Error message if present
                if let MessageStatus::Error(ref err) = message.status {
                    div {
                        class: "text-xs text-error mt-1",
                        "{err}"
                    }
                }
            }
        }
    }
}
