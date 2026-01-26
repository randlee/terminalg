//! Settings adapter - bridges `TerminalG` to Zed's settings system
//!
//! This module provides integration between `TerminalG`'s existing settings types
//! and Zed's settings infrastructure. It allows `TerminalG` settings to be
//! registered with Zed's `SettingsStore`.

use gpui::App;
use settings::Settings;

/// Initialize `TerminalG` settings with Zed's settings system
///
/// This registers `TerminalG`-specific settings with Zed's global `SettingsStore`.
/// Call this during app initialization after `settings::init()`.
///
/// # Example
///
/// ```no_run
/// use gpui::App;
///
/// fn main() {
///     gpui::Application::new().run(|cx: &mut App| {
///         settings::init(cx);
///         settings_adapter::init(cx);
///         // Settings are now ready to use
///     });
/// }
/// ```
pub fn init(cx: &mut App) {
    // Register terminal settings with Zed's system
    terminal::terminal_settings::TerminalSettings::register(cx);
    tracing::info!("TerminalG settings adapter initialized");
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // Basic compilation test - verifying the module structure exists
        // The init function is tested via integration tests that require App context
    }
}
