//! Conversation type and related structures

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

use super::message::{Message, MessageStatus};

/// A conversation containing messages
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub last_message_preview: Option<String>,
    pub message_count: u32,
    pub pending_messages: HashSet<String>,
}

impl Conversation {
    /// Create a new conversation
    pub fn new(id: String, title: Option<String>) -> Self {
        Self {
            id,
            title: title.unwrap_or_else(|| "New Chat".to_string()),
            messages: Vec::new(),
            last_message_time: None,
            last_message_preview: None,
            message_count: 0,
            pending_messages: HashSet::new(),
        }
    }

    /// Create a conversation from server data (for conversation list)
    pub fn from_server(
        id: String,
        last_message: Option<String>,
        last_message_time: Option<i64>,
        message_count: u32,
    ) -> Self {
        // Extract a short ID from the full conversation ID for display
        let short_id = id
            .split('-')
            .nth(1)
            .map(|s| s.chars().take(8).collect::<String>())
            .unwrap_or_else(|| id.chars().take(8).collect());

        Self {
            id,
            title: format!("Chat {}", short_id),
            messages: Vec::new(),
            last_message_time: last_message_time
                .and_then(DateTime::from_timestamp_millis),
            last_message_preview: last_message,
            message_count,
            pending_messages: HashSet::new(),
        }
    }

    /// Add a user message to the conversation
    pub fn add_user_message(&mut self, message: Message) {
        self.pending_messages.insert(message.id.clone());
        self.last_message_time = Some(message.timestamp);
        self.last_message_preview = Some(message.body.clone());
        self.message_count += 1;
        self.messages.push(message);
    }

    /// Add a response to a pending message
    pub fn add_response(&mut self, reply_to: &str, response: Message) {
        self.pending_messages.remove(reply_to);

        // Mark the original message as delivered
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == reply_to) {
            msg.status = MessageStatus::Delivered;
        }

        self.last_message_time = Some(response.timestamp);
        self.last_message_preview = Some(response.body.clone());
        self.message_count += 1;
        self.messages.push(response);
    }

    /// Mark a message as having an error
    pub fn mark_message_error(&mut self, id: &str, error: String) {
        self.pending_messages.remove(id);
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == id) {
            msg.status = MessageStatus::Error(error);
        }
    }

    /// Mark a message as sent (received by server)
    pub fn mark_message_sent(&mut self, id: &str) {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == id) {
            msg.status = MessageStatus::Sent;
        }
    }

    /// Set messages from history
    pub fn set_messages(&mut self, messages: Vec<Message>) {
        self.messages = messages;
        if let Some(last) = self.messages.last() {
            self.last_message_time = Some(last.timestamp);
            self.last_message_preview = Some(last.body.clone());
        }
    }
}
