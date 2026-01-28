//! File browser components
//!
//! This module provides file system navigation and management capabilities
//! for `TerminalG`, supporting workspace-level file operations.

// Allow dead_code until integration is complete
#![allow(dead_code)]
// Allow PartialEq without Eq for generated action types
#![allow(clippy::derive_partial_eq_without_eq)]

mod context_menu;
mod pane;
mod render;
mod state;

#[allow(unused_imports)] // Will be used when integrated with workspace
pub use pane::{FileBrowserPane, FileBrowserPaneEvent};

use gpui::actions;

actions!(
    file_browser,
    [
        NewFile,
        NewDirectory,
        Rename,
        Delete,
        Copy,
        Cut,
        Paste,
        CopyPath,
        CopyRelativePath,
        RevealInFinder,
        OpenInTerminal,
        CollapseAll,
        ExpandSelectedEntry,
        CollapseSelectedEntry,
    ]
);
