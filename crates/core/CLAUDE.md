# AI Agent Instructions for prsnl-core

This document provides guidance for AI agents working on the `prsnl-core` crate.

## Purpose

This is the **domain layer** of the PrsnlAssistant application. It defines:
- Domain types (Message, Conversation, etc.)
- Platform abstraction traits (Transport, EventBus)
- Application events (AppEvent)
- Wire protocol types (WSClientMessage, WSServerMessage)

## Core Principles

### No Platform-Specific Code

This crate must compile for both native and WASM targets. Do not add:
- `tokio` dependencies
- `web-sys` dependencies
- Any async runtime-specific code
- File system operations
- Network implementations

The only exception is conditional compilation for `Send` bounds:
```rust
#[cfg(not(target_arch = "wasm32"))]
pub type TransportResult<T> = Pin<Box<dyn Future<Output = Result<T, String>> + Send>>;

#[cfg(target_arch = "wasm32")]
pub type TransportResult<T> = Pin<Box<dyn Future<Output = Result<T, String>>>>;
```

### All Types Must Be Serializable

Every type should derive `Serialize` and `Deserialize`:
```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Message { /* ... */ }
```

### Keep It Minimal

Only add types that are truly shared across features/platforms. Feature-specific types belong in `crates/ui`.

## File Structure

```
src/
├── lib.rs              # Re-exports public API
├── types/              # Domain types
│   ├── mod.rs
│   ├── message.rs      # Message, MessageSender, MessageStatus, ImageData
│   ├── conversation.rs # Conversation
│   └── connection.rs   # ConnectionStatus
├── traits.rs           # Transport, EventBus traits
├── events.rs           # AppEvent enum
└── protocol.rs         # WebSocket message types
```

## Adding New Types

### Domain Types (in types/)

1. Create file in `types/` directory
2. Derive standard traits:
   ```rust
   #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
   pub struct NewType { /* ... */ }
   ```
3. Export from `types/mod.rs`
4. Re-export from `lib.rs` if commonly used

### Events (in events.rs)

Add new variants to `AppEvent`:
```rust
pub enum AppEvent {
    // Existing events...

    // New event
    NewFeatureEvent { field: Type },
}
```

Events should:
- Use owned types (not references)
- Implement `Clone` (derived from the enum)
- Be documented with their purpose

### Protocol Messages (in protocol.rs)

For client-to-server messages:
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum WSClientMessage {
    // Existing variants...

    #[serde(rename = "new_action")]
    NewAction {
        id: String,
        timestamp: i64,
        // action-specific fields
    },
}
```

For server-to-client messages, add to `WSServerMessage` similarly.

Note the `#[serde(tag = "type")]` attribute - messages are tagged by type field.

## Trait Guidelines

### Transport Trait

Methods return boxed futures (not async fn) to avoid lifetime issues:
```rust
fn some_action(&self, arg: String) -> TransportResultVoid;
```

Not:
```rust
async fn some_action(&self, arg: String) -> Result<(), String>;
```

This pattern supports both `Send` (native) and `!Send` (web) implementations.

### EventBus Trait

Keep the interface simple:
- `publish()` - Fire and forget
- `subscribe()` - Returns a stream

Do not add methods like `publish_and_wait()` or request-response patterns.

## Dependency Rules

Allowed dependencies:
- `serde`, `serde_json` - Serialization
- `uuid` - ID generation
- `chrono` - Timestamps
- `base64` - Image encoding
- `futures` - Stream/Future traits
- `async-trait` - Trait async support (though we use boxed futures instead)

Not allowed:
- Any async runtime (tokio, async-std)
- Any platform API (web-sys, winapi)
- Any UI framework
- Any network implementation

## Common Patterns

### ID Generation

Use UUID v4 for message/conversation IDs:
```rust
use uuid::Uuid;

let id = Uuid::new_v4().to_string();
```

### Timestamps

Use chrono UTC timestamps:
```rust
use chrono::{DateTime, Utc};

let timestamp: DateTime<Utc> = Utc::now();
```

For protocol messages (milliseconds since epoch):
```rust
let timestamp_ms = Utc::now().timestamp_millis();
```

### Implementing Methods

Prefer builder-style constructors:
```rust
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
}
```

## Testing

Write unit tests in the same file:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new_user("Hello".to_string());
        assert_eq!(msg.sender, MessageSender::User);
        assert_eq!(msg.status, MessageStatus::Sending);
    }
}
```

Run tests:
```bash
cargo test -p prsnl-core
```

## Checklist for Changes

Before committing changes to this crate:

- [ ] All types derive `Serialize` and `Deserialize`
- [ ] No platform-specific imports
- [ ] Compiles for native: `cargo check -p prsnl-core`
- [ ] Compiles for WASM: `cargo check -p prsnl-core --target wasm32-unknown-unknown`
- [ ] New types are documented
- [ ] Exports are updated in `lib.rs` if needed
- [ ] Tests pass: `cargo test -p prsnl-core`

## Do Not

- Add UI logic or components
- Add platform-specific implementations
- Use `async_trait` macro on traits (use boxed futures instead)
- Add heavy dependencies
- Put feature-specific state here (belongs in `crates/ui`)
