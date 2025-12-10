//! Web WebSocket transport using web-sys::WebSocket
//!
//! This module provides a WebSocket transport implementation for WASM targets
//! using the browser's native WebSocket API via web-sys.

use prsnl_core::{
    AppEvent, ConnectionStatus, Conversation, EventBus, HistoryMessage, ImageData, ImagePayload,
    Message, MessageSender, MessageStatus, Transport, TransportResult, TransportResultVoid,
    WSClientMessage, WSServerMessage,
};
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;
use tracing::{info, warn};
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{CloseEvent, ErrorEvent, MessageEvent, WebSocket};

/// Reconnection configuration
const RECONNECT_DELAY_MS: u32 = 3000;
const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Internal state shared between callbacks
struct WebTransportInner {
    ws: Option<WebSocket>,
    event_bus: Option<Arc<dyn EventBus>>,
    url: Option<String>,
    reconnect_attempts: u32,
    // Store closures to prevent them from being dropped
    _onmessage: Option<Closure<dyn FnMut(MessageEvent)>>,
    _onerror: Option<Closure<dyn FnMut(ErrorEvent)>>,
    _onclose: Option<Closure<dyn FnMut(CloseEvent)>>,
    _onopen: Option<Closure<dyn FnMut()>>,
}

impl WebTransportInner {
    fn new() -> Self {
        Self {
            ws: None,
            event_bus: None,
            url: None,
            reconnect_attempts: 0,
            _onmessage: None,
            _onerror: None,
            _onclose: None,
            _onopen: None,
        }
    }

    fn is_connected(&self) -> bool {
        self.ws
            .as_ref()
            .map(|ws| ws.ready_state() == WebSocket::OPEN)
            .unwrap_or(false)
    }
}

/// Web transport implementation using web-sys::WebSocket
///
/// This transport is designed for single-threaded WASM environments.
/// It uses RefCell for interior mutability since WASM is single-threaded.
pub struct WebTransport {
    inner: Rc<RefCell<WebTransportInner>>,
}

impl WebTransport {
    /// Create a new web transport
    pub fn new() -> Self {
        Self {
            inner: Rc::new(RefCell::new(WebTransportInner::new())),
        }
    }

    /// Connect to the WebSocket server
    fn connect_internal(inner: Rc<RefCell<WebTransportInner>>) -> Result<(), String> {
        let (url, event_bus) = {
            let state = inner.borrow();
            let url = state.url.clone().ok_or("URL not set")?;
            let event_bus = state
                .event_bus
                .clone()
                .ok_or("Event bus not set")?;
            (url, event_bus)
        };

        info!("Attempting WebSocket connection to {}", url);
        event_bus.publish(AppEvent::ConnectionChanged(ConnectionStatus::Connecting));

        // Create the WebSocket
        let ws = WebSocket::new(&url).map_err(|e| format!("Failed to create WebSocket: {:?}", e))?;

        // Set binary type
        ws.set_binary_type(web_sys::BinaryType::Arraybuffer);

        // Clone references for callbacks
        let inner_onopen = inner.clone();
        let inner_onclose = inner.clone();

        // Set up onopen callback
        let event_bus_open = event_bus.clone();
        let onopen = Closure::wrap(Box::new(move || {
            info!("WebSocket connection established");

            // Reset reconnect attempts on successful connection
            {
                let mut state = inner_onopen.borrow_mut();
                state.reconnect_attempts = 0;
            }

            event_bus_open.publish(AppEvent::ConnectionChanged(ConnectionStatus::Connected));

            // Subscribe to notifications and request conversations
            let inner = inner_onopen.clone();
            wasm_bindgen_futures::spawn_local(async move {
                // Send subscribe message
                if let Err(e) = send_subscribe_internal(&inner) {
                    warn!("Failed to subscribe: {}", e);
                }

                // Request conversations list
                if let Err(e) = send_list_conversations_internal(&inner) {
                    warn!("Failed to request conversations: {}", e);
                }
            });
        }) as Box<dyn FnMut()>);

        // Set up onmessage callback
        let event_bus_msg = event_bus.clone();
        let onmessage = Closure::wrap(Box::new(move |e: MessageEvent| {
            if let Ok(text) = e.data().dyn_into::<js_sys::JsString>() {
                let text: String = text.into();
                dispatch_message(&text, &event_bus_msg);
            }
        }) as Box<dyn FnMut(MessageEvent)>);

        // Set up onerror callback
        let event_bus_err = event_bus.clone();
        let onerror = Closure::wrap(Box::new(move |e: ErrorEvent| {
            warn!("WebSocket error: {:?}", e.message());
            event_bus_err.publish(AppEvent::ConnectionChanged(ConnectionStatus::Disconnected));
        }) as Box<dyn FnMut(ErrorEvent)>);

        // Set up onclose callback
        let event_bus_close = event_bus.clone();
        let onclose = Closure::wrap(Box::new(move |e: CloseEvent| {
            info!(
                "WebSocket closed: code={}, reason={}",
                e.code(),
                e.reason()
            );
            event_bus_close.publish(AppEvent::ConnectionChanged(ConnectionStatus::Disconnected));

            // Attempt reconnection
            let inner = inner_onclose.clone();
            schedule_reconnect(inner);
        }) as Box<dyn FnMut(CloseEvent)>);

        // Attach callbacks to WebSocket
        ws.set_onopen(Some(onopen.as_ref().unchecked_ref()));
        ws.set_onmessage(Some(onmessage.as_ref().unchecked_ref()));
        ws.set_onerror(Some(onerror.as_ref().unchecked_ref()));
        ws.set_onclose(Some(onclose.as_ref().unchecked_ref()));

        // Store WebSocket and closures in state
        {
            let mut state = inner.borrow_mut();
            state.ws = Some(ws);
            state._onopen = Some(onopen);
            state._onmessage = Some(onmessage);
            state._onerror = Some(onerror);
            state._onclose = Some(onclose);
        }

        Ok(())
    }

    /// Disconnect from the WebSocket server
    fn disconnect_internal(inner: &Rc<RefCell<WebTransportInner>>) {
        let mut state = inner.borrow_mut();

        if let Some(ws) = state.ws.take() {
            // Clear callbacks to prevent reconnect attempts
            ws.set_onopen(None);
            ws.set_onmessage(None);
            ws.set_onerror(None);
            ws.set_onclose(None);

            // Close the WebSocket
            let _ = ws.close();
        }

        // Clear stored closures
        state._onopen = None;
        state._onmessage = None;
        state._onerror = None;
        state._onclose = None;

        // Publish disconnected event
        if let Some(event_bus) = &state.event_bus {
            event_bus.publish(AppEvent::ConnectionChanged(ConnectionStatus::Disconnected));
        }
    }

    /// Send a message over the WebSocket
    fn send_internal(inner: &Rc<RefCell<WebTransportInner>>, msg: &WSClientMessage) -> Result<(), String> {
        let state = inner.borrow();
        let ws = state.ws.as_ref().ok_or("WebSocket not connected")?;

        if ws.ready_state() != WebSocket::OPEN {
            return Err("WebSocket not open".to_string());
        }

        let json =
            serde_json::to_string(msg).map_err(|e| format!("Serialization error: {}", e))?;

        ws.send_with_str(&json)
            .map_err(|e| format!("Send error: {:?}", e))?;

        Ok(())
    }
}

impl Default for WebTransport {
    fn default() -> Self {
        Self::new()
    }
}

// SAFETY: WebTransport will only be used from the main browser thread
// WASM is single-threaded, so these markers are safe
unsafe impl Send for WebTransport {}
unsafe impl Sync for WebTransport {}

impl Transport for WebTransport {
    fn connect(&self, url: String, event_bus: Arc<dyn EventBus>) -> TransportResultVoid {
        let inner = self.inner.clone();

        // Store URL and event bus for reconnection
        {
            let mut state = inner.borrow_mut();
            state.url = Some(url);
            state.event_bus = Some(event_bus);
        }

        Box::pin(async move { WebTransport::connect_internal(inner) })
    }

    fn disconnect(&self) -> TransportResultVoid {
        let inner = self.inner.clone();
        Box::pin(async move {
            // Set reconnect attempts to max to prevent reconnection
            {
                let mut state = inner.borrow_mut();
                state.reconnect_attempts = MAX_RECONNECT_ATTEMPTS;
            }
            WebTransport::disconnect_internal(&inner);
            Ok(())
        })
    }

    fn send_chat(
        &self,
        conv_id: String,
        text: String,
        image: Option<ImagePayload>,
    ) -> TransportResult<String> {
        let inner = self.inner.clone();
        Box::pin(async move {
            let msg_id = generate_uuid();
            let msg = WSClientMessage::Chat {
                id: msg_id.clone(),
                timestamp: current_timestamp_millis(),
                conversation_id: conv_id,
                body: text,
                image,
                reply_to: None,
            };

            WebTransport::send_internal(&inner, &msg)?;
            Ok(msg_id)
        })
    }

    fn send_list_conversations(&self) -> TransportResultVoid {
        let inner = self.inner.clone();
        Box::pin(async move { send_list_conversations_internal(&inner) })
    }

    fn send_get_history(&self, conv_id: String, limit: Option<u32>) -> TransportResultVoid {
        let inner = self.inner.clone();
        Box::pin(async move {
            let msg = WSClientMessage::GetHistory {
                id: generate_uuid(),
                timestamp: current_timestamp_millis(),
                conversation_id: conv_id,
                limit,
            };

            WebTransport::send_internal(&inner, &msg)
        })
    }

    fn send_create_conversation(&self, title: Option<String>) -> TransportResultVoid {
        let inner = self.inner.clone();
        Box::pin(async move {
            let msg = WSClientMessage::CreateConversation {
                id: generate_uuid(),
                timestamp: current_timestamp_millis(),
                title,
            };

            WebTransport::send_internal(&inner, &msg)
        })
    }

    fn send_delete_conversation(&self, conv_id: String) -> TransportResultVoid {
        let inner = self.inner.clone();
        Box::pin(async move {
            let msg = WSClientMessage::DeleteConversation {
                id: generate_uuid(),
                timestamp: current_timestamp_millis(),
                conversation_id: conv_id,
            };

            WebTransport::send_internal(&inner, &msg)
        })
    }

    fn is_connected(&self) -> bool {
        self.inner.borrow().is_connected()
    }
}

// ============================================
// Helper functions
// ============================================

/// Generate a UUID v4 string
fn generate_uuid() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Get current timestamp in milliseconds
fn current_timestamp_millis() -> i64 {
    js_sys::Date::now() as i64
}

/// Send subscribe message
fn send_subscribe_internal(inner: &Rc<RefCell<WebTransportInner>>) -> Result<(), String> {
    let msg = WSClientMessage::Subscribe {
        id: generate_uuid(),
        timestamp: current_timestamp_millis(),
        events: vec!["notifications".to_string(), "reminders".to_string()],
    };

    WebTransport::send_internal(inner, &msg)
}

/// Send list conversations request
fn send_list_conversations_internal(inner: &Rc<RefCell<WebTransportInner>>) -> Result<(), String> {
    let msg = WSClientMessage::ListConversations {
        id: generate_uuid(),
        timestamp: current_timestamp_millis(),
    };

    WebTransport::send_internal(inner, &msg)
}

/// Schedule a reconnection attempt
fn schedule_reconnect(inner: Rc<RefCell<WebTransportInner>>) {
    let should_reconnect = {
        let mut state = inner.borrow_mut();
        if state.reconnect_attempts >= MAX_RECONNECT_ATTEMPTS {
            warn!(
                "Max reconnect attempts ({}) reached, giving up",
                MAX_RECONNECT_ATTEMPTS
            );
            false
        } else {
            state.reconnect_attempts += 1;
            true
        }
    };

    if !should_reconnect {
        return;
    }

    let attempts = inner.borrow().reconnect_attempts;
    info!(
        "Scheduling reconnect attempt {} in {}ms",
        attempts, RECONNECT_DELAY_MS
    );

    // Publish reconnecting status
    {
        let state = inner.borrow();
        if let Some(event_bus) = &state.event_bus {
            event_bus.publish(AppEvent::ConnectionChanged(ConnectionStatus::Reconnecting));
        }
    }

    // Schedule reconnect using gloo-timers
    let inner_clone = inner.clone();
    gloo_timers::callback::Timeout::new(RECONNECT_DELAY_MS, move || {
        info!("Attempting reconnection...");
        if let Err(e) = WebTransport::connect_internal(inner_clone) {
            warn!("Reconnection failed: {}", e);
        }
    })
    .forget();
}

/// Dispatch a received message to the event bus
fn dispatch_message(text: &str, event_bus: &Arc<dyn EventBus>) {
    match serde_json::from_str::<WSServerMessage>(text) {
        Ok(msg) => handle_server_message(msg, event_bus),
        Err(e) => {
            warn!("Failed to parse server message: {:?} - raw: {}", e, text);
        }
    }
}

/// Handle a parsed server message and publish appropriate events
fn handle_server_message(msg: WSServerMessage, event_bus: &Arc<dyn EventBus>) {
    match msg {
        WSServerMessage::Response {
            id,
            reply_to,
            conversation_id,
            body,
            image,
            ..
        } => {
            info!(
                "Received response for message {} in {:?}",
                reply_to, conversation_id
            );

            let image_data = image.map(|img| ImageData {
                data: img.data,
                mimetype: img.mimetype,
            });

            let message = Message::new_assistant(id, body, image_data);

            if let Some(conv_id) = conversation_id {
                event_bus.publish(AppEvent::MessageReceived { conv_id, message });
            }
        }

        WSServerMessage::Typing {
            conversation_id,
            is_typing,
            ..
        } => {
            if let Some(conv_id) = conversation_id {
                event_bus.publish(AppEvent::TypingChanged { conv_id, is_typing });
            }
        }

        WSServerMessage::Notification {
            title,
            body,
            category,
            ..
        } => {
            info!("Notification [{}]: {} - {}", category, title, body);
            // TODO: Publish notification event when notification feature is added
        }

        WSServerMessage::Error {
            reply_to,
            conversation_id,
            message,
            ..
        } => {
            warn!("Error received: {}", message);
            if let (Some(msg_id), Some(conv_id)) = (reply_to, conversation_id) {
                event_bus.publish(AppEvent::MessageError {
                    conv_id,
                    msg_id,
                    error: message,
                });
            }
        }

        WSServerMessage::ConversationsList { conversations, .. } => {
            info!("Received {} conversations", conversations.len());

            let convs: Vec<Conversation> = conversations
                .into_iter()
                .map(|c| {
                    Conversation::from_server(c.id, c.last_message, c.last_message_time, c.message_count)
                })
                .collect();

            event_bus.publish(AppEvent::ConversationsLoaded(convs));
        }

        WSServerMessage::History {
            conversation_id,
            messages,
            ..
        } => {
            info!(
                "Received {} history messages for {}",
                messages.len(),
                conversation_id
            );

            let parsed_messages: Vec<Message> =
                messages.into_iter().filter_map(parse_history_message).collect();

            event_bus.publish(AppEvent::HistoryLoaded {
                conv_id: conversation_id,
                messages: parsed_messages,
            });
        }

        WSServerMessage::ConversationCreated {
            conversation_id,
            title,
            ..
        } => {
            info!("Conversation created: {} ({:?})", conversation_id, title);
            event_bus.publish(AppEvent::ConversationCreated {
                id: conversation_id,
                title,
            });
        }

        WSServerMessage::ConversationDeleted {
            conversation_id, ..
        } => {
            info!("Conversation deleted: {}", conversation_id);
            event_bus.publish(AppEvent::ConversationDeleted(conversation_id));
        }

        WSServerMessage::Pong { .. } => {
            // Heartbeat response, nothing to do
        }
    }
}

/// Parse a history message into a Message struct
fn parse_history_message(m: HistoryMessage) -> Option<Message> {
    let sender = match m.role.as_str() {
        "user" => MessageSender::User,
        "assistant" => MessageSender::Assistant,
        "system" => MessageSender::System,
        _ => return None,
    };

    // Strip the metadata prefix from user messages if present
    // Format: "Current Date: ...\nCurrent Time: ...\nFrom: ...\nBody: ..."
    let body = if sender == MessageSender::User && m.content.starts_with("Current Date:") {
        m.content
            .lines()
            .find(|line| line.starts_with("Body: "))
            .map(|line| line.strip_prefix("Body: ").unwrap_or(line).to_string())
            .unwrap_or(m.content)
    } else {
        m.content
    };

    // Use js_sys::Date for timestamp in WASM
    let timestamp = m
        .timestamp
        .and_then(chrono::DateTime::from_timestamp_millis)
        .unwrap_or_else(chrono::Utc::now);

    Some(Message {
        id: generate_uuid(),
        body,
        timestamp,
        sender,
        status: MessageStatus::Delivered,
        image: None,
    })
}
