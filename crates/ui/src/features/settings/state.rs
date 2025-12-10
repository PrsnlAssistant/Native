//! Settings feature state

use dioxus::prelude::*;

const DEFAULT_SERVER_URL: &str = "ws://10.8.0.8:8765/ws";

/// Internal state for settings
#[derive(Debug, Clone)]
pub struct SettingsStateInner {
    pub server_url: String,
    pub modal_open: bool,
}

/// State for the settings feature (wraps a Signal)
#[derive(Clone, Copy)]
pub struct SettingsState {
    inner: Signal<SettingsStateInner>,
}

impl SettingsState {
    /// Create new settings state
    pub fn new() -> Self {
        Self {
            inner: Signal::new(SettingsStateInner {
                server_url: DEFAULT_SERVER_URL.to_string(),
                modal_open: false,
            }),
        }
    }

    // ============================================
    // Read accessors
    // ============================================

    /// Get current server URL
    pub fn server_url(&self) -> String {
        self.inner.read().server_url.clone()
    }

    /// Check if settings modal is open
    pub fn is_modal_open(&self) -> bool {
        self.inner.read().modal_open
    }

    // ============================================
    // Mutations (use mut self for Signal write access)
    // ============================================

    /// Set server URL
    pub fn set_server_url(&mut self, url: String) {
        self.inner.write().server_url = url;
    }

    /// Open settings modal
    pub fn open_modal(&mut self) {
        self.inner.write().modal_open = true;
    }

    /// Close settings modal
    pub fn close_modal(&mut self) {
        self.inner.write().modal_open = false;
    }

    /// Toggle settings modal
    pub fn toggle_modal(&mut self) {
        let mut inner = self.inner.write();
        inner.modal_open = !inner.modal_open;
    }
}

impl Default for SettingsState {
    fn default() -> Self {
        Self::new()
    }
}
