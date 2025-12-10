//! WebSocket client for communicating with the PrsnlAssistant backend

use dioxus::prelude::*;
use dioxus_logger::tracing::info;
use futures_util::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use tokio_tungstenite::{connect_async, tungstenite::Message as WsMessage};
use uuid::Uuid;
use chrono::Utc;

use crate::state::{AppState, ConnectionStatus, Conversation, ImageData, Message, MessageSender};

// Re-export the connection type for external use
pub type WsConnection = tokio_tungstenite::WebSocketStream<
    tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
>;

// ============================================
// Client -> Server message types
// ============================================

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ImagePayload {
    pub data: String,
    pub mimetype: String,
}

// ============================================
// Server -> Client message types
// ============================================

#[derive(Debug, Serialize, Deserialize)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct ConversationInfo {
    pub id: String,
    #[serde(rename = "lastMessage")]
    pub last_message: Option<String>,
    #[serde(rename = "lastMessageTime")]
    pub last_message_time: Option<i64>,
    #[serde(rename = "messageCount")]
    pub message_count: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HistoryMessage {
    pub role: String,
    pub content: String,
    pub timestamp: Option<i64>,
}

/// Shared WebSocket sender wrapped in Arc for thread-safe access
static WS_SENDER: std::sync::OnceLock<
    tokio::sync::Mutex<
        Option<
            futures_util::stream::SplitSink<WsConnection, WsMessage>,
        >,
    >,
> = std::sync::OnceLock::new();

/// Connect to the WebSocket server
pub async fn connect(url: &str) -> Result<WsConnection, Box<dyn std::error::Error + Send + Sync>> {
    info!("Attempting WebSocket connection to {}", url);

    let (ws_stream, _response) = connect_async(url).await?;
    info!("WebSocket connection established");

    Ok(ws_stream)
}

/// Handle incoming WebSocket messages
pub async fn handle_messages(ws: WsConnection, mut state: Signal<AppState>) {
    let (write, mut read) = ws.split();

    // Store sender for later use
    let sender_lock = WS_SENDER.get_or_init(|| tokio::sync::Mutex::new(None));
    *sender_lock.lock().await = Some(write);

    // Subscribe to notifications
    if let Err(e) = send_subscribe().await {
        info!("Failed to subscribe to events: {:?}", e);
    }

    // Request conversations list
    if let Err(e) = send_list_conversations().await {
        info!("Failed to request conversations: {:?}", e);
    }

    // Read incoming messages
    while let Some(msg_result) = read.next().await {
        match msg_result {
            Ok(WsMessage::Text(text)) => {
                match serde_json::from_str::<WSServerMessage>(&text) {
                    Ok(server_msg) => {
                        handle_server_message(server_msg, &mut state);
                    }
                    Err(e) => {
                        info!("Failed to parse server message: {:?} - raw: {}", e, text);
                    }
                }
            }
            Ok(WsMessage::Close(_)) => {
                info!("WebSocket connection closed by server");
                state.write().connection_status = ConnectionStatus::Disconnected;
                break;
            }
            Err(e) => {
                info!("WebSocket error: {:?}", e);
                state.write().connection_status = ConnectionStatus::Disconnected;
                break;
            }
            _ => {}
        }
    }

    // Clear sender on disconnect
    if let Some(lock) = WS_SENDER.get() {
        *lock.lock().await = None;
    }
}

/// Handle a parsed server message
fn handle_server_message(msg: WSServerMessage, state: &mut Signal<AppState>) {
    match msg {
        WSServerMessage::Response {
            id,
            reply_to,
            conversation_id,
            body,
            image,
            ..
        } => {
            info!("Received response for message {} in {:?}", reply_to, conversation_id);

            let image_data = image.map(|img| ImageData {
                data: img.data,
                mimetype: img.mimetype,
            });

            let response = Message::new_assistant(id, body, image_data);

            if let Some(conv_id) = conversation_id {
                state.write().add_response_to_conversation(&conv_id, &reply_to, response);
            }
        }
        WSServerMessage::Typing {
            reply_to,
            conversation_id,
            is_typing,
            ..
        } => {
            info!("Typing indicator for {} in {:?}: {}", reply_to, conversation_id, is_typing);
            // Only update typing if it's for the current conversation
            let mut state_write = state.write();
            if let Some(conv_id) = conversation_id {
                if state_write.current_conversation_id() == Some(&conv_id) {
                    state_write.is_typing = is_typing;
                }
            }
        }
        WSServerMessage::Notification {
            title,
            body,
            category,
            ..
        } => {
            info!("Notification [{}]: {} - {}", category, title, body);
            // TODO: Show as toast/notification in UI
        }
        WSServerMessage::Error {
            reply_to,
            conversation_id,
            message,
            ..
        } => {
            info!("Error received: {}", message);
            if let (Some(reply_id), Some(conv_id)) = (reply_to, conversation_id) {
                state.write().mark_message_error_in_conversation(&conv_id, &reply_id, message);
            }
        }
        WSServerMessage::ConversationsList {
            conversations,
            ..
        } => {
            info!("Received {} conversations", conversations.len());
            let mut state_write = state.write();
            state_write.loading_conversations = false;

            for conv_info in conversations {
                let conv = Conversation::from_server(
                    conv_info.id,
                    conv_info.last_message,
                    conv_info.last_message_time,
                    conv_info.message_count,
                );
                state_write.upsert_conversation(conv);
            }
        }
        WSServerMessage::History {
            conversation_id,
            messages,
            ..
        } => {
            info!("Received {} history messages for {}", messages.len(), conversation_id);
            let mut state_write = state.write();

            let parsed_messages: Vec<Message> = messages
                .into_iter()
                .filter_map(|m| {
                    let sender = match m.role.as_str() {
                        "user" => MessageSender::User,
                        "assistant" => MessageSender::Assistant,
                        "system" => MessageSender::System,
                        _ => return None,
                    };
                    Some(Message {
                        id: Uuid::new_v4().to_string(),
                        body: m.content,
                        timestamp: m.timestamp
                            .and_then(|t| chrono::DateTime::from_timestamp_millis(t))
                            .unwrap_or_else(Utc::now),
                        sender,
                        status: crate::state::MessageStatus::Delivered,
                        image: None,
                    })
                })
                .collect();

            state_write.set_conversation_history(&conversation_id, parsed_messages);
        }
        WSServerMessage::ConversationCreated {
            conversation_id,
            title,
            ..
        } => {
            info!("Conversation created: {} ({:?})", conversation_id, title);
            state.write().create_conversation(conversation_id, title);
        }
        WSServerMessage::ConversationDeleted {
            conversation_id,
            ..
        } => {
            info!("Conversation deleted: {}", conversation_id);
            state.write().delete_conversation(&conversation_id);
        }
        WSServerMessage::Pong { .. } => {
            // Heartbeat response, nothing to do
        }
    }
}

/// Send a chat message to a specific conversation
pub async fn send_message(
    conversation_id: &str,
    text: &str,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let msg_id = Uuid::new_v4().to_string();
    let msg = WSClientMessage::Chat {
        id: msg_id.clone(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        conversation_id: conversation_id.to_string(),
        body: text.to_string(),
        image: None,
        reply_to: None,
    };

    send_ws_message(&msg).await?;
    Ok(msg_id)
}

/// Request list of conversations
pub async fn send_list_conversations() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg = WSClientMessage::ListConversations {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
    };

    send_ws_message(&msg).await
}

/// Request history for a specific conversation
pub async fn send_get_history(
    conversation_id: &str,
    limit: Option<u32>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg = WSClientMessage::GetHistory {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        conversation_id: conversation_id.to_string(),
        limit,
    };

    send_ws_message(&msg).await
}

/// Create a new conversation
pub async fn send_create_conversation(
    title: Option<String>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg = WSClientMessage::CreateConversation {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        title,
    };

    send_ws_message(&msg).await
}

/// Delete a conversation
pub async fn send_delete_conversation(
    conversation_id: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg = WSClientMessage::DeleteConversation {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        conversation_id: conversation_id.to_string(),
    };

    send_ws_message(&msg).await
}

/// Send a subscribe message
async fn send_subscribe() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let msg = WSClientMessage::Subscribe {
        id: Uuid::new_v4().to_string(),
        timestamp: chrono::Utc::now().timestamp_millis(),
        events: vec!["notifications".to_string(), "reminders".to_string()],
    };

    send_ws_message(&msg).await
}

/// Internal helper to send any WebSocket message
async fn send_ws_message(
    msg: &WSClientMessage,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let sender_lock = WS_SENDER
        .get()
        .ok_or("WebSocket not initialized")?;

    let mut guard = sender_lock.lock().await;
    let sender = guard
        .as_mut()
        .ok_or("WebSocket sender not available")?;

    let json = serde_json::to_string(msg)?;
    sender.send(WsMessage::Text(json.into())).await?;

    Ok(())
}
