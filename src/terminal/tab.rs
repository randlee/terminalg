//! Terminal tab state management
//!
//! Each `TerminalTab` represents a single terminal session within the terminal pane.

use gpui::{Context, Entity};
use std::path::PathBuf;
use terminal::Terminal;

/// A terminal tab representing a single terminal session
pub struct TerminalTab {
    /// The underlying Zed terminal
    pub terminal: Entity<Terminal>,
    /// Working directory for this terminal
    working_directory: Option<PathBuf>,
    /// Custom title (if set by user)
    custom_title: Option<String>,
}

impl TerminalTab {
    /// Create a new terminal tab
    #[allow(clippy::missing_const_for_fn)] // Cannot be const due to generic lifetime bounds
    pub fn new<V: 'static>(
        terminal: Entity<Terminal>,
        working_directory: Option<PathBuf>,
        _cx: &mut Context<V>,
    ) -> Self {
        Self {
            terminal,
            working_directory,
            custom_title: None,
        }
    }

    /// Get the display title for this tab
    pub fn title(&self) -> String {
        if let Some(ref title) = self.custom_title {
            return title.clone();
        }

        // Use working directory name or default
        if let Some(ref dir) = self.working_directory {
            if let Some(name) = dir.file_name() {
                return name.to_string_lossy().to_string();
            }
        }

        "Terminal".to_string()
    }

    /// Set a custom title for this tab
    #[allow(dead_code)]
    pub fn set_title(&mut self, title: impl Into<String>) {
        self.custom_title = Some(title.into());
    }

    /// Get the working directory
    #[allow(dead_code)]
    pub const fn working_directory(&self) -> Option<&PathBuf> {
        self.working_directory.as_ref()
    }

    /// Update the working directory
    #[allow(dead_code)]
    pub fn set_working_directory(&mut self, dir: Option<PathBuf>) {
        self.working_directory = dir;
    }
}
