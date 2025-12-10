//! Chat feature state

use std::collections::{HashMap, HashSet};
use dioxus::prelude::*;
use prsnl_core::{Message, MessageStatus};

/// Internal state for the chat feature
#[derive(Debug, Clone)]
pub struct ChatStateInner {
    /// Messages organized by conversation ID
    pub messages: HashMap<String, Vec<Message>>,
    /// Currently selected conversation
    pub current_conv_id: Option<String>,
    /// Whether assistant is typing
    pub is_typing: bool,
    /// Messages that are pending server acknowledgment
    pub pending_messages: HashSet<String>,
}

/// State for the chat feature (wraps a Signal)
#[derive(Clone, Copy)]
pub struct ChatState {
    inner: Signal<ChatStateInner>,
}

impl ChatState {
    /// Create new chat state
    pub fn new() -> Self {
        Self {
            inner: Signal::new(ChatStateInner {
                messages: HashMap::new(),
                current_conv_id: None,
                is_typing: false,
                pending_messages: HashSet::new(),
            }),
        }
    }

    // ============================================
    // Read accessors
    // ============================================

    /// Get messages for the current conversation
    pub fn current_messages(&self) -> Vec<Message> {
        let inner = self.inner.read();
        inner.current_conv_id
            .as_ref()
            .and_then(|id| inner.messages.get(id))
            .cloned()
            .unwrap_or_default()
    }

    /// Get messages for a specific conversation
    pub fn messages_for(&self, conv_id: &str) -> Vec<Message> {
        self.inner.read().messages.get(conv_id).cloned().unwrap_or_default()
    }

    /// Check if assistant is typing
    pub fn is_typing(&self) -> bool {
        self.inner.read().is_typing
    }

    /// Get current conversation ID
    pub fn current_conv_id(&self) -> Option<String> {
        self.inner.read().current_conv_id.clone()
    }

    /// Check if a message is pending
    pub fn is_pending(&self, msg_id: &str) -> bool {
        self.inner.read().pending_messages.contains(msg_id)
    }

    // ============================================
    // Mutations (use mut self for Signal write access)
    // ============================================

    /// Set the current conversation
    pub fn set_current_conversation(&mut self, conv_id: Option<String>) {
        let mut inner = self.inner.write();
        inner.current_conv_id = conv_id;
        inner.is_typing = false; // Reset typing when switching conversations
    }

    /// Set typing indicator
    pub fn set_typing(&mut self, conv_id: &str, is_typing: bool) {
        let mut inner = self.inner.write();
        // Only update if it's for the current conversation
        if inner.current_conv_id.as_ref() == Some(&conv_id.to_string()) {
            inner.is_typing = is_typing;
        }
    }

    /// Add a user message (optimistic update)
    pub fn add_user_message(&mut self, conv_id: &str, message: Message) {
        let mut inner = self.inner.write();
        inner.pending_messages.insert(message.id.clone());
        inner.messages
            .entry(conv_id.to_string())
            .or_default()
            .push(message);
    }

    /// Add a received message (from assistant)
    pub fn add_received_message(&mut self, conv_id: &str, reply_to: &str, message: Message) {
        let mut inner = self.inner.write();

        // Remove from pending
        inner.pending_messages.remove(reply_to);

        // Mark original message as delivered
        if let Some(messages) = inner.messages.get_mut(conv_id) {
            if let Some(msg) = messages.iter_mut().find(|m| m.id == reply_to) {
                msg.status = MessageStatus::Delivered;
            }
        }

        // Clear typing indicator
        if inner.current_conv_id.as_ref() == Some(&conv_id.to_string()) {
            inner.is_typing = false;
        }

        // Add response message
        inner.messages
            .entry(conv_id.to_string())
            .or_default()
            .push(message);
    }

    /// Mark a message as having an error
    pub fn mark_message_error(&mut self, conv_id: &str, msg_id: &str, error: String) {
        let mut inner = self.inner.write();
        inner.pending_messages.remove(msg_id);

        if let Some(messages) = inner.messages.get_mut(conv_id) {
            if let Some(msg) = messages.iter_mut().find(|m| m.id == msg_id) {
                msg.status = MessageStatus::Error(error);
            }
        }
    }

    /// Set messages from history
    pub fn set_history(&mut self, conv_id: &str, messages: Vec<Message>) {
        self.inner.write().messages.insert(conv_id.to_string(), messages);
    }

    /// Clear messages for a conversation (when deleted)
    pub fn clear_conversation(&mut self, conv_id: &str) {
        let mut inner = self.inner.write();
        inner.messages.remove(conv_id);
        if inner.current_conv_id.as_ref() == Some(&conv_id.to_string()) {
            inner.current_conv_id = None;
        }
    }
}

impl Default for ChatState {
    fn default() -> Self {
        Self::new()
    }
}
