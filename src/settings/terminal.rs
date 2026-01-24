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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_terminal_settings_defaults_are_correct() {
        let settings = TerminalSettings::default();

        assert_eq!(settings.font_size, 12);
        assert_eq!(settings.font_family, "Menlo");
        assert_eq!(settings.padding.horizontal, 8);
        assert_eq!(settings.padding.vertical, 8);
        assert!(settings.enable_url_recognition);
        assert_eq!(settings.scrollback_lines, 10000);
        assert!(settings.shell.is_none());
    }

    #[test]
    fn test_terminal_settings_serialization() {
        let settings = TerminalSettings::default();

        // Serialize to JSON
        let json = serde_json::to_string(&settings)
            .expect("Failed to serialize TerminalSettings");

        // Verify JSON contains expected keys
        assert!(json.contains("font_size"));
        assert!(json.contains("font_family"));
        assert!(json.contains("padding"));
        assert!(json.contains("enable_url_recognition"));
        assert!(json.contains("scrollback_lines"));

        // Deserialize back
        let deserialized: TerminalSettings = serde_json::from_str(&json)
            .expect("Failed to deserialize TerminalSettings");

        // Verify round-trip
        assert_eq!(deserialized.font_size, settings.font_size);
        assert_eq!(deserialized.font_family, settings.font_family);
        assert_eq!(deserialized.padding.horizontal, settings.padding.horizontal);
        assert_eq!(deserialized.padding.vertical, settings.padding.vertical);
        assert_eq!(deserialized.enable_url_recognition, settings.enable_url_recognition);
        assert_eq!(deserialized.scrollback_lines, settings.scrollback_lines);
    }

    #[test]
    fn test_terminal_settings_deserialization() {
        let json = r#"{
            "font_size": 14,
            "font_family": "Monaco",
            "padding": {
                "horizontal": 10,
                "vertical": 12
            },
            "enable_url_recognition": false,
            "scrollback_lines": 5000,
            "shell": "/bin/bash"
        }"#;

        let settings: TerminalSettings = serde_json::from_str(json)
            .expect("Failed to deserialize JSON");

        assert_eq!(settings.font_size, 14);
        assert_eq!(settings.font_family, "Monaco");
        assert_eq!(settings.padding.horizontal, 10);
        assert_eq!(settings.padding.vertical, 12);
        assert!(!settings.enable_url_recognition);
        assert_eq!(settings.scrollback_lines, 5000);
        assert_eq!(settings.shell, Some("/bin/bash".to_string()));
    }

    #[test]
    fn test_terminal_settings_with_custom_values() {
        let settings = TerminalSettings {
            font_size: 16,
            font_family: "Consolas".to_string(),
            padding: Padding {
                horizontal: 12,
                vertical: 12,
            },
            enable_url_recognition: false,
            scrollback_lines: 20000,
            shell: Some("/bin/zsh".to_string()),
        };

        assert_eq!(settings.font_size, 16);
        assert_eq!(settings.font_family, "Consolas");
        assert_eq!(settings.padding.horizontal, 12);
        assert_eq!(settings.padding.vertical, 12);
        assert!(!settings.enable_url_recognition);
        assert_eq!(settings.scrollback_lines, 20000);
        assert_eq!(settings.shell, Some("/bin/zsh".to_string()));
    }

    #[test]
    fn test_padding_serialization() {
        let padding = Padding {
            horizontal: 5,
            vertical: 10,
        };

        let json = serde_json::to_string(&padding)
            .expect("Failed to serialize Padding");

        let deserialized: Padding = serde_json::from_str(&json)
            .expect("Failed to deserialize Padding");

        assert_eq!(deserialized.horizontal, padding.horizontal);
        assert_eq!(deserialized.vertical, padding.vertical);
    }

    #[test]
    fn test_terminal_settings_shell_default_handling() {
        // Test that shell field defaults to None when missing from JSON
        let json = r#"{
            "font_size": 12,
            "font_family": "Menlo",
            "padding": {
                "horizontal": 8,
                "vertical": 8
            },
            "enable_url_recognition": true,
            "scrollback_lines": 10000
        }"#;

        let settings: TerminalSettings = serde_json::from_str(json)
            .expect("Failed to deserialize JSON without shell field");

        assert!(settings.shell.is_none());
    }
}
