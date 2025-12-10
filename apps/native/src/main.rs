//! Native application entry point for PrsnlAssistant

use std::sync::Arc;

use dioxus::prelude::*;
use futures::StreamExt;
use prsnl_core::{AppEvent, ConnectionStatus, SharedEventBus, SharedTransport};
use prsnl_platform_native::{NativeEventBus, NativeTransport};
use prsnl_ui::{
    provide_chat_feature, provide_conversations_feature, provide_settings_feature, ResponsiveApp,
};
use tracing::info;

fn main() {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    // Launch the Dioxus app
    dioxus::launch(App);
}

#[component]
fn App() -> Element {
    // ============================================
    // Initialize shared infrastructure
    // ============================================

    let event_bus: SharedEventBus =
        use_context_provider(|| Arc::new(NativeEventBus::new()) as SharedEventBus);
    let transport: SharedTransport =
        use_context_provider(|| Arc::new(NativeTransport::new()) as SharedTransport);

    // ============================================
    // Initialize features
    // ============================================

    // Conversations feature
    let (conv_state, conv_service) = use_hook(|| {
        provide_conversations_feature(event_bus.clone(), transport.clone())
    });
    use_context_provider(|| conv_state.clone());
    use_context_provider(|| conv_service.clone());

    // Chat feature
    let (chat_state, chat_service) =
        use_hook(|| provide_chat_feature(event_bus.clone(), transport.clone()));
    use_context_provider(|| chat_state.clone());
    use_context_provider(|| chat_service.clone());

    // Settings feature
    let (settings_state, settings_service) =
        use_hook(|| provide_settings_feature(event_bus.clone()));
    use_context_provider(|| settings_state.clone());
    use_context_provider(|| settings_service.clone());

    // ============================================
    // Subscribe features to events
    // ============================================

    {
        let conv_service = conv_service.clone();
        let chat_service = chat_service.clone();
        let settings_service = settings_service.clone();
        use_effect(move || {
            conv_service.subscribe_to_events();
            chat_service.subscribe_to_events();
            settings_service.subscribe_to_events();
        });
    }

    // ============================================
    // Connection state
    // ============================================

    let mut connection_status = use_signal(|| ConnectionStatus::Disconnected);
    let mut reconnect_trigger = use_signal(|| 0u32);

    // Provide connection status to children
    use_context_provider(|| connection_status);

    // Listen for connection changes
    use_effect({
        let event_bus = event_bus.clone();
        move || {
            let mut rx = event_bus.subscribe();
            spawn(async move {
                while let Some(event) = rx.next().await {
                    if let AppEvent::ConnectionChanged(status) = event {
                        connection_status.set(status);
                    }
                }
            });
        }
    });

    // WebSocket connection effect
    use_effect({
        let transport = transport.clone();
        let event_bus = event_bus.clone();
        let settings_state = settings_state.clone();
        move || {
            let _trigger = reconnect_trigger.read(); // Subscribe to trigger
            let url = settings_state.server_url();
            let transport = transport.clone();
            let event_bus = event_bus.clone();

            spawn(async move {
                info!("Connecting to WebSocket server: {}", url);
                if let Err(e) = transport.connect(url, event_bus).await {
                    info!("Connection error: {}", e);
                }
            });
        }
    });

    // Listen for server URL changes to trigger reconnection
    use_effect({
        let event_bus = event_bus.clone();
        move || {
            let mut rx = event_bus.subscribe();
            spawn(async move {
                while let Some(event) = rx.next().await {
                    if let AppEvent::ServerUrlChanged(_) = event {
                        let current = *reconnect_trigger.read();
                        reconnect_trigger.set(current.wrapping_add(1));
                    }
                }
            });
        }
    });

    // ============================================
    // Render
    // ============================================

    rsx! {
        ResponsiveApp {}
    }
}
