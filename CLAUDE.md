# AI Agent Instructions for PrsnlAssistant Native

This document provides guidance for AI agents working on the PrsnlAssistant Native codebase.

## Environment

This is a **NixOS environment** with `nix-shell` available. When creating scripts:
- For general usage: `#!/usr/bin/env bash`
- For one-off scripts needing custom dependencies: `#!/usr/bin/env nix-shell` with `#!nix-shell -i bash`

## Architecture Overview

The project follows a **hexagonal (ports and adapters) architecture**:

```
apps/           -> Application entry points (thin wiring layer)
crates/ui/      -> UI components (features, shells, shared)
crates/core/    -> Domain types and trait definitions (ports)
crates/platform-*/ -> Platform-specific implementations (adapters)
```

### Layer Responsibilities

| Layer | Crate | Responsibility |
|-------|-------|----------------|
| Application | `apps/native`, `apps/web` | Initialize platform adapters, wire up features, launch Dioxus |
| UI | `crates/ui` | Components, state management, event handling |
| Domain | `crates/core` | Types, traits (ports), protocol definitions |
| Platform | `crates/platform-*` | Implement `Transport` and `EventBus` traits |

## Workspace Structure

```
Native/
├── apps/
│   ├── native/src/main.rs      # Desktop + Android entry point
│   └── web/src/main.rs         # WASM entry point
├── crates/
│   ├── core/                   # prsnl-core
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── types/          # Message, Conversation, ConnectionStatus
│   │       ├── traits.rs       # Transport, EventBus traits
│   │       ├── events.rs       # AppEvent enum
│   │       └── protocol.rs     # WebSocket message types
│   ├── platform-native/        # prsnl-platform-native
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── transport.rs    # NativeTransport (tokio-tungstenite)
│   │       └── events.rs       # NativeEventBus (tokio broadcast)
│   ├── platform-web/           # prsnl-platform-web
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── transport.rs    # WebTransport (web-sys WebSocket)
│   │       └── events.rs       # WebEventBus (futures-channel)
│   └── ui/                     # prsnl-ui
│       └── src/
│           ├── lib.rs
│           ├── features/       # Feature modules
│           │   ├── chat/
│           │   ├── conversations/
│           │   ├── settings/
│           │   └── media/
│           ├── shells/         # Responsive shells
│           └── shared/         # Shared components
└── assets/
    └── input.css               # Tailwind source with @theme
```

## Feature Module Pattern

Each feature in `crates/ui/src/features/` follows this structure:

```
feature_name/
├── mod.rs              # Public exports, provide_*_feature() function
├── state.rs            # Signal-wrapped state type
├── service.rs          # Business logic, event handling
├── hooks.rs            # Optional: Custom hooks (use_memo wrappers)
└── components/
    ├── mod.rs          # Re-exports
    └── *.rs            # Individual UI components
```

### Feature Initialization Pattern

```rust
// In feature/mod.rs
pub fn provide_chat_feature(
    event_bus: SharedEventBus,
    transport: SharedTransport,
) -> (ChatState, ChatService) {
    let state = ChatState::new();
    let service = ChatService::new(state.clone(), event_bus, transport);
    (state, service)
}

// In apps/*/main.rs
let (chat_state, chat_service) = use_hook(|| provide_chat_feature(event_bus.clone(), transport.clone()));
use_context_provider(|| chat_state.clone());
use_context_provider(|| chat_service.clone());
```

### State Pattern

State types wrap a Dioxus `Signal` for reactivity:

```rust
#[derive(Clone, Copy)]
pub struct ChatState {
    inner: Signal<ChatStateInner>,
}

impl ChatState {
    pub fn new() -> Self {
        Self { inner: Signal::new(ChatStateInner::default()) }
    }

    // Read accessors - take &self
    pub fn current_messages(&self) -> Vec<Message> {
        self.inner.read().messages.clone()
    }

    // Mutations - take &mut self for Signal write access
    pub fn add_user_message(&mut self, conv_id: &str, message: Message) {
        self.inner.write().messages.push(message);
    }
}
```

### Service Pattern

Services handle business logic and subscribe to the event bus:

```rust
impl ChatService {
    pub fn subscribe_to_events(&self) {
        let mut state = self.state;
        let mut rx = self.event_bus.subscribe();

        spawn(async move {
            while let Some(event) = rx.next().await {
                match event {
                    AppEvent::ConversationSelected(id) => {
                        state.set_current_conversation(Some(id));
                    }
                    // ... handle other events
                    _ => {}
                }
            }
        });
    }
}
```

## Key Patterns and Conventions

### Event Bus for Cross-Feature Communication

Use `AppEvent` to communicate between features. Never call other features' services directly.

```rust
// Publishing an event
self.event_bus.publish(AppEvent::ConversationSelected(id));

// Subscribing (in service.subscribe_to_events())
while let Some(event) = rx.next().await {
    if let AppEvent::ConversationSelected(id) = event {
        // Handle event
    }
}
```

### Reactive Hooks with use_memo

For computed values that depend on state, use `use_memo`:

```rust
let messages = use_memo(move || chat_state.current_messages());
```

### Optimistic Updates

User actions should update UI immediately, then sync with server:

```rust
// 1. Update state immediately
state.add_user_message(&conv_id, msg.clone());

// 2. Publish event for other features
self.event_bus.publish(AppEvent::MessageSent { conv_id, message: msg });

// 3. Send to server asynchronously
spawn(async move {
    transport.send_chat(conv_id, text, image).await;
});
```

### Platform Abstraction

All platform-specific code goes through traits defined in `crates/core`:

```rust
// In crates/core/src/traits.rs
pub trait Transport: Send + Sync + 'static {
    fn connect(&self, url: String, event_bus: Arc<dyn EventBus>) -> TransportResultVoid;
    fn send_chat(&self, conv_id: String, text: String, image: Option<ImagePayload>) -> TransportResult<String>;
    // ...
}

pub trait EventBus: Send + Sync + 'static {
    fn publish(&self, event: AppEvent);
    fn subscribe(&self) -> EventStream;
}
```

## Styling Guidelines

### Tailwind CSS 4

Styles use Tailwind CSS 4 with a custom theme defined in `assets/input.css`:

```css
@theme {
  --color-bg-primary: #0f0f23;
  --color-text-primary: #e0e0e0;
  --color-accent: #1e88e5;
  /* ... */
}
```

### Component Styling

Use Tailwind utility classes directly in `class` attributes:

```rust
rsx! {
    div { class: "flex flex-col h-full bg-bg-primary",
        // ...
    }
}
```

Common patterns:
- Layout: `flex`, `flex-col`, `flex-row`, `items-center`, `justify-between`
- Spacing: `p-4`, `px-4`, `py-2`, `gap-2`, `space-y-2`
- Colors: `bg-bg-primary`, `text-text-primary`, `border-border`
- Sizing: `w-full`, `h-full`, `max-w-[80%]`, `min-h-dvh`

### Custom Component Classes

Defined in `assets/input.css` under `@layer components`:

```css
.message-bubble {
  @apply max-w-[80%] rounded-2xl px-4 py-3 text-[15px] leading-relaxed;
}
```

## Important Files to Know

| File | Purpose |
|------|---------|
| `Cargo.toml` | Workspace definition, shared dependencies |
| `Taskfile.yml` | All build/run/deploy commands |
| `Dioxus.toml` | Dioxus CLI configuration |
| `assets/input.css` | Tailwind source with custom theme |
| `crates/core/src/events.rs` | All application events |
| `crates/core/src/protocol.rs` | WebSocket message types |
| `crates/core/src/traits.rs` | Platform abstraction traits |
| `apps/native/src/main.rs` | Native app wiring |
| `apps/web/src/main.rs` | Web app wiring |

## Common Tasks

### Adding a New Feature

1. Create feature directory in `crates/ui/src/features/`
2. Add `state.rs`, `service.rs`, `components/mod.rs`
3. Export from `mod.rs` with `provide_*_feature()` function
4. Add feature to `crates/ui/src/features/mod.rs` exports
5. Wire up in `apps/*/main.rs`

### Adding a New Event

1. Add variant to `AppEvent` in `crates/core/src/events.rs`
2. Handle in relevant services' `subscribe_to_events()`
3. Publish from UI components or services as needed

### Adding a New Protocol Message

1. Add variant to `WSClientMessage` or `WSServerMessage` in `crates/core/src/protocol.rs`
2. Add method to `Transport` trait in `crates/core/src/traits.rs`
3. Implement in both `platform-native` and `platform-web`

### Conditional Platform Code

```rust
// In crates/ui (with appropriate features)
#[cfg(feature = "desktop")]
fn pick_file() { /* rfd file picker */ }

#[cfg(feature = "web")]
fn pick_file() { /* web-sys file input */ }
```

## Testing

```bash
task test         # Run all tests
task check:all    # Check all targets compile
```

## Debugging

### Native
```bash
RUST_LOG=debug task run
```

### Android
```bash
task logs         # Filtered logcat
task logs:all     # Full logcat
```

### Web
Open browser DevTools console. Tracing logs go to console via `tracing-wasm`.

## Do Not

- Put platform-specific code in `crates/core`
- Call services/state from other features directly (use events)
- Use `async_trait` in traits with web targets (use boxed futures)
- Forget to handle both `Send` (native) and `!Send` (web) in trait bounds
- Commit `target/`, `node_modules/`, or `assets/tailwind.css` (generated)
