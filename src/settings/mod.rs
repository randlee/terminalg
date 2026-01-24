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
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub terminal: TerminalSettings,
    pub ui: UiSettings,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            terminal: TerminalSettings::default(),
            ui: UiSettings::default(),
        }
    }
}

impl Settings {
    /// Save settings to file
    pub fn save(&self, path: &PathBuf) -> Result<()> {
        let content = serde_json::to_string_pretty(&self)
            .context("Failed to serialize settings")?;
        fs::write(path, content)
            .context(format!("Failed to write settings to {:?}", path))?;
        tracing::info!("Settings saved to {:?}", path);
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
        fs::create_dir_all(&config_dir)
            .context("Failed to create config directory")?;
        Ok(config_dir.join("settings.json"))
    }

    /// Load settings from file
    fn load_from_file(path: &PathBuf) -> Result<Settings> {
        let content = fs::read_to_string(path)
            .context(format!("Failed to read settings from {:?}", path))?;
        let settings: Settings = serde_json::from_str(&content)
            .context("Failed to parse settings.json")?;
        Ok(settings)
    }

    /// Get current settings (immutable reference)
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get mutable reference (call save() after modifications)
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
    }

    /// Save settings to file
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.settings)
            .context("Failed to serialize settings")?;
        fs::write(&self.settings_path, content)
            .context(format!("Failed to write settings to {:?}", self.settings_path))?;
        tracing::info!("Settings saved to {:?}", self.settings_path);
        Ok(())
    }

    /// Reload settings from disk
    pub fn reload(&mut self) -> Result<()> {
        self.settings = Self::load_from_file(&self.settings_path)?;
        tracing::info!("Settings reloaded from disk");
        Ok(())
    }

    /// Get settings path for debugging
    pub fn settings_path(&self) -> &PathBuf {
        &self.settings_path
    }
}
