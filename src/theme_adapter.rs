//! Theme adapter - bridges `TerminalG` to Zed's theme system
//!
//! This module initializes Zed's theme system and provides utilities for
//! accessing theme colors in `TerminalG`'s UI components.

use gpui::App;
use std::sync::Arc;
use theme::{ActiveTheme, LoadThemes, Theme, ThemeRegistry};

/// Initialize Zed's theme system
///
/// Sets up the `ThemeRegistry` with Zed's built-in themes.
/// This should be called during app initialization.
///
/// # Example
///
/// ```no_run
/// use gpui::App;
/// use theme_adapter::init;
///
/// fn main() {
///     gpui::Application::new().run(|cx: &mut App| {
///         theme_adapter::init(cx);
///         // Theme system is now ready to use
///     });
/// }
/// ```
pub fn init(cx: &mut App) {
    // Initialize Zed's theme system with base themes only
    // This includes the fallback "One" theme family (dark and light variants)
    theme::init(LoadThemes::JustBase, cx);
    tracing::info!("Zed theme system initialized");
}

/// Get the currently active theme
///
/// Returns a reference to the active Zed Theme, which contains comprehensive
/// color and style information for the UI.
///
/// # Arguments
///
/// * `cx` - The application context
///
/// # Returns
///
/// An Arc reference to the active `Theme`
#[allow(dead_code)] // Will be used in workspace.rs
pub fn current_theme(cx: &App) -> Arc<Theme> {
    cx.theme().clone()
}

/// Get the theme registry for accessing available themes
///
/// The `ThemeRegistry` provides access to all loaded themes and allows
/// querying theme metadata and switching themes.
#[allow(dead_code)] // Available for future theme switching
pub fn theme_registry(cx: &App) -> Arc<ThemeRegistry> {
    ThemeRegistry::global(cx)
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_module_compiles() {
        // Basic compilation test - GPUI tests require TestAppContext
        // which needs special setup, so we keep this simple
        // The init function is tested via integration tests that require App context
    }
}
