# PrsnlAssistant Native

A cross-platform AI assistant client built with Rust and Dioxus. The application runs on desktop (Linux, macOS, Windows), web browsers, and Android devices, all from a shared codebase.

## Overview

PrsnlAssistant is a WebSocket-based chat client that connects to an AI assistant backend. It features:

- Real-time bidirectional communication via WebSocket
- Conversation management (create, list, delete)
- Message history with optimistic updates
- Image attachments
- Responsive UI adapting to desktop and mobile viewports
- Dark theme with custom design tokens

## Project Structure

```
Native/
├── apps/                       # Application entry points
│   ├── native/                 # Desktop and Android app
│   └── web/                    # Web (WASM) app
├── crates/                     # Library crates
│   ├── core/                   # Platform-agnostic domain types and traits
│   ├── platform-native/        # Native platform adapter (tokio + tungstenite)
│   ├── platform-web/           # Web platform adapter (web-sys + futures-channel)
│   └── ui/                     # Dioxus UI components and shells
├── assets/                     # Static assets and CSS
│   ├── input.css               # Tailwind CSS source with custom theme
│   └── tailwind.css            # Generated Tailwind output
├── Cargo.toml                  # Workspace configuration
├── Taskfile.yml                # Task runner commands
├── package.json                # Node.js dependencies for Tailwind
└── Dioxus.toml                 # Dioxus build configuration
```

## Prerequisites

### Required

- **Rust** (latest stable) - Install via [rustup](https://rustup.rs/)
- **Node.js** (v18+) - For Tailwind CSS compilation
- **Dioxus CLI** - Install with `cargo install dioxus-cli`

### For Android Development

- **Android SDK** with platform-tools and build-tools
- **Android NDK** (version 26.x recommended)
- **Rust Android targets**: `rustup target add aarch64-linux-android x86_64-linux-android`

Set environment variables:
```bash
export ANDROID_HOME=$HOME/Android/Sdk
export ANDROID_NDK_HOME=$ANDROID_HOME/ndk/<version>
```

### NixOS Users

This project includes a `flake.nix` for development environment setup:
```bash
nix develop
```

## Quick Start

1. **Install dependencies**:
   ```bash
   npm install
   ```

2. **Run desktop development server**:
   ```bash
   task dev
   ```
   This builds Tailwind CSS and launches the desktop app with hot reload.

3. **Run web development server**:
   ```bash
   task web:dev
   ```
   Opens a local server at `http://localhost:8080`.

## Available Commands

### Development

| Command | Description |
|---------|-------------|
| `task dev` | Run desktop app in development mode |
| `task dev:hot` | Run desktop with hot reload |
| `task web:dev` | Run web app in development mode |
| `task web:dev:hot` | Run web app with hot reload |
| `task check` | Cargo check for native packages |
| `task check:web` | Cargo check for WASM target |
| `task check:all` | Check all packages |

### Building

| Command | Description |
|---------|-------------|
| `task build` | Build desktop release |
| `task web:build` | Build web release |
| `task web:bundle` | Bundle web app for distribution |

### Android

| Command | Description |
|---------|-------------|
| `task android:setup` | Install Rust Android targets |
| `task android:check` | Verify Android build environment |
| `task android:build` | Build APK for arm64 devices |
| `task android:build:x86` | Build APK for x86_64 emulator |
| `task deploy` | Build, install, and start on device |
| `task deploy:fresh` | Uninstall, then deploy clean |

### Device Management

| Command | Description |
|---------|-------------|
| `task adb:devices` | List connected ADB devices |
| `task adb:wireless` | Interactive wireless ADB setup guide |
| `task adb:pair IP=<ip:port> CODE=<code>` | Pair with wireless device |
| `task adb:connect IP=<ip:port>` | Connect to paired device |
| `task logs` | Show filtered app logs |
| `task start` | Start the app on device |
| `task stop` | Force stop the app |

### Tailwind CSS

| Command | Description |
|---------|-------------|
| `task tailwind:build` | Build Tailwind CSS once |
| `task tailwind:watch` | Watch and rebuild on changes |
| `task tailwind:minify` | Build minified CSS for production |

## Tech Stack

| Technology | Purpose |
|------------|---------|
| [Dioxus 0.7](https://dioxuslabs.com/) | Cross-platform UI framework (React-like) |
| [Tailwind CSS 4](https://tailwindcss.com/) | Utility-first CSS framework |
| [tokio](https://tokio.rs/) | Async runtime for native platforms |
| [tokio-tungstenite](https://github.com/snapview/tokio-tungstenite) | WebSocket client for native |
| [web-sys](https://rustwasm.github.io/wasm-bindgen/) | WebSocket client for WASM |
| [serde](https://serde.rs/) | Serialization framework |

## Architecture

The project follows a hexagonal (ports and adapters) architecture:

```
┌─────────────────────────────────────────────────────────────────┐
│                         Applications                             │
│  ┌─────────────────────┐           ┌─────────────────────┐      │
│  │    apps/native      │           │      apps/web       │      │
│  │ (Desktop + Android) │           │       (WASM)        │      │
│  └─────────────────────┘           └─────────────────────┘      │
├─────────────────────────────────────────────────────────────────┤
│                          UI Layer                                │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                       crates/ui                            │  │
│  │  Features: chat, conversations, settings, media            │  │
│  │  Shells: DesktopShell, MobileShell, ResponsiveApp          │  │
│  └───────────────────────────────────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                      Platform Adapters                           │
│  ┌─────────────────────────┐   ┌─────────────────────────────┐  │
│  │  crates/platform-native │   │    crates/platform-web      │  │
│  │  NativeTransport        │   │    WebTransport             │  │
│  │  NativeEventBus         │   │    WebEventBus              │  │
│  │  (tokio + tungstenite)  │   │    (web-sys + futures)      │  │
│  └─────────────────────────┘   └─────────────────────────────┘  │
├─────────────────────────────────────────────────────────────────┤
│                         Core Layer                               │
│  ┌───────────────────────────────────────────────────────────┐  │
│  │                      crates/core                           │  │
│  │  Types: Message, Conversation, ConnectionStatus            │  │
│  │  Traits: Transport, EventBus                               │  │
│  │  Protocol: WSClientMessage, WSServerMessage                │  │
│  └───────────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

### Key Patterns

- **Event Bus**: Cross-feature communication via `AppEvent` enum
- **Signal-based State**: Dioxus Signals for reactive UI updates
- **Optimistic Updates**: User messages appear immediately, status updates on server response
- **Platform Traits**: `Transport` and `EventBus` traits allow platform-specific implementations

## Configuration

### Server URL

The default WebSocket server URL can be changed in the application settings modal. The settings are managed by the `SettingsState` in the UI crate.

### Tailwind Theme

Custom theme tokens are defined in `assets/input.css`:

- Background colors: `bg-primary`, `bg-secondary`, `bg-tertiary`
- Text colors: `text-primary`, `text-secondary`, `text-muted`
- Accent color: `accent` (blue)
- Status colors: `success`, `warning`, `error`

## Development Workflow

1. **Start Tailwind watcher** in one terminal:
   ```bash
   task tailwind:watch
   ```

2. **Run the app** in another terminal:
   ```bash
   task dev        # Desktop
   task web:dev    # Web
   ```

3. Changes to Rust code trigger hot reload. Changes to CSS in `input.css` are picked up by the Tailwind watcher.

## Testing

```bash
task test        # Run all tests
task test:watch  # Run tests in watch mode
```

## Troubleshooting

### Android Build Issues

1. Run `task android:check` to verify your environment
2. Ensure `ANDROID_HOME` and `ANDROID_NDK_HOME` are set correctly
3. Install required SDK components: `sdkmanager 'platforms;android-34' 'ndk;26.1.10909125'`

### Tailwind Not Updating

1. Ensure Node.js dependencies are installed: `npm install`
2. Check that `assets/tailwind.css` exists
3. Run `task tailwind:build` to regenerate

### WebSocket Connection Fails

1. Verify the server URL in settings
2. Check browser/device network connectivity
3. Review console/logcat output for error messages

## License

This project is proprietary software.
