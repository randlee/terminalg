//! Workspace configuration system
//!
//! Provides typed workspace configuration management with JSON serialization.
//! Workspace configs are stored in `.terminalg/workspace.json` relative to the
//! workspace root (git repo root if available, otherwise current directory).

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for a single workspace
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceConfig {
    /// Unique workspace identifier
    pub id: String,

    /// Human-readable workspace name
    pub name: String,

    /// Whether file browser pane is visible
    pub file_browser_visible: bool,

    /// Whether terminal pane is visible
    pub terminal_visible: bool,

    /// Whether document viewer pane is visible
    pub document_viewer_visible: bool,

    /// Pane width ratios (`file_browser`, `terminal`, `document_viewer`)
    /// Values are relative (e.g., [1.0, 2.0, 1.0] = 25%, 50%, 25%)
    pub pane_ratios: [f32; 3],
}

impl Default for WorkspaceConfig {
    fn default() -> Self {
        Self {
            id: "default".to_string(),
            name: "Default".to_string(),
            file_browser_visible: true,
            terminal_visible: true,
            document_viewer_visible: false, // Per spec: "file browser + terminal visible"
            pane_ratios: [1.0, 2.0, 1.0],   // 25%, 50%, 25%
        }
    }
}

/// Collection of all workspaces
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspacesConfig {
    /// Index of active workspace
    pub active_workspace_index: usize,

    /// List of workspace configurations
    pub workspaces: Vec<WorkspaceConfig>,
}

impl Default for WorkspacesConfig {
    fn default() -> Self {
        Self {
            active_workspace_index: 0,
            workspaces: vec![
                WorkspaceConfig::default(),
                WorkspaceConfig {
                    id: "debug".to_string(),
                    name: "Debug".to_string(),
                    file_browser_visible: false,
                    terminal_visible: true,
                    document_viewer_visible: true,
                    pane_ratios: [1.0, 2.0, 1.0],
                },
            ],
        }
    }
}

/// Workspace configuration store manages loading, saving, and reloading workspace config
pub struct WorkspaceConfigStore {
    config: WorkspacesConfig,
    config_path: PathBuf,
}

impl WorkspaceConfigStore {
    /// Create new workspace config store, loading from disk if it exists
    pub fn new() -> Result<Self> {
        let config_path = Self::get_config_path()?;

        let mut config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            let defaults = WorkspacesConfig::default();
            Self::save_to_file(&defaults, &config_path)?;
            defaults
        };

        // Validate loaded config
        Self::validate_config(&mut config);

        Ok(Self {
            config,
            config_path,
        })
    }

    /// Create new workspace config store with a custom config path (for testing)
    pub fn new_with_path(config_path: PathBuf) -> Result<Self> {
        let mut config = if config_path.exists() {
            Self::load_from_file(&config_path)?
        } else {
            let defaults = WorkspacesConfig::default();
            Self::save_to_file(&defaults, &config_path)?;
            defaults
        };

        // Validate loaded config
        Self::validate_config(&mut config);

        Ok(Self {
            config,
            config_path,
        })
    }

    /// Validate and fix config invariants after loading
    fn validate_config(config: &mut WorkspacesConfig) {
        // Ensure at least one workspace exists
        if config.workspaces.is_empty() {
            tracing::warn!("No workspaces found in config, adding default workspace");
            config.workspaces.push(WorkspaceConfig::default());
        }

        // Ensure active_workspace_index is within bounds
        if config.active_workspace_index >= config.workspaces.len() {
            tracing::warn!(
                "Invalid active_workspace_index {} (max {}), resetting to 0",
                config.active_workspace_index,
                config.workspaces.len() - 1
            );
            config.active_workspace_index = 0;
        }
    }

    /// Get workspace config path (.terminalg/workspace.json)
    ///
    /// Searches for git repo root first (walking up directories), falls back to
    /// current working directory if not in a git repository.
    fn get_config_path() -> Result<PathBuf> {
        let current_dir = std::env::current_dir().context("Failed to get current directory")?;

        // Try to find git repo root, fall back to current directory
        let workspace_root = Self::find_repo_root(&current_dir).unwrap_or_else(|| {
            tracing::debug!(
                "Not in a git repository, using current directory for workspace config"
            );
            current_dir
        });

        // Create .terminalg directory if it doesn't exist
        let config_dir = workspace_root.join(".terminalg");
        fs::create_dir_all(&config_dir).context("Failed to create .terminalg directory")?;

        Ok(config_dir.join("workspace.json"))
    }

    /// Find git repository root by walking up directories
    fn find_repo_root(start_dir: &Path) -> Option<PathBuf> {
        let mut current = start_dir.to_path_buf();
        loop {
            if current.join(".git").exists() {
                return Some(current);
            }
            if !current.pop() {
                return None;
            }
        }
    }

    /// Load workspace config from file
    fn load_from_file(path: &Path) -> Result<WorkspacesConfig> {
        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read workspace config from {}", path.display()))?;
        let config: WorkspacesConfig =
            serde_json::from_str(&content).context("Failed to parse workspace.json")?;
        Ok(config)
    }

    /// Save workspace config to file
    fn save_to_file(config: &WorkspacesConfig, path: &Path) -> Result<()> {
        let content =
            serde_json::to_string_pretty(config).context("Failed to serialize workspace config")?;
        fs::write(path, content)
            .with_context(|| format!("Failed to write workspace config to {}", path.display()))?;
        tracing::debug!("Workspace config saved to {}", path.display());
        Ok(())
    }

    /// Get current config (immutable reference)
    pub const fn config(&self) -> &WorkspacesConfig {
        &self.config
    }

    /// Get mutable reference to config (call `save()` after modifications)
    #[allow(dead_code)] // Used in tests
    #[allow(clippy::missing_const_for_fn)] // const mutable ref not stable
    pub fn config_mut(&mut self) -> &mut WorkspacesConfig {
        &mut self.config
    }

    /// Get active workspace (with bounds checking)
    pub fn active_workspace(&self) -> &WorkspaceConfig {
        self.config
            .workspaces
            .get(self.config.active_workspace_index)
            .unwrap_or_else(|| {
                // This should never happen after validate_config, but be defensive
                &self.config.workspaces[0]
            })
    }

    /// Get mutable reference to active workspace (with bounds checking)
    pub fn active_workspace_mut(&mut self) -> &mut WorkspaceConfig {
        // Fix index if out of bounds
        if self.config.active_workspace_index >= self.config.workspaces.len() {
            self.config.active_workspace_index = 0;
        }
        &mut self.config.workspaces[self.config.active_workspace_index]
    }

    /// Switch to workspace at index
    #[allow(clippy::missing_const_for_fn)] // Vec::len() not const
    pub fn switch_workspace(&mut self, index: usize) {
        if index < self.config.workspaces.len() {
            self.config.active_workspace_index = index;
        }
    }

    /// Save config to disk
    pub fn save(&self) -> Result<()> {
        Self::save_to_file(&self.config, &self.config_path)
    }

    /// Get config path for debugging
    pub const fn config_path(&self) -> &PathBuf {
        &self.config_path
    }
}

impl Clone for WorkspaceConfigStore {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            config_path: self.config_path.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_workspace_config_defaults() {
        let config = WorkspaceConfig::default();
        assert_eq!(config.id, "default");
        assert!(config.file_browser_visible);
        assert!(config.terminal_visible);
        assert!(!config.document_viewer_visible);
    }

    #[test]
    fn test_workspaces_config_defaults() {
        let config = WorkspacesConfig::default();
        assert_eq!(config.active_workspace_index, 0);
        assert_eq!(config.workspaces.len(), 2);
    }

    #[test]
    fn test_workspace_config_serialization() {
        let config = WorkspaceConfig::default();
        let json = serde_json::to_string(&config).unwrap();
        let deserialized: WorkspaceConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.id, deserialized.id);
    }

    #[test]
    fn test_workspace_config_store_save_load() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspace.json");

        // Create and save
        let mut store = WorkspaceConfigStore::new_with_path(config_path.clone()).unwrap();
        store.config_mut().active_workspace_index = 1;
        store.save().unwrap();

        // Load and verify
        let loaded = WorkspaceConfigStore::new_with_path(config_path).unwrap();
        assert_eq!(loaded.config().active_workspace_index, 1);
    }

    #[test]
    fn test_workspace_switch() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspace.json");
        let mut store = WorkspaceConfigStore::new_with_path(config_path).unwrap();

        assert_eq!(store.config().active_workspace_index, 0);
        store.switch_workspace(1);
        assert_eq!(store.config().active_workspace_index, 1);
    }

    #[test]
    fn test_active_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspace.json");
        let store = WorkspaceConfigStore::new_with_path(config_path).unwrap();

        let active = store.active_workspace();
        assert_eq!(active.id, "default");
    }

    #[test]
    fn test_active_workspace_mut() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspace.json");
        let mut store = WorkspaceConfigStore::new_with_path(config_path).unwrap();

        let active = store.active_workspace_mut();
        active.name = "Modified".to_string();

        assert_eq!(store.active_workspace().name, "Modified");
    }

    #[test]
    fn test_workspace_config_store_creates_file() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspace.json");

        let _store = WorkspaceConfigStore::new_with_path(config_path.clone()).unwrap();

        assert!(config_path.exists());
    }

    #[test]
    fn test_validate_config_fixes_invalid_index() {
        let mut config = WorkspacesConfig {
            active_workspace_index: 99, // Invalid - way out of bounds
            workspaces: vec![WorkspaceConfig::default()],
        };

        WorkspaceConfigStore::validate_config(&mut config);

        assert_eq!(config.active_workspace_index, 0);
    }

    #[test]
    fn test_validate_config_adds_default_workspace_if_empty() {
        let mut config = WorkspacesConfig {
            active_workspace_index: 0,
            workspaces: vec![], // Empty!
        };

        WorkspaceConfigStore::validate_config(&mut config);

        assert_eq!(config.workspaces.len(), 1);
        assert_eq!(config.workspaces[0].id, "default");
    }

    #[test]
    fn test_active_workspace_handles_corrupted_index() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspace.json");

        // Create store and manually corrupt the index
        let mut store = WorkspaceConfigStore::new_with_path(config_path).unwrap();
        store.config.active_workspace_index = 999; // Corrupt!

        // Should not panic, should return first workspace
        let active = store.active_workspace();
        assert_eq!(active.id, "default");
    }

    #[test]
    fn test_active_workspace_mut_fixes_corrupted_index() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("workspace.json");

        // Create store and manually corrupt the index
        let mut store = WorkspaceConfigStore::new_with_path(config_path).unwrap();
        store.config.active_workspace_index = 999; // Corrupt!

        // Should not panic, should fix index and return first workspace
        let active = store.active_workspace_mut();
        assert_eq!(active.id, "default");
        assert_eq!(store.config.active_workspace_index, 0); // Index was fixed
    }
}
