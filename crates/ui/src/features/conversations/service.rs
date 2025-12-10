//! Conversations feature service

use std::sync::Arc;
use dioxus::prelude::spawn;
use futures::StreamExt;
use tracing::info;

use prsnl_core::{AppEvent, EventBus, Transport};
use super::state::ConversationsState;

/// Service for managing conversations
#[derive(Clone)]
pub struct ConversationsService {
    state: ConversationsState,
    event_bus: Arc<dyn EventBus>,
    transport: Arc<dyn Transport>,
}

impl ConversationsService {
    /// Create a new conversations service
    pub fn new(
        state: ConversationsState,
        event_bus: Arc<dyn EventBus>,
        transport: Arc<dyn Transport>,
    ) -> Self {
        Self { state, event_bus, transport }
    }

    /// Subscribe to relevant events from the event bus
    pub fn subscribe_to_events(&self) {
        let mut state = self.state;
        let mut rx = self.event_bus.subscribe();

        spawn(async move {
            while let Some(event) = rx.next().await {
                match event {
                    AppEvent::ConversationsLoaded(conversations) => {
                        state.set_conversations(conversations);
                    }
                    AppEvent::ConversationCreated { id, title } => {
                        state.create_conversation(id, title);
                    }
                    AppEvent::ConversationDeleted(id) => {
                        state.delete_conversation(&id);
                    }
                    AppEvent::NavigateToList => {
                        state.go_to_list();
                    }
                    AppEvent::NavigateToChat(id) => {
                        state.open_conversation(&id);
                    }
                    _ => {}
                }
            }
        });
    }

    /// Select a conversation to view
    pub fn select_conversation(&self, id: &str) {
        info!("Opening conversation: {}", id);
        let mut state = self.state;
        state.open_conversation(id);
        self.event_bus.publish(AppEvent::ConversationSelected(id.to_string()));

        // Request history for this conversation
        let transport = self.transport.clone();
        let conv_id = id.to_string();
        spawn(async move {
            if let Err(e) = transport.send_get_history(conv_id, Some(50)).await {
                info!("Failed to get history: {:?}", e);
            }
        });
    }

    /// Create a new conversation
    pub fn create_conversation(&self, title: Option<String>) {
        info!("Creating new conversation");
        let transport = self.transport.clone();
        spawn(async move {
            if let Err(e) = transport.send_create_conversation(title).await {
                info!("Failed to create conversation: {:?}", e);
            }
        });
    }

    /// Delete a conversation
    pub fn delete_conversation(&self, id: &str) {
        info!("Deleting conversation: {}", id);
        let transport = self.transport.clone();
        let conv_id = id.to_string();
        spawn(async move {
            if let Err(e) = transport.send_delete_conversation(conv_id).await {
                info!("Failed to delete conversation: {:?}", e);
            }
        });
    }

    /// Go back to conversation list
    pub fn go_back(&self) {
        let mut state = self.state;
        state.go_to_list();
        self.event_bus.publish(AppEvent::NavigateToList);
    }
}
