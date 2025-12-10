//! Connection status types

use serde::{Deserialize, Serialize};

/// WebSocket connection status
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConnectionStatus {
    Connecting,
    Connected,
    #[default]
    Disconnected,
    Reconnecting,
}
