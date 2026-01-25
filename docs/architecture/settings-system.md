# Settings System Architecture (Phase 1 - Custom Implementation)

> **Status:** Phase 1 Implementation Only
>
> **Migration Plan:** Phase 2 will adopt Zed's `settings`, `settings_json`, and `settings_macros` crates as git dependencies. This is required because Zed's `terminal` crate depends on Zed's settings system - we cannot substitute our own.
>
> **See:** `docs/architecture/zed-reuse-strategy.md` Section 3.2 for migration details.

---

## Overview

This document describes TerminalG's **Phase 1 custom settings implementation**. This approach provides a working settings system for Sprints 1.1-1.5 while we evaluate Zed crate adoption.

**Phase 1 (Current):** Custom `SettingsStore` with JSON serialization
**Phase 2 (Future):** Migrate to Zed's settings crates for terminal integration compatibility

---

## Project Structure

```
terminalg/
├── src/
│   ├── main.rs
│   ├── settings/
│   │   ├── mod.rs          # Settings struct + SettingsStore
│   │   ├── terminal.rs     # Terminal settings
│   │   └── ui.rs           # UI settings
│   └── ...
├── Cargo.toml
└── .terminalg/
    └── settings.json       # Workspace settings (repo-local)
```

## Settings Locations

- **User settings:** `~/.config/terminalg/settings.json`
- **Workspace settings:** `.terminalg/workspace.json` or `.terminalg/workspace-<name>.json` (in the project root; user decides whether to commit)

## Cargo.toml Dependencies

```toml
[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
anyhow = "1.0"
tracing = "0.1"
dirs = "5.0"  # For config directory
notify = "6.0"  # Optional: file watching for live reload
```

## Implementation

### 1. Terminal Settings (settings/terminal.rs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalSettings {
    /// Font size in points
    pub font_size: u32,

    /// Font family name
    pub font_family: String,

    /// Terminal padding
    pub padding: Padding,

    /// Enable URL recognition
    pub enable_url_recognition: bool,

    /// Scrollback lines
    pub scrollback_lines: usize,

    /// Shell to launch (e.g., "/bin/zsh")
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
            font_family: "Mono".to_string(),
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
```

### 2. UI Settings (settings/ui.rs)

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiSettings {
    /// Theme name
    pub theme: String,

    /// Terminal pane width ratio (0.0-1.0)
    pub terminal_width_ratio: f32,

    /// Show markdown preview
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
```

### 3. Main Settings Struct (settings/mod.rs)

```rust
pub mod terminal;
pub mod ui;

use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use anyhow::{Result, Context};

pub use terminal::TerminalSettings;
pub use ui::UiSettings;

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
            Settings::default()
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

        // Create config dir if it doesn't exist
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

    /// Save settings to file
    pub fn save(&self) -> Result<()> {
        let content = serde_json::to_string_pretty(&self.settings)
            .context("Failed to serialize settings")?;

        fs::write(&self.settings_path, content)
            .context(format!("Failed to write settings to {:?}", self.settings_path))?;

        tracing::info!("Settings saved to {:?}", self.settings_path);
        Ok(())
    }

    /// Get current settings (immutable reference)
    pub fn settings(&self) -> &Settings {
        &self.settings
    }

    /// Get mutable reference (call save() after modifications)
    pub fn settings_mut(&mut self) -> &mut Settings {
        &mut self.settings
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
```

### 4. Usage in main.rs

```rust
mod settings;

use settings::SettingsStore;
use anyhow::Result;

fn main() -> Result<()> {
    // Initialize settings
    let mut store = SettingsStore::new()?;

    // Access settings
    let terminal_settings = &store.settings().terminal;
    println!("Font size: {}", terminal_settings.font_size);
    println!("Settings file: {:?}", store.settings_path());

    // Modify settings
    store.settings_mut().terminal.font_size = 14;
    store.save()?;

    Ok(())
}
```

## Generated settings.json

First run creates this (in the user config dir at `~/.config/terminalg/settings.json`):

```json
{
  "terminal": {
    "font_size": 12,
    "font_family": "Mono",
    "padding": {
      "horizontal": 8,
      "vertical": 8
    },
    "enable_url_recognition": true,
    "scrollback_lines": 10000,
    "shell": null
  },
  "ui": {
    "theme": "dark",
    "terminal_width_ratio": 0.5,
    "show_preview": true,
    "autosave_interval": 0
  }
}
```

User can edit directly, app reloads on next launch (or add notify watcher for live reload). Workspace overrides live in `.terminalg/workspace.json` or `.terminalg/workspace-<name>.json` and are loaded per project. The workspace root is defined by the folder containing `.terminalg/`, and loading/switching workspaces sets the process working directory to that root. App settings track known workspace paths for the current machine.

## Key Features

✓ Typed settings (serde handles validation)
✓ Defaults for all values
✓ Platform-aware config directory
✓ JSON format (human-readable)
✓ Modular structure (terminal/ui separate)
✓ Load/save/reload methods
✓ Easy to extend (add more modules)
✓ Mirrors Zed's mental model without Zed dependencies

## Extending

To add a new setting group:

1. Create `settings/myfeature.rs` with struct + Default impl
2. Add to Settings struct in `settings/mod.rs`
3. Done! Automatically serialized/deserialized

## Optional: File Watching (live reload)

Add notify dependency and watch for changes:

```rust
use notify::{Watcher, RecursiveMode, watcher};
use std::sync::mpsc::channel;
use std::time::Duration;

pub fn watch_settings(&self) -> Result<()> {
    let (tx, rx) = channel();
    let mut watcher = watcher(tx, Duration::from_secs(1))?;
    watcher.watch(&self.settings_path, RecursiveMode::NonRecursive)?;

    // In event loop: if rx.recv() => settings_store.reload()?
    Ok(())
}
```

This gives you hot-reload of settings without restarting the app.
