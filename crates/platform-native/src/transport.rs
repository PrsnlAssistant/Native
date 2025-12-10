//! Native WebSocket transport using tokio-tungstenite
//!
//! This module provides a full WebSocket transport implementation for native platforms.
//! It handles connection management, message dispatch, ping/pong keep-alive, and reconnection.

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

use chrono::Utc;
use futures_util::{stream::SplitSink, SinkExt, StreamExt};
use tokio::sync::Mutex;
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use tracing::info;
use uuid::Uuid;

use prsnl_core::{
    AppEvent, ConnectionStatus, Conversation, EventBus, HistoryMessage, ImageData, ImagePayload,
    Message, MessageSender, MessageStatus, Transport, TransportResult, TransportResultVoid,
    WSClientMessage, WSServerMessage,
};

/// WebSocket connection type alias
pub type WsConnection = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

/// Ping interval for keep-alive
const PING_INTERVAL: Duration = Duration::from_secs(30);

/// Maximum reconnection attempts
const MAX_RECONNECT_ATTEMPTS: u32 = 5;

/// Delay between reconnection attempts (starts at this and increases exponentially)
const INITIAL_RECONNECT_DELAY: Duration = Duration::from_secs(1);

/// Native transport implementation using tokio-tungstenite
pub struct NativeTransport {
    /// WebSocket sender for outgoing messages
    sender: Arc<Mutex<Option<SplitSink<WsConnection, WsMessage>>>>,
    /// Connection state flag
    connected: Arc<AtomicBool>,
    /// Flag to signal shutdown
    shutdown: Arc<AtomicBool>,
}

impl NativeTransport {
    /// Create a new native transport
    pub fn new() -> Self {
        Self {
            sender: Arc::new(Mutex::new(None)),
            connected: Arc::new(AtomicBool::new(false)),
            shutdown: Arc::new(AtomicBool::new(false)),
        }
    }
}

impl Default for NativeTransport {
    fn default() -> Self {
        Self::new()
    }
}

impl Transport for NativeTransport {
    fn connect(&self, url: String, event_bus: Arc<dyn EventBus>) -> TransportResultVoid {
        let sender = self.sender.clone();
        let connected = self.connected.clone();
        let shutdown = self.shutdown.clone();

        // Reset shutdown flag
        shutdown.store(false, Ordering::SeqCst);

        Box::pin(async move {
            info!("Attempting WebSocket connection to {}", url);
            event_bus.publish(AppEvent::ConnectionChanged(ConnectionStatus::Connecting));

            let mut reconnect_attempts = 0;
            let mut reconnect_delay = INITIAL_RECONNECT_DELAY;

            loop {
                match connect_async(&url).await {
                    Ok((ws_stream, _)) => {
                        let (write, mut read) = ws_stream.split();

                        // Store sender for outgoing messages
                        *sender.lock().await = Some(write);
                        connected.store(true, Ordering::SeqCst);
                        reconnect_attempts = 0;
                        reconnect_delay = INITIAL_RECONNECT_DELAY;

                        event_bus.publish(AppEvent::ConnectionChanged(ConnectionStatus::Connected));
                        info!("WebSocket connection established");

                        // Subscribe to notifications
                        {
                            let msg = WSClientMessage::Subscribe {
                                id: Uuid::new_v4().to_string(),
                                timestamp: Utc::now().timestamp_millis(),
                                events: vec![
                                    "notifications".to_string(),
                                    "reminders".to_string(),
                                ],
                            };
                            if let Ok(json) = serde_json::to_string(&msg) {
                                let mut guard = sender.lock().await;
                                if let Some(s) = guard.as_mut() {
                                    let _ = s.send(WsMessage::Text(json.into())).await;
                                }
                            }
                        }

                        // Request conversations list
                        {
                            let msg = WSClientMessage::ListConversations {
                                id: Uuid::new_v4().to_string(),
                                timestamp: Utc::now().timestamp_millis(),
                            };
                            if let Ok(json) = serde_json::to_string(&msg) {
                                let mut guard = sender.lock().await;
                                if let Some(s) = guard.as_mut() {
                                    let _ = s.send(WsMessage::Text(json.into())).await;
                                }
                            }
                        }

                        // Spawn ping task for keep-alive
                        let ping_sender = sender.clone();
                        let ping_connected = connected.clone();
                        let ping_shutdown = shutdown.clone();

                        tokio::spawn(async move {
                            let mut interval = tokio::time::interval(PING_INTERVAL);
                            loop {
                                interval.tick().await;

                                if ping_shutdown.load(Ordering::SeqCst) {
                                    break;
                                }

                                if !ping_connected.load(Ordering::SeqCst) {
                                    break;
                                }

                                let msg = WSClientMessage::Ping {
                                    id: Uuid::new_v4().to_string(),
                                    timestamp: Utc::now().timestamp_millis(),
                                };

                                let mut guard = ping_sender.lock().await;
                                if let Some(s) = guard.as_mut() {
                                    if let Ok(json) = serde_json::to_string(&msg) {
                                        if s.send(WsMessage::Text(json.into())).await.is_err() {
                                            break;
                                        }
                                    }
                                }
                            }
                        });

                        // Process incoming messages
                        while let Some(msg_result) = read.next().await {
                            if shutdown.load(Ordering::SeqCst) {
                                info!("Shutdown requested, closing connection");
                                break;
                            }

                            match msg_result {
                                Ok(WsMessage::Text(text)) => {
                                    // Parse and dispatch the message
                                    match serde_json::from_str::<WSServerMessage>(&text) {
                                        Ok(msg) => {
                                            dispatch_server_message(msg, &event_bus);
                                        }
                                        Err(e) => {
                                            info!(
                                                "Failed to parse server message: {:?} - raw: {}",
                                                e, text
                                            );
                                        }
                                    }
                                }
                                Ok(WsMessage::Ping(data)) => {
                                    let mut guard = sender.lock().await;
                                    if let Some(s) = guard.as_mut() {
                                        let _ = s.send(WsMessage::Pong(data)).await;
                                    }
                                }
                                Ok(WsMessage::Close(_)) => {
                                    info!("WebSocket connection closed by server");
                                    connected.store(false, Ordering::SeqCst);
                                    event_bus.publish(AppEvent::ConnectionChanged(
                                        ConnectionStatus::Disconnected,
                                    ));
                                    break;
                                }
                                Err(e) => {
                                    info!("WebSocket error: {:?}", e);
                                    connected.store(false, Ordering::SeqCst);
                                    event_bus.publish(AppEvent::ConnectionChanged(
                                        ConnectionStatus::Disconnected,
                                    ));
                                    break;
                                }
                                _ => {}
                            }
                        }

                        // Clear sender on disconnect
                        *sender.lock().await = None;
                        connected.store(false, Ordering::SeqCst);

                        // If shutdown was requested, exit the reconnect loop
                        if shutdown.load(Ordering::SeqCst) {
                            return Ok(());
                        }
                    }
                    Err(e) => {
                        info!("Failed to connect: {:?}", e);
                        connected.store(false, Ordering::SeqCst);
                        event_bus.publish(AppEvent::ConnectionChanged(
                            ConnectionStatus::Disconnected,
                        ));
                    }
                }

                // Reconnection logic
                reconnect_attempts += 1;
                if reconnect_attempts > MAX_RECONNECT_ATTEMPTS {
                    info!(
                        "Max reconnection attempts ({}) reached, giving up",
                        MAX_RECONNECT_ATTEMPTS
                    );
                    return Err(format!(
                        "Failed to connect after {} attempts",
                        MAX_RECONNECT_ATTEMPTS
                    ));
                }

                if shutdown.load(Ordering::SeqCst) {
                    return Ok(());
                }

                info!(
                    "Reconnecting in {:?} (attempt {}/{})",
                    reconnect_delay, reconnect_attempts, MAX_RECONNECT_ATTEMPTS
                );
                event_bus.publish(AppEvent::ConnectionChanged(ConnectionStatus::Connecting));
                tokio::time::sleep(reconnect_delay).await;

                // Exponential backoff
                reconnect_delay = std::cmp::min(reconnect_delay * 2, Duration::from_secs(30));
            }
        })
    }

    fn disconnect(&self) -> TransportResultVoid {
        let sender = self.sender.clone();
        let connected = self.connected.clone();
        let shutdown = self.shutdown.clone();

        Box::pin(async move {
            info!("Disconnecting WebSocket");
            shutdown.store(true, Ordering::SeqCst);

            // Send close frame if connected
            let mut guard = sender.lock().await;
            if let Some(s) = guard.as_mut() {
                let _ = s.send(WsMessage::Close(None)).await;
            }
            *guard = None;

            connected.store(false, Ordering::SeqCst);
            Ok(())
        })
    }

    fn send_chat(
        &self,
        conv_id: String,
        text: String,
        image: Option<ImagePayload>,
    ) -> TransportResult<String> {
        let sender = self.sender.clone();

        Box::pin(async move {
            let msg_id = Uuid::new_v4().to_string();
            let msg = WSClientMessage::Chat {
                id: msg_id.clone(),
                timestamp: Utc::now().timestamp_millis(),
                conversation_id: conv_id,
                body: text,
                image,
                reply_to: None,
            };

            let json =
                serde_json::to_string(&msg).map_err(|e| format!("Serialization error: {}", e))?;

            let mut guard = sender.lock().await;
            let s = guard.as_mut().ok_or("WebSocket not connected")?;
            s.send(WsMessage::Text(json.into()))
                .await
                .map_err(|e| format!("Send error: {}", e))?;

            Ok(msg_id)
        })
    }

    fn send_list_conversations(&self) -> TransportResultVoid {
        let sender = self.sender.clone();

        Box::pin(async move {
            let msg = WSClientMessage::ListConversations {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().timestamp_millis(),
            };

            let json =
                serde_json::to_string(&msg).map_err(|e| format!("Serialization error: {}", e))?;

            let mut guard = sender.lock().await;
            let s = guard.as_mut().ok_or("WebSocket not connected")?;
            s.send(WsMessage::Text(json.into()))
                .await
                .map_err(|e| format!("Send error: {}", e))?;

            Ok(())
        })
    }

    fn send_get_history(&self, conv_id: String, limit: Option<u32>) -> TransportResultVoid {
        let sender = self.sender.clone();

        Box::pin(async move {
            let msg = WSClientMessage::GetHistory {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().timestamp_millis(),
                conversation_id: conv_id,
                limit,
            };

            let json =
                serde_json::to_string(&msg).map_err(|e| format!("Serialization error: {}", e))?;

            let mut guard = sender.lock().await;
            let s = guard.as_mut().ok_or("WebSocket not connected")?;
            s.send(WsMessage::Text(json.into()))
                .await
                .map_err(|e| format!("Send error: {}", e))?;

            Ok(())
        })
    }

    fn send_create_conversation(&self, title: Option<String>) -> TransportResultVoid {
        let sender = self.sender.clone();

        Box::pin(async move {
            let msg = WSClientMessage::CreateConversation {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().timestamp_millis(),
                title,
            };

            let json =
                serde_json::to_string(&msg).map_err(|e| format!("Serialization error: {}", e))?;

            let mut guard = sender.lock().await;
            let s = guard.as_mut().ok_or("WebSocket not connected")?;
            s.send(WsMessage::Text(json.into()))
                .await
                .map_err(|e| format!("Send error: {}", e))?;

            Ok(())
        })
    }

    fn send_delete_conversation(&self, conv_id: String) -> TransportResultVoid {
        let sender = self.sender.clone();

        Box::pin(async move {
            let msg = WSClientMessage::DeleteConversation {
                id: Uuid::new_v4().to_string(),
                timestamp: Utc::now().timestamp_millis(),
                conversation_id: conv_id,
            };

            let json =
                serde_json::to_string(&msg).map_err(|e| format!("Serialization error: {}", e))?;

            let mut guard = sender.lock().await;
            let s = guard.as_mut().ok_or("WebSocket not connected")?;
            s.send(WsMessage::Text(json.into()))
                .await
                .map_err(|e| format!("Send error: {}", e))?;

            Ok(())
        })
    }

    fn is_connected(&self) -> bool {
        self.connected.load(Ordering::SeqCst)
    }
}

/// Dispatch a server message to the event bus (standalone function for use in async context)
fn dispatch_server_message(msg: WSServerMessage, event_bus: &Arc<dyn EventBus>) {
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
        }

        WSServerMessage::Error {
            reply_to,
            conversation_id,
            message,
            ..
        } => {
            info!("Error received: {}", message);
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
                    Conversation::from_server(
                        c.id,
                        c.last_message,
                        c.last_message_time,
                        c.message_count,
                    )
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

            let parsed_messages: Vec<Message> = messages
                .into_iter()
                .filter_map(parse_history_message)
                .collect();

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

    Some(Message {
        id: Uuid::new_v4().to_string(),
        body,
        timestamp: m
            .timestamp
            .and_then(chrono::DateTime::from_timestamp_millis)
            .unwrap_or_else(chrono::Utc::now),
        sender,
        status: MessageStatus::Delivered,
        image: None,
    })
}
