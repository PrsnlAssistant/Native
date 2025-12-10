//! Single conversation item component

use dioxus::prelude::*;
use prsnl_core::Conversation;

/// A single conversation in the list
#[component]
pub fn ConversationItem(
    conversation: Conversation,
    on_select: EventHandler<String>,
) -> Element {
    let conv_id = conversation.id.clone();

    // Format time ago
    let time_ago = conversation.last_message_time.map(|time| {
        let now = chrono::Utc::now();
        let diff = now.signed_duration_since(time);

        if diff.num_days() > 0 {
            format!("{}d ago", diff.num_days())
        } else if diff.num_hours() > 0 {
            format!("{}h ago", diff.num_hours())
        } else if diff.num_minutes() > 0 {
            format!("{}m ago", diff.num_minutes())
        } else {
            "Just now".to_string()
        }
    }).unwrap_or_else(|| "".to_string());

    // Truncate preview safely for UTF-8
    let preview = conversation.last_message_preview.unwrap_or_default();
    let preview_truncated = if preview.chars().count() > 50 {
        format!("{}...", preview.chars().take(50).collect::<String>())
    } else {
        preview
    };

    rsx! {
        button {
            onclick: move |_| on_select.call(conv_id.clone()),
            class: "w-full p-4 bg-transparent border-none border-b border-border text-left cursor-pointer flex flex-col gap-1 hover:bg-bg-hover transition-colors",

            // Title and time row
            div {
                class: "flex justify-between items-center",
                span {
                    class: "text-text-white font-medium",
                    "{conversation.title}"
                }
                span {
                    class: "text-text-muted text-xs",
                    "{time_ago}"
                }
            }

            // Preview and count row
            div {
                class: "flex justify-between items-center",
                span {
                    class: "text-text-secondary text-sm overflow-hidden text-ellipsis whitespace-nowrap flex-1",
                    "{preview_truncated}"
                }
                if conversation.message_count > 0 {
                    span {
                        class: "text-text-muted text-xs ml-2",
                        "{conversation.message_count} msgs"
                    }
                }
            }
        }
    }
}
