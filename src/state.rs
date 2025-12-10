//! Application state management with multi-conversation support

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use std::collections::HashMap;

/// Connection status to the backend server
#[derive(Clone, Debug, PartialEq)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    Disconnected,
    Reconnecting,
}

/// A chat message
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub sender: MessageSender,
    pub status: MessageStatus,
    /// Optional image data (base64)
    pub image: Option<ImageData>,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MessageSender {
    User,
    Assistant,
    System,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub enum MessageStatus {
    Sending,
    Sent,
    Delivered,
    Error(String),
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ImageData {
    pub data: String,      // base64
    pub mimetype: String,
}

impl Message {
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

/// A conversation (chat thread)
#[derive(Clone, Debug, PartialEq)]
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub last_message_preview: Option<String>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub message_count: usize,
    /// Pending message IDs (waiting for response)
    pub pending_messages: Vec<String>,
}

impl Conversation {
    pub fn new(id: String, title: Option<String>) -> Self {
        let display_title = title.unwrap_or_else(|| "New Chat".to_string());
        Self {
            id,
            title: display_title,
            messages: vec![],
            last_message_preview: None,
            last_message_time: None,
            message_count: 0,
            pending_messages: vec![],
        }
    }

    pub fn from_server(
        id: String,
        last_message: Option<String>,
        last_message_time: Option<i64>,
        message_count: usize,
    ) -> Self {
        // Generate a title from the ID (remove prefix, truncate)
        let title = id
            .strip_prefix("native-")
            .unwrap_or(&id)
            .chars()
            .take(8)
            .collect::<String>();

        Self {
            id,
            title: format!("Chat {}", title),
            messages: vec![],
            last_message_preview: last_message,
            last_message_time: last_message_time.map(|t| {
                DateTime::from_timestamp_millis(t).unwrap_or_else(Utc::now)
            }),
            message_count,
            pending_messages: vec![],
        }
    }

    /// Add a message and track if it's pending
    pub fn add_user_message(&mut self, msg: Message) {
        self.pending_messages.push(msg.id.clone());
        self.last_message_preview = Some(msg.body.clone());
        self.last_message_time = Some(msg.timestamp);
        self.message_count += 1;
        self.messages.push(msg);
    }

    /// Mark a message as sent
    pub fn mark_message_sent(&mut self, id: &str) {
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == id) {
            msg.status = MessageStatus::Sent;
        }
    }

    /// Add response and remove from pending
    pub fn add_response(&mut self, reply_to: &str, response: Message) {
        self.pending_messages.retain(|id| id != reply_to);

        // Mark original as delivered
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == reply_to) {
            msg.status = MessageStatus::Delivered;
        }

        self.last_message_preview = Some(response.body.clone());
        self.last_message_time = Some(response.timestamp);
        self.message_count += 1;
        self.messages.push(response);
    }

    /// Mark a message as errored
    pub fn mark_message_error(&mut self, id: &str, error: String) {
        self.pending_messages.retain(|pid| pid != id);
        if let Some(msg) = self.messages.iter_mut().find(|m| m.id == id) {
            msg.status = MessageStatus::Error(error);
        }
    }
}

/// UI view state
#[derive(Clone, Debug, PartialEq)]
pub enum ViewState {
    /// Chat list view
    ConversationList,
    /// Inside a specific conversation
    Chat(String), // conversation_id
}

/// Global application state
#[derive(Clone, Debug)]
pub struct AppState {
    /// All conversations indexed by ID
    pub conversations: HashMap<String, Conversation>,
    /// Current view state
    pub view: ViewState,
    /// Connection status to the server
    pub connection_status: ConnectionStatus,
    /// Whether the assistant is typing in the current conversation
    pub is_typing: bool,
    /// Server WebSocket URL
    pub server_url: String,
    /// Loading state for conversations list
    pub loading_conversations: bool,
}

impl AppState {
    pub fn new() -> Self {
        Self {
            conversations: HashMap::new(),
            view: ViewState::ConversationList,
            connection_status: ConnectionStatus::Connecting,
            is_typing: false,
            server_url: "ws://10.8.0.8:8765/ws".to_string(),
            loading_conversations: true,
        }
    }

    /// Get the current conversation (if in chat view)
    pub fn current_conversation(&self) -> Option<&Conversation> {
        if let ViewState::Chat(ref id) = self.view {
            self.conversations.get(id)
        } else {
            None
        }
    }

    /// Get the current conversation mutably
    pub fn current_conversation_mut(&mut self) -> Option<&mut Conversation> {
        if let ViewState::Chat(ref id) = self.view {
            self.conversations.get_mut(id)
        } else {
            None
        }
    }

    /// Get current conversation ID
    pub fn current_conversation_id(&self) -> Option<&str> {
        if let ViewState::Chat(ref id) = self.view {
            Some(id)
        } else {
            None
        }
    }

    /// Switch to a conversation
    pub fn open_conversation(&mut self, id: &str) {
        self.view = ViewState::Chat(id.to_string());
    }

    /// Go back to conversation list
    pub fn go_to_list(&mut self) {
        self.view = ViewState::ConversationList;
    }

    /// Add or update a conversation
    pub fn upsert_conversation(&mut self, conv: Conversation) {
        self.conversations.insert(conv.id.clone(), conv);
    }

    /// Create a new conversation and switch to it
    pub fn create_conversation(&mut self, id: String, title: Option<String>) {
        let conv = Conversation::new(id.clone(), title);
        self.conversations.insert(id.clone(), conv);
        self.view = ViewState::Chat(id);
    }

    /// Delete a conversation
    pub fn delete_conversation(&mut self, id: &str) {
        self.conversations.remove(id);
        // If we were viewing this conversation, go back to list
        if let ViewState::Chat(ref current_id) = self.view {
            if current_id == id {
                self.view = ViewState::ConversationList;
            }
        }
    }

    /// Get sorted conversations (most recent first)
    pub fn sorted_conversations(&self) -> Vec<&Conversation> {
        let mut convs: Vec<_> = self.conversations.values().collect();
        convs.sort_by(|a, b| {
            b.last_message_time.cmp(&a.last_message_time)
        });
        convs
    }

    /// Find a conversation by ID and add a response
    pub fn add_response_to_conversation(
        &mut self,
        conversation_id: &str,
        reply_to: &str,
        response: Message,
    ) {
        if let Some(conv) = self.conversations.get_mut(conversation_id) {
            conv.add_response(reply_to, response);
        }
    }

    /// Find a conversation and mark message error
    pub fn mark_message_error_in_conversation(
        &mut self,
        conversation_id: &str,
        message_id: &str,
        error: String,
    ) {
        if let Some(conv) = self.conversations.get_mut(conversation_id) {
            conv.mark_message_error(message_id, error);
        }
    }

    /// Set conversation messages from history
    pub fn set_conversation_history(&mut self, conversation_id: &str, messages: Vec<Message>) {
        if let Some(conv) = self.conversations.get_mut(conversation_id) {
            conv.messages = messages;
        }
    }
}

impl Default for AppState {
    fn default() -> Self {
        Self::new()
    }
}
