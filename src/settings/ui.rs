//! UI-specific settings

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// Theme name (e.g., "dark", "light", "nord")
    pub theme: String,

    /// Terminal pane width ratio (0.0-1.0)
    pub terminal_width_ratio: f32,

    /// Show preview/viewer pane
    pub show_preview: bool,

    /// Auto-save interval in seconds (0 = disabled)
    pub autosave_interval: u32,
}

impl Default for UiSettings {
    fn default() -> Self {
        Self {
            theme: "dark".to_string(),
            terminal_width_ratio: 0.5,
            show_preview: true,
            autosave_interval: 0,
        }
    }
}
