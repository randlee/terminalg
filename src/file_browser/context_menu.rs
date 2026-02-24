//! File browser context menu
//!
//! This module provides context menu building for file browser operations.

use crate::file_browser::state::ProjectEntryId;

/// Context menu item for file operations
#[derive(Clone, Debug)]
pub struct ContextMenuItem {
    /// Display label
    pub label: String,
    /// Optional keyboard shortcut hint
    pub shortcut: Option<String>,
    /// Whether this item is enabled
    pub enabled: bool,
}

impl ContextMenuItem {
    /// Create a new context menu item
    #[must_use]
    pub fn new(label: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            shortcut: None,
            enabled: true,
        }
    }

    /// Add a keyboard shortcut hint
    #[must_use]
    pub fn shortcut(mut self, shortcut: impl Into<String>) -> Self {
        self.shortcut = Some(shortcut.into());
        self
    }

    /// Set whether this item is enabled
    #[must_use]
    pub const fn enabled(mut self, enabled: bool) -> Self {
        self.enabled = enabled;
        self
    }
}

/// Build context menu items for a file/folder entry
///
/// # Arguments
/// * `entry_id` - ID of the entry
/// * `is_dir` - Whether the entry is a directory
/// * `has_clipboard` - Whether there's content in the clipboard
///
/// # Returns
/// Vector of context menu items
#[must_use]
pub fn build_context_menu_items(
    _entry_id: ProjectEntryId,
    is_dir: bool,
    has_clipboard: bool,
) -> Vec<ContextMenuItem> {
    let mut items = vec![
        ContextMenuItem::new("New File").shortcut("N"),
        ContextMenuItem::new("New Folder").shortcut("Shift+N"),
    ];

    // Separator represented by empty label
    items.push(ContextMenuItem::new("---"));

    items.extend([
        ContextMenuItem::new("Rename").shortcut("F2"),
        ContextMenuItem::new("Delete").shortcut("Delete"),
    ]);

    items.push(ContextMenuItem::new("---"));

    items.extend([
        ContextMenuItem::new("Cut").shortcut("Cmd+X"),
        ContextMenuItem::new("Copy").shortcut("Cmd+C"),
        ContextMenuItem::new("Paste")
            .shortcut("Cmd+V")
            .enabled(has_clipboard),
    ]);

    items.push(ContextMenuItem::new("---"));

    items.extend([
        ContextMenuItem::new("Copy Path"),
        ContextMenuItem::new("Copy Relative Path"),
    ]);

    items.push(ContextMenuItem::new("---"));

    items.push(ContextMenuItem::new("Reveal in Finder"));

    if is_dir {
        items.push(ContextMenuItem::new("Open in Terminal"));
    }

    items.push(ContextMenuItem::new("---"));
    items.push(ContextMenuItem::new("Collapse All"));

    items
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn context_menu_item_new() {
        let item = ContextMenuItem::new("Test");
        assert_eq!(item.label, "Test");
        assert!(item.shortcut.is_none());
        assert!(item.enabled);
    }

    #[test]
    fn context_menu_item_with_shortcut() {
        let item = ContextMenuItem::new("Test").shortcut("Cmd+T");
        assert_eq!(item.shortcut, Some("Cmd+T".to_string()));
    }

    #[test]
    fn context_menu_item_disabled() {
        let item = ContextMenuItem::new("Test").enabled(false);
        assert!(!item.enabled);
    }

    #[test]
    fn build_context_menu_for_file() {
        let items = build_context_menu_items(1, false, false);

        // Should not have "Open in Terminal"
        assert!(!items.iter().any(|i| i.label == "Open in Terminal"));

        // Paste should be disabled
        let paste = items.iter().find(|i| i.label == "Paste").unwrap();
        assert!(!paste.enabled);
    }

    #[test]
    fn build_context_menu_for_directory() {
        let items = build_context_menu_items(1, true, true);

        // Should have "Open in Terminal"
        assert!(items.iter().any(|i| i.label == "Open in Terminal"));

        // Paste should be enabled
        let paste = items.iter().find(|i| i.label == "Paste").unwrap();
        assert!(paste.enabled);
    }
}
