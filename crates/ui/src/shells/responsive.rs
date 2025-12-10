//! Responsive shell that switches between mobile and desktop layouts
//!
//! This module provides viewport detection and automatic layout switching
//! based on the viewport width and platform.
//!
//! Platform behavior:
//! - **Android/iOS**: Always uses MobileShell (navigation-based)
//! - **Web**: Uses viewport detection at 768px breakpoint
//! - **Desktop (Linux/macOS/Windows)**: Uses DesktopShell (side-by-side)

use dioxus::prelude::*;

// Conditionally import shells based on what's needed for each platform
#[cfg(any(target_os = "android", target_os = "ios", target_arch = "wasm32"))]
use super::MobileShell;
#[cfg(any(target_arch = "wasm32", all(not(target_os = "android"), not(target_os = "ios"))))]
use super::DesktopShell;

/// Breakpoint for switching between mobile and desktop layouts (in pixels)
#[allow(dead_code)]
const BREAKPOINT_WIDTH: u32 = 768;

/// Hook to detect viewport width with automatic updates on resize (Web)
///
/// Returns the current viewport width in pixels.
#[cfg(target_arch = "wasm32")]
fn use_viewport_width() -> Signal<u32> {
    use std::cell::RefCell;
    use std::rc::Rc;
    use wasm_bindgen::closure::Closure;
    use wasm_bindgen::JsCast;

    let width = use_signal(|| get_window_width());

    // Set up resize listener on mount
    use_effect(move || {
        let window = match web_sys::window() {
            Some(w) => w,
            None => {
                tracing::warn!("No window object available");
                return;
            }
        };

        // Wrap the signal in Rc<RefCell> for interior mutability
        let width_cell = Rc::new(RefCell::new(width));
        let width_for_closure = width_cell.clone();

        // Create closure for resize event - use Fn since we use RefCell
        let closure = Closure::wrap(Box::new(move || {
            let new_width = get_window_width();
            width_for_closure.borrow_mut().set(new_width);
        }) as Box<dyn Fn()>);

        // Add event listener
        let result = window.add_event_listener_with_callback(
            "resize",
            closure.as_ref().unchecked_ref(),
        );

        if let Err(e) = result {
            tracing::error!("Failed to add resize listener: {:?}", e);
        }

        // Keep closure alive for the lifetime of the component
        // by leaking it (it will be cleaned up when the page unloads)
        closure.forget();
    });

    width
}

/// Get the current window inner width (Web)
#[cfg(target_arch = "wasm32")]
fn get_window_width() -> u32 {
    web_sys::window()
        .and_then(|w| w.inner_width().ok())
        .and_then(|v| v.as_f64())
        .map(|f| f as u32)
        .unwrap_or(BREAKPOINT_WIDTH) // Default to breakpoint if detection fails
}

/// Responsive application shell that automatically switches layout based on platform/viewport
///
/// # Platform Behavior
/// - **Android/iOS**: Always uses `MobileShell` (navigation-based layout)
/// - **Web**: Uses viewport detection - `MobileShell` for < 768px, `DesktopShell` for >= 768px
/// - **Desktop (Linux/macOS/Windows)**: Always uses `DesktopShell` (side-by-side layout)
///
/// # Example
///
/// ```rust,ignore
/// use dioxus::prelude::*;
/// use prsnl_ui::ResponsiveApp;
///
/// fn main() {
///     dioxus::launch(|| {
///         rsx! { ResponsiveApp {} }
///     });
/// }
/// ```
#[component]
pub fn ResponsiveApp() -> Element {
    // On Android/iOS, always use mobile layout
    #[cfg(any(target_os = "android", target_os = "ios"))]
    {
        tracing::debug!("ResponsiveApp: Mobile platform detected, using MobileShell");
        rsx! {
            MobileShell {}
        }
    }

    // On Web, use viewport-based detection
    #[cfg(target_arch = "wasm32")]
    {
        let width = use_viewport_width();
        let is_mobile = *width.read() < BREAKPOINT_WIDTH;

        tracing::debug!("ResponsiveApp: Web platform, viewport width: {}px, is_mobile: {}", *width.read(), is_mobile);

        rsx! {
            if is_mobile {
                MobileShell {}
            } else {
                DesktopShell {}
            }
        }
    }

    // On desktop (Linux, macOS, Windows), always use desktop layout
    #[cfg(all(
        not(target_arch = "wasm32"),
        not(target_os = "android"),
        not(target_os = "ios")
    ))]
    {
        tracing::debug!("ResponsiveApp: Desktop platform detected, using DesktopShell");
        rsx! {
            DesktopShell {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_breakpoint_value() {
        assert_eq!(BREAKPOINT_WIDTH, 768);
    }
}
