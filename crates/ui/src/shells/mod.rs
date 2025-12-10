//! Application shells for different viewport sizes

mod desktop;
mod mobile;
mod responsive;

pub use desktop::DesktopShell;
pub use mobile::MobileShell;
pub use responsive::ResponsiveApp;
