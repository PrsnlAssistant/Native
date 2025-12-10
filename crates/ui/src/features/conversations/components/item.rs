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
            style: "width: 100%; padding: 16px; background: transparent; border: none; border-bottom: 1px solid #2d2d44; text-align: left; cursor: pointer; display: flex; flex-direction: column; gap: 4px;",

            // Title and time row
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                span {
                    style: "color: white; font-weight: 500;",
                    "{conversation.title}"
                }
                span {
                    style: "color: #888; font-size: 0.75rem;",
                    "{time_ago}"
                }
            }

            // Preview and count row
            div {
                style: "display: flex; justify-content: space-between; align-items: center;",
                span {
                    style: "color: #888; font-size: 0.875rem; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; flex: 1;",
                    "{preview_truncated}"
                }
                if conversation.message_count > 0 {
                    span {
                        style: "color: #666; font-size: 0.75rem; margin-left: 8px;",
                        "{conversation.message_count} msgs"
                    }
                }
            }
        }
    }
}
