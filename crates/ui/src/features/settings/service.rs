//! Settings feature service

use dioxus::prelude::spawn;
use futures::StreamExt;
use tracing::info;

use prsnl_core::{AppEvent, SharedEventBus};
use super::state::SettingsState;

/// Service for managing settings
#[derive(Clone)]
pub struct SettingsService {
    state: SettingsState,
    event_bus: SharedEventBus,
}

impl SettingsService {
    /// Create a new settings service
    pub fn new(state: SettingsState, event_bus: SharedEventBus) -> Self {
        Self { state, event_bus }
    }

    /// Subscribe to relevant events
    pub fn subscribe_to_events(&self) {
        let mut state = self.state;
        let mut rx = self.event_bus.subscribe();

        spawn(async move {
            while let Some(event) = rx.next().await {
                match event {
                    AppEvent::SettingsModalToggled(open) => {
                        if open {
                            state.open_modal();
                        } else {
                            state.close_modal();
                        }
                    }
                    _ => {}
                }
            }
        });
    }

    /// Open settings modal
    pub fn open_modal(&self) {
        let mut state = self.state;
        state.open_modal();
    }

    /// Close settings modal
    pub fn close_modal(&self) {
        let mut state = self.state;
        state.close_modal();
    }

    /// Update server URL and trigger reconnection
    pub fn update_server_url(&self, url: String) {
        info!("Updating server URL to: {}", url);
        let mut state = self.state;
        state.set_server_url(url.clone());
        state.close_modal();
        self.event_bus.publish(AppEvent::ServerUrlChanged(url));
    }
}
