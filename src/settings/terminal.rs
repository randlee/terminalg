//! Terminal-specific settings

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    /// Font size in points
    pub font_size: u32,

    /// Font family name
    pub font_family: String,

    /// Terminal padding
    pub padding: Padding,

    /// Enable URL recognition and clickable links
    pub enable_url_recognition: bool,

    /// Number of scrollback lines to keep
    pub scrollback_lines: usize,

    /// Shell to launch (e.g., "/bin/zsh", "pwsh")
    #[serde(default)]
    pub shell: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Padding {
    pub horizontal: u32,
    pub vertical: u32,
}

impl Default for TerminalSettings {
    fn default() -> Self {
        Self {
            font_size: 12,
            font_family: "Menlo".to_string(), // macOS default
            padding: Padding {
                horizontal: 8,
                vertical: 8,
            },
            enable_url_recognition: true,
            scrollback_lines: 10000,
            shell: None,
        }
    }
}
