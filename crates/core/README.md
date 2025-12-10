# prsnl-core

Platform-agnostic domain types and traits for PrsnlAssistant.

## Overview

This crate defines the shared types, traits, and protocol structures used across all platform implementations. It contains no platform-specific code and can be compiled for any target (native or WASM).

## Modules

### types/

Domain model types representing the application's core concepts:

- **Message** - A chat message with sender, body, timestamp, status, and optional image
- **MessageSender** - Enum: `User`, `Assistant`, `System`
- **MessageStatus** - Enum: `Sending`, `Sent`, `Delivered`, `Error(String)`
- **ImageData** - Base64-encoded image with mimetype
- **Conversation** - A collection of messages with metadata
- **ConnectionStatus** - Enum: `Connecting`, `Connected`, `Disconnected`, `Reconnecting`

### traits.rs

Platform abstraction traits (ports in hexagonal architecture):

- **Transport** - WebSocket communication interface
  - `connect()`, `disconnect()`
  - `send_chat()`, `send_list_conversations()`, `send_get_history()`
  - `send_create_conversation()`, `send_delete_conversation()`
  - `is_connected()`

- **EventBus** - Cross-feature event communication
  - `publish()` - Send an event to all subscribers
  - `subscribe()` - Get a stream of events

### events.rs

Application events for cross-feature communication:

```rust
pub enum AppEvent {
    // Connection
    ConnectionChanged(ConnectionStatus),

    // Conversations
    ConversationSelected(String),
    ConversationCreated { id: String, title: Option<String> },
    ConversationDeleted(String),
    ConversationsLoaded(Vec<Conversation>),

    // Chat
    MessageSent { conv_id: String, message: Message },
    MessageReceived { conv_id: String, message: Message },
    MessageError { conv_id: String, msg_id: String, error: String },
    TypingChanged { conv_id: String, is_typing: bool },
    HistoryLoaded { conv_id: String, messages: Vec<Message> },

    // Settings
    ServerUrlChanged(String),
    SettingsModalToggled(bool),

    // Navigation
    NavigateToList,
    NavigateToChat(String),
}
```

### protocol.rs

WebSocket message definitions for client-server communication:

**Client -> Server (`WSClientMessage`):**
- `Chat` - Send a message
- `Ping` - Keepalive
- `Subscribe` - Subscribe to events
- `ListConversations` - Request conversation list
- `GetHistory` - Request message history
- `CreateConversation` - Create new conversation
- `DeleteConversation` - Delete a conversation

**Server -> Client (`WSServerMessage`):**
- `Response` - AI response to a message
- `Pong` - Keepalive response
- `Notification` - System notification
- `Error` - Error response
- `Typing` - Typing indicator
- `ConversationsList` - List of conversations
- `History` - Message history
- `ConversationCreated` - Confirmation of creation
- `ConversationDeleted` - Confirmation of deletion

## Key Types

### Message

```rust
pub struct Message {
    pub id: String,
    pub body: String,
    pub timestamp: DateTime<Utc>,
    pub sender: MessageSender,
    pub status: MessageStatus,
    pub image: Option<ImageData>,
}

impl Message {
    pub fn new_user(body: String) -> Self;
    pub fn new_user_with_image(body: String, image: ImageData) -> Self;
    pub fn new_assistant(id: String, body: String, image: Option<ImageData>) -> Self;
    pub fn new_system(body: String) -> Self;
}
```

### Conversation

```rust
pub struct Conversation {
    pub id: String,
    pub title: String,
    pub messages: Vec<Message>,
    pub last_message_time: Option<DateTime<Utc>>,
    pub last_message_preview: Option<String>,
    pub message_count: u32,
    pub pending_messages: HashSet<String>,
}

impl Conversation {
    pub fn new(id: String, title: Option<String>) -> Self;
    pub fn from_server(id: String, last_message: Option<String>, last_message_time: Option<i64>, message_count: u32) -> Self;
    pub fn add_user_message(&mut self, message: Message);
    pub fn add_response(&mut self, reply_to: &str, response: Message);
    pub fn mark_message_error(&mut self, id: &str, error: String);
    pub fn set_messages(&mut self, messages: Vec<Message>);
}
```

### Transport Trait

```rust
pub trait Transport: Send + Sync + 'static {
    fn connect(&self, url: String, event_bus: Arc<dyn EventBus>) -> TransportResultVoid;
    fn disconnect(&self) -> TransportResultVoid;
    fn send_chat(&self, conv_id: String, text: String, image: Option<ImagePayload>) -> TransportResult<String>;
    fn send_list_conversations(&self) -> TransportResultVoid;
    fn send_get_history(&self, conv_id: String, limit: Option<u32>) -> TransportResultVoid;
    fn send_create_conversation(&self, title: Option<String>) -> TransportResultVoid;
    fn send_delete_conversation(&self, conv_id: String) -> TransportResultVoid;
    fn is_connected(&self) -> bool;
}
```

### EventBus Trait

```rust
pub trait EventBus: Send + Sync + 'static {
    fn publish(&self, event: AppEvent);
    fn subscribe(&self) -> EventStream;
}
```

## Platform-Specific Result Types

The crate provides conditional type aliases for async return types:

```rust
// Native (requires Send)
pub type TransportResult<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send>>;
pub type EventStream = Pin<Box<dyn futures::Stream<Item = AppEvent> + Send>>;

// WASM (no Send requirement)
pub type TransportResult<T> = Pin<Box<dyn Future<Output = Result<T, String>>>>;
pub type EventStream = Pin<Box<dyn futures::Stream<Item = AppEvent>>>;
```

## Dependencies

```toml
[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
uuid = { version = "1", features = ["v4", "js"] }
chrono = { version = "0.4", features = ["serde", "wasmbind"] }
base64 = "0.22"
async-trait = "0.1"
futures = "0.3"
```

All types derive `Serialize` and `Deserialize` for JSON communication.

## Usage

Add to your `Cargo.toml`:

```toml
[dependencies]
prsnl-core = { path = "../core" }
```

Import commonly used types:

```rust
use prsnl_core::{
    // Types
    Message, MessageSender, MessageStatus, ImageData,
    Conversation, ConnectionStatus,
    // Events
    AppEvent,
    // Protocol
    WSClientMessage, WSServerMessage, ImagePayload,
    // Traits
    Transport, EventBus, SharedTransport, SharedEventBus,
    TransportResult, TransportResultVoid, EventStream,
};
```
