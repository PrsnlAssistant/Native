//! Message types for chat functionality

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Sender type for messages
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageSender {
    User,
    Assistant,
    System,
}

/// Message delivery status
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageStatus {
    Sending,
    Sent,
    Delivered,
    Error(String),
}

/// Image data attached to a message
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImageData {
    pub data: String,      // Base64 encoded
    pub mimetype: String,
}

/// A chat message
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub sender: MessageSender,
    pub status: MessageStatus,
    pub image: Option<ImageData>,
}

impl Message {
    /// Create a new user message
    pub fn new_user(body: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            body,
            timestamp: Utc::now(),
            sender: MessageSender::User,
            status: MessageStatus::Sending,
            image: None,
        }
    }

    /// Create a new user message with an image attachment
    pub fn new_user_with_image(body: String, image: ImageData) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            body,
            timestamp: Utc::now(),
            sender: MessageSender::User,
            status: MessageStatus::Sending,
            image: Some(image),
        }
    }

    /// Create a new assistant message
    pub fn new_assistant(id: String, body: String, image: Option<ImageData>) -> Self {
        Self {
            id,
            body,
            timestamp: Utc::now(),
            sender: MessageSender::Assistant,
            status: MessageStatus::Delivered,
            image,
        }
    }

    /// Create a new system message
    pub fn new_system(body: String) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            body,
            timestamp: Utc::now(),
            sender: MessageSender::System,
            status: MessageStatus::Delivered,
            image: None,
        }
    }
}
