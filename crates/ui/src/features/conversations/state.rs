//! Conversations feature state

use std::collections::HashMap;
use dioxus::prelude::*;
use prsnl_core::Conversation;

/// View state for navigation
#[derive(Debug, Clone, PartialEq)]
pub enum ViewState {
    ConversationList,
    Chat(String),
}

/// Internal state for the conversations feature
#[derive(Debug, Clone)]
pub struct ConversationsStateInner {
    pub conversations: HashMap<String, Conversation>,
    pub view: ViewState,
    pub loading: bool,
}

/// State for the conversations feature (wraps a Signal)
#[derive(Clone, Copy)]
pub struct ConversationsState {
    inner: Signal<ConversationsStateInner>,
}

impl ConversationsState {
    /// Create new conversations state
    pub fn new() -> Self {
        Self {
            inner: Signal::new(ConversationsStateInner {
                conversations: HashMap::new(),
                view: ViewState::ConversationList,
                loading: true,
            }),
        }
    }

    // ============================================
    // Read accessors
    // ============================================

    /// Get current view state
    pub fn view(&self) -> ViewState {
        self.inner.read().view.clone()
    }

    /// Check if conversations are loading
    pub fn is_loading(&self) -> bool {
        self.inner.read().loading
    }

    /// Get all conversations sorted by most recent first
    pub fn sorted_conversations(&self) -> Vec<Conversation> {
        let inner = self.inner.read();
        let mut convs: Vec<_> = inner.conversations.values().cloned().collect();
        convs.sort_by(|a, b| b.last_message_time.cmp(&a.last_message_time));
        convs
    }

    /// Get a specific conversation
    pub fn get_conversation(&self, id: &str) -> Option<Conversation> {
        self.inner.read().conversations.get(id).cloned()
    }

    /// Get current conversation ID if viewing a chat
    pub fn current_conversation_id(&self) -> Option<String> {
        match &self.inner.read().view {
            ViewState::Chat(id) => Some(id.clone()),
            ViewState::ConversationList => None,
        }
    }

    // ============================================
    // Mutations (use mut self for Signal write access)
    // ============================================

    /// Set loading state
    pub fn set_loading(&mut self, loading: bool) {
        self.inner.write().loading = loading;
    }

    /// Navigate to conversation list
    pub fn go_to_list(&mut self) {
        self.inner.write().view = ViewState::ConversationList;
    }

    /// Open a specific conversation
    pub fn open_conversation(&mut self, id: &str) {
        self.inner.write().view = ViewState::Chat(id.to_string());
    }

    /// Add or update a conversation
    pub fn upsert_conversation(&mut self, conversation: Conversation) {
        self.inner.write().conversations.insert(conversation.id.clone(), conversation);
    }

    /// Set all conversations (from server load)
    pub fn set_conversations(&mut self, conversations: Vec<Conversation>) {
        let mut inner = self.inner.write();
        inner.loading = false;
        for conv in conversations {
            inner.conversations.insert(conv.id.clone(), conv);
        }
    }

    /// Create a new conversation and navigate to it
    pub fn create_conversation(&mut self, id: String, title: Option<String>) {
        let conv = Conversation::new(id.clone(), title);
        let mut inner = self.inner.write();
        inner.conversations.insert(id.clone(), conv);
        inner.view = ViewState::Chat(id);
    }

    /// Delete a conversation
    pub fn delete_conversation(&mut self, id: &str) {
        let mut inner = self.inner.write();
        inner.conversations.remove(id);

        // If viewing the deleted conversation, go back to list
        if matches!(&inner.view, ViewState::Chat(view_id) if view_id == id) {
            inner.view = ViewState::ConversationList;
        }
    }
}

impl Default for ConversationsState {
    fn default() -> Self {
        Self::new()
    }
}
