//! Application settings system
//!
//! Provides typed settings management with JSON serialization.
//! Settings are stored in platform-specific config directories.

pub mod terminal;
pub mod ui;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

pub use terminal::TerminalSettings;
pub use ui::UiSettings;

/// Main Settings struct combining all setting groups
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Settings {
    #[serde(default)]
    pub terminal: TerminalSettings,
    #[serde(default)]
    pub ui: UiSettings,
}

impl Settings {
    /// Save settings to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content =
            serde_json::to_string_pretty(&self).context("Failed to serialize settings")?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write settings to {}", path.display()))?;
        tracing::info!("Settings saved to {}", path.display());
        Ok(())
    }
}

/// Settings store manages loading, saving, and reloading configuration
pub struct SettingsStore {
    settings: Settings,
    settings_path: PathBuf,
}

impl SettingsStore {
    /// Create new settings store, loading from disk if it exists
    pub fn new() -> Result<Self> {
        let settings_path = Self::get_settings_path()?;

        let settings = if settings_path.exists() {
            Self::load_from_file(&settings_path)?
        } else {
            let defaults = Settings::default();
            defaults.save(&settings_path)?;
            defaults
        };

        Ok(Self {
            settings,
            settings_path,
        })
    }

    /// Get config directory (platform-specific)
    fn get_config_dir() -> Result<PathBuf> {
        dirs::config_dir()
            .ok_or_else(|| anyhow::anyhow!("Could not determine config directory"))
            .map(|p| p.join("terminalg"))
    }

    /// Get full settings file path
    fn get_settings_path() -> Result<PathBuf> {
        let config_dir = Self::get_config_dir()?;
        fs::create_dir_all(&config_dir).context("Failed to create config directory")?;
        Ok(config_dir.join("settings.json"))
    }

    /// Load settings from file
    fn load_from_file(path: &PathBuf) -> Result<Settings> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read settings from {}", path.display()))?;
        let settings: Settings =
            serde_json::from_str(&content).context("Failed to parse settings.json")?;
        Ok(settings)
    }

    /// Get current settings (immutable reference)
    pub const fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get mutable reference (call `save()` after modifications)
    #[allow(dead_code)] // Part of public API, will be used in future phases
    pub const fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Save settings to file
    #[allow(dead_code)] // Part of public API, will be used in future phases
    pub fn save(&self) -> Result<()> {
        let content =
            serde_json::to_string_pretty(&self.settings).context("Failed to serialize settings")?;
        fs::write(&self.settings_path, content).with_context(|| {
            format!(
                "Failed to write settings to {}",
                self.settings_path.display()
            )
        })?;
        tracing::info!("Settings saved to {}", self.settings_path.display());
        Ok(())
    }

    /// Reload settings from disk
    #[allow(dead_code)] // Part of public API, will be used in future phases
    pub fn reload(&mut self) -> Result<()> {
        self.settings = Self::load_from_file(&self.settings_path)?;
        tracing::info!("Settings reloaded from disk");
        Ok(())
    }

    /// Get settings path for debugging
    pub const fn settings_path(&self) -> &PathBuf {
        &self.settings_path
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    /// Helper function to create a test config directory
    fn setup_test_config_dir() -> TempDir {
        tempfile::tempdir().expect("Failed to create temp dir")
    }

    #[test]
    fn test_settings_default_values() {
        let settings = Settings::default();

        // Verify terminal defaults
        assert_eq!(settings.terminal.font_size, 12);
        assert_eq!(settings.terminal.font_family, "Menlo");
        assert_eq!(settings.terminal.padding.horizontal, 8);
        assert_eq!(settings.terminal.padding.vertical, 8);
        assert!(settings.terminal.enable_url_recognition);
        assert_eq!(settings.terminal.scrollback_lines, 10000);
        assert!(settings.terminal.shell.is_none());

        // Verify UI defaults
        assert_eq!(settings.ui.theme, "dark");
        assert!((settings.ui.terminal_width_ratio - 0.5).abs() < f32::EPSILON);
        assert!(settings.ui.show_preview);
        assert_eq!(settings.ui.autosave_interval, 0);
    }

    #[test]
    fn test_settings_save_and_load() {
        let temp_dir = setup_test_config_dir();
        let settings_path = temp_dir.path().join("settings.json");

        // Create and save settings
        let settings = Settings::default();
        settings
            .save(&settings_path)
            .expect("Failed to save settings");

        // Verify file exists
        assert!(settings_path.exists());

        // Load settings back
        let loaded =
            SettingsStore::load_from_file(&settings_path).expect("Failed to load settings");

        // Verify loaded settings match defaults
        assert_eq!(loaded.terminal.font_size, settings.terminal.font_size);
        assert_eq!(loaded.ui.theme, settings.ui.theme);
    }

    #[test]
    fn test_settings_serialization() {
        let settings = Settings::default();

        // Serialize to JSON
        let json = serde_json::to_string(&settings).expect("Failed to serialize settings");

        // Deserialize back
        let deserialized: Settings =
            serde_json::from_str(&json).expect("Failed to deserialize settings");

        // Verify round-trip
        assert_eq!(deserialized.terminal.font_size, settings.terminal.font_size);
        assert_eq!(deserialized.ui.theme, settings.ui.theme);
    }

    #[test]
    fn test_settings_store_creates_default_settings() {
        // Note: This test uses the real config directory
        // It will create settings if they don't exist
        let store = SettingsStore::new().expect("Failed to create settings store");

        // Verify settings path exists
        assert!(store.settings_path().exists());

        // Verify default values
        assert_eq!(store.settings().terminal.font_size, 12);
        assert_eq!(store.settings().ui.theme, "dark");
    }

    #[test]
    fn test_settings_store_save_and_reload() {
        // Create a new store (uses real config directory)
        let mut store = SettingsStore::new().expect("Failed to create settings store");

        // Modify settings
        let original_font_size = store.settings().terminal.font_size;
        store.settings_mut().terminal.font_size = 16;

        // Save to disk
        store.save().expect("Failed to save settings");

        // Reload from disk
        store.reload().expect("Failed to reload settings");

        // Verify the change persisted
        assert_eq!(store.settings().terminal.font_size, 16);

        // Restore original for other tests
        store.settings_mut().terminal.font_size = original_font_size;
        store.save().expect("Failed to restore settings");
    }

    #[test]
    fn test_settings_store_get_config_dir_returns_path() {
        let config_dir = SettingsStore::get_config_dir().expect("Failed to get config directory");

        // Verify path ends with "terminalg"
        assert!(config_dir.to_string_lossy().ends_with("terminalg"));
    }

    #[test]
    fn test_settings_store_get_settings_path_creates_dir() {
        let settings_path =
            SettingsStore::get_settings_path().expect("Failed to get settings path");

        // Verify path exists and points to settings.json
        assert!(settings_path.parent().unwrap().exists());
        assert!(settings_path.to_string_lossy().ends_with("settings.json"));
    }

    #[test]
    fn test_settings_store_load_from_nonexistent_file_fails() {
        let temp_dir = setup_test_config_dir();
        let nonexistent_path = temp_dir.path().join("nonexistent.json");

        let result = SettingsStore::load_from_file(&nonexistent_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_settings_store_load_from_invalid_json_fails() {
        let temp_dir = setup_test_config_dir();
        let invalid_path = temp_dir.path().join("invalid.json");

        // Write invalid JSON
        fs::write(&invalid_path, "{ invalid json }").expect("Failed to write file");

        let result = SettingsStore::load_from_file(&invalid_path);
        assert!(result.is_err());
    }

    #[test]
    fn test_settings_store_settings_immutable_access() {
        let store = SettingsStore::new().expect("Failed to create settings store");

        // Get immutable reference
        let settings = store.settings();

        // Verify we can read values (no modification)
        assert!(settings.terminal.font_size > 0);
        assert!(!settings.ui.theme.is_empty());
    }

    #[test]
    fn test_settings_store_settings_mutable_access() {
        let mut store = SettingsStore::new().expect("Failed to create settings store");

        // Save original font size
        let original_font_size = store.settings().terminal.font_size;

        // Get mutable reference
        let settings_mut = store.settings_mut();

        // Modify a value
        settings_mut.terminal.font_size = 20;

        // Verify the change
        assert_eq!(store.settings().terminal.font_size, 20);

        // Restore original
        store.settings_mut().terminal.font_size = original_font_size;
    }
}
