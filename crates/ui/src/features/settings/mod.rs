//! Settings feature module
//!
//! This feature manages app settings like server URL configuration.

mod state;
mod service;
pub mod components;

pub use state::SettingsState;
pub use service::SettingsService;

use prsnl_core::SharedEventBus;

/// Initialize the settings feature
pub fn provide_settings_feature(event_bus: SharedEventBus) -> (SettingsState, SettingsService) {
    let state = SettingsState::new();
    let service = SettingsService::new(state.clone(), event_bus);
    (state, service)
}
