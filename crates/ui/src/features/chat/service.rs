//! Chat feature service

use dioxus::prelude::spawn;
use futures::StreamExt;
use tracing::info;

use prsnl_core::{
    AppEvent, SharedEventBus, SharedTransport, ImagePayload,
    Message, ImageData,
};
use crate::features::media::SelectedMedia;
use super::state::ChatState;

/// Service for managing chat functionality
#[derive(Clone)]
pub struct ChatService {
    state: ChatState,
    event_bus: SharedEventBus,
    transport: SharedTransport,
}

impl ChatService {
    /// Create a new chat service
    pub fn new(
        state: ChatState,
        event_bus: SharedEventBus,
        transport: SharedTransport,
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
                    AppEvent::ConversationSelected(id) => {
                        state.set_current_conversation(Some(id));
                    }
                    AppEvent::MessageReceived { conv_id, message } => {
                        // Find the reply_to from the message context
                        // For now, we'll use the last pending message
                        let reply_to = state.current_messages()
                            .iter()
                            .rev()
                            .find(|m| state.is_pending(&m.id))
                            .map(|m| m.id.clone())
                            .unwrap_or_default();

                        state.add_received_message(&conv_id, &reply_to, message);
                    }
                    AppEvent::MessageError { conv_id, msg_id, error } => {
                        state.mark_message_error(&conv_id, &msg_id, error);
                    }
                    AppEvent::TypingChanged { conv_id, is_typing } => {
                        state.set_typing(&conv_id, is_typing);
                    }
                    AppEvent::HistoryLoaded { conv_id, messages } => {
                        state.set_history(&conv_id, messages);
                    }
                    AppEvent::ConversationDeleted(id) => {
                        state.clear_conversation(&id);
                    }
                    AppEvent::NavigateToList => {
                        state.set_current_conversation(None);
                    }
                    _ => {}
                }
            }
        });
    }

    /// Send a message in the current conversation
    pub fn send_message(&self, text: String, media: Option<SelectedMedia>) {
        // Validate input
        if text.trim().is_empty() && media.is_none() {
            return;
        }

        let conv_id = match self.state.current_conv_id() {
            Some(id) => id,
            None => {
                info!("Cannot send message: no conversation selected");
                return;
            }
        };

        // Create message
        let msg = match media {
            Some(ref m) => Message::new_user_with_image(
                text.clone(),
                ImageData {
                    data: m.data.clone(),
                    mimetype: m.mimetype.clone(),
                },
            ),
            None => Message::new_user(text.clone()),
        };

        // Optimistic update - add message to state immediately
        let mut state = self.state;
        state.add_user_message(&conv_id, msg.clone());

        // Publish event
        self.event_bus.publish(AppEvent::MessageSent {
            conv_id: conv_id.clone(),
            message: msg.clone(),
        });

        // Send to server
        let transport = self.transport.clone();
        let image_payload = media.map(|m| ImagePayload {
            data: m.data,
            mimetype: m.mimetype,
        });
        let conv_id_owned = conv_id;
        let text_owned = text;

        spawn(async move {
            if let Err(e) = transport.send_chat(conv_id_owned, text_owned, image_payload).await {
                info!("Failed to send message: {:?}", e);
            }
        });
    }

    /// Request history for a conversation
    pub fn load_history(&self, conv_id: &str) {
        let transport = self.transport.clone();
        let id = conv_id.to_string();
        spawn(async move {
            if let Err(e) = transport.send_get_history(id, Some(50)).await {
                info!("Failed to load history: {:?}", e);
            }
        });
    }
}
