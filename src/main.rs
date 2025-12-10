//! Personal Assistant Native App
//!
//! A Dioxus-based cross-platform application that connects to the PrsnlAssistant
//! backend via WebSocket for chat functionality with multiple conversations.

mod shared;
mod features;
mod app;

use dioxus_logger::tracing::Level;

fn main() {
    // Initialize logger
    dioxus_logger::init(Level::INFO).expect("failed to init logger");
    dioxus_logger::tracing::info!("Starting Personal Assistant app");

    // Launch the app
    dioxus::launch(app::App);
}
