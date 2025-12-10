//! WebSocket protocol message types
//!
//! This module defines the wire protocol for client-server communication.

use serde::{Deserialize, Serialize};

// ============================================
// Client -> Server message types
// ============================================

/// Messages sent from client to server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WSClientMessage {
    #[serde(rename = "chat")]
    Chat {
        id: String,
        timestamp: i64,
        #[serde(rename = "conversationId")]
        conversation_id: String,
        body: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<ImagePayload>,
        #[serde(skip_serializing_if = "Option::is_none")]
        #[serde(rename = "replyTo")]
        reply_to: Option<String>,
    },
    #[serde(rename = "ping")]
    Ping { id: String, timestamp: i64 },
    #[serde(rename = "subscribe")]
    Subscribe {
        id: String,
        timestamp: i64,
        events: Vec<String>,
    },
    #[serde(rename = "list_conversations")]
    ListConversations { id: String, timestamp: i64 },
    #[serde(rename = "get_history")]
    GetHistory {
        id: String,
        timestamp: i64,
        #[serde(rename = "conversationId")]
        conversation_id: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        limit: Option<u32>,
    },
    #[serde(rename = "create_conversation")]
    CreateConversation {
        id: String,
        timestamp: i64,
        #[serde(skip_serializing_if = "Option::is_none")]
        title: Option<String>,
    },
    #[serde(rename = "delete_conversation")]
    DeleteConversation {
        id: String,
        timestamp: i64,
        #[serde(rename = "conversationId")]
        conversation_id: String,
    },
}

/// Image payload for messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImagePayload {
    pub data: String,
    pub mimetype: String,
}

// ============================================
// Server -> Client message types
// ============================================

/// Messages received from server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WSServerMessage {
    #[serde(rename = "response")]
    Response {
        id: String,
        timestamp: i64,
        #[serde(rename = "replyTo")]
        reply_to: String,
        #[serde(rename = "conversationId")]
        conversation_id: Option<String>,
        body: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        image: Option<ImagePayload>,
    },
    #[serde(rename = "pong")]
    Pong { id: String, timestamp: i64 },
    #[serde(rename = "notification")]
    Notification {
        id: String,
        timestamp: i64,
        title: String,
        body: String,
        category: String,
    },
    #[serde(rename = "error")]
    Error {
        id: String,
        timestamp: i64,
        #[serde(rename = "replyTo")]
        reply_to: Option<String>,
        #[serde(rename = "conversationId")]
        conversation_id: Option<String>,
        code: String,
        message: String,
    },
    #[serde(rename = "typing")]
    Typing {
        id: String,
        timestamp: i64,
        #[serde(rename = "replyTo")]
        reply_to: String,
        #[serde(rename = "conversationId")]
        conversation_id: Option<String>,
        #[serde(rename = "isTyping")]
        is_typing: bool,
    },
    #[serde(rename = "conversations_list")]
    ConversationsList {
        id: String,
        timestamp: i64,
        conversations: Vec<ConversationInfo>,
    },
    #[serde(rename = "history")]
    History {
        id: String,
        timestamp: i64,
        #[serde(rename = "conversationId")]
        conversation_id: String,
        messages: Vec<HistoryMessage>,
    },
    #[serde(rename = "conversation_created")]
    ConversationCreated {
        id: String,
        timestamp: i64,
        #[serde(rename = "conversationId")]
        conversation_id: String,
        title: Option<String>,
    },
    #[serde(rename = "conversation_deleted")]
    ConversationDeleted {
        id: String,
        timestamp: i64,
        #[serde(rename = "conversationId")]
        conversation_id: String,
    },
}

/// Conversation info from list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationInfo {
    pub id: String,
    #[serde(rename = "lastMessage")]
    pub last_message: Option<String>,
    #[serde(rename = "lastMessageTime")]
    pub last_message_time: Option<i64>,
    #[serde(rename = "messageCount")]
    pub message_count: u32,
}

/// Message from history response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<i64>,
}
