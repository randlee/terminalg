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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ui_settings_defaults_are_correct() {
        let settings = UiSettings::default();

        assert_eq!(settings.theme, "dark");
        assert_eq!(settings.terminal_width_ratio, 0.5);
        assert!(settings.show_preview);
        assert_eq!(settings.autosave_interval, 0);
    }

    #[test]
    fn test_ui_settings_serialization() {
        let settings = UiSettings::default();

        // Serialize to JSON
        let json = serde_json::to_string(&settings)
            .expect("Failed to serialize UiSettings");

        // Verify JSON contains expected keys
        assert!(json.contains("theme"));
        assert!(json.contains("terminal_width_ratio"));
        assert!(json.contains("show_preview"));
        assert!(json.contains("autosave_interval"));

        // Deserialize back
        let deserialized: UiSettings = serde_json::from_str(&json)
            .expect("Failed to deserialize UiSettings");

        // Verify round-trip
        assert_eq!(deserialized.theme, settings.theme);
        assert_eq!(deserialized.terminal_width_ratio, settings.terminal_width_ratio);
        assert_eq!(deserialized.show_preview, settings.show_preview);
        assert_eq!(deserialized.autosave_interval, settings.autosave_interval);
    }

    #[test]
    fn test_ui_settings_deserialization() {
        let json = r#"{
            "theme": "light",
            "terminal_width_ratio": 0.6,
            "show_preview": false,
            "autosave_interval": 300
        }"#;

        let settings: UiSettings = serde_json::from_str(json)
            .expect("Failed to deserialize JSON");

        assert_eq!(settings.theme, "light");
        assert_eq!(settings.terminal_width_ratio, 0.6);
        assert!(!settings.show_preview);
        assert_eq!(settings.autosave_interval, 300);
    }

    #[test]
    fn test_ui_settings_with_custom_values() {
        let settings = UiSettings {
            theme: "nord".to_string(),
            terminal_width_ratio: 0.7,
            show_preview: false,
            autosave_interval: 60,
        };

        assert_eq!(settings.theme, "nord");
        assert_eq!(settings.terminal_width_ratio, 0.7);
        assert!(!settings.show_preview);
        assert_eq!(settings.autosave_interval, 60);
    }

    #[test]
    fn test_ui_settings_terminal_width_ratio_bounds() {
        // Test with various width ratios
        let settings_min = UiSettings {
            terminal_width_ratio: 0.0,
            ..Default::default()
        };
        assert_eq!(settings_min.terminal_width_ratio, 0.0);

        let settings_max = UiSettings {
            terminal_width_ratio: 1.0,
            ..Default::default()
        };
        assert_eq!(settings_max.terminal_width_ratio, 1.0);

        let settings_mid = UiSettings {
            terminal_width_ratio: 0.5,
            ..Default::default()
        };
        assert_eq!(settings_mid.terminal_width_ratio, 0.5);
    }

    #[test]
    fn test_ui_settings_autosave_disabled() {
        let settings = UiSettings::default();
        // Default should have autosave disabled (0 = disabled)
        assert_eq!(settings.autosave_interval, 0);
    }

    #[test]
    fn test_ui_settings_clone() {
        let settings = UiSettings::default();
        let cloned = settings.clone();

        assert_eq!(cloned.theme, settings.theme);
        assert_eq!(cloned.terminal_width_ratio, settings.terminal_width_ratio);
        assert_eq!(cloned.show_preview, settings.show_preview);
        assert_eq!(cloned.autosave_interval, settings.autosave_interval);
    }
}
