//! File browser entry rendering utilities
//!
//! This module provides utilities for rendering file and folder entries
//! with icons, git status indicators, and proper indentation.

use gpui::{div, prelude::*, IntoElement, Styled};
use std::path::PathBuf;
use theme::ActiveTheme;

use crate::file_browser::state::ProjectEntryId;

/// Details for rendering an entry
#[derive(Clone, Debug)]
#[allow(clippy::struct_excessive_bools)] // These bools represent distinct states
pub struct EntryDetails {
    /// Entry ID
    pub id: ProjectEntryId,
    /// Display filename
    pub filename: String,
    /// Entry depth for indentation
    pub depth: usize,
    /// Whether this is a directory
    pub is_dir: bool,
    /// Whether this directory is expanded
    pub is_expanded: bool,
    /// Whether this entry is selected
    pub is_selected: bool,
    /// Whether this entry is marked (multi-select)
    pub is_marked: bool,
    /// Whether this entry is being edited
    pub is_editing: bool,
    /// Full path for this entry
    pub path: PathBuf,
}

impl EntryDetails {
    /// Create entry details from path
    #[must_use]
    pub fn from_path(id: ProjectEntryId, path: PathBuf, is_dir: bool, depth: usize) -> Self {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_string();

        Self {
            id,
            filename,
            depth,
            is_dir,
            is_expanded: false,
            is_selected: false,
            is_marked: false,
            is_editing: false,
            path,
        }
    }
}

/// Render a single file/folder entry
///
/// # Arguments
/// * `details` - Entry rendering details
/// * `cx` - GPUI context
///
/// # Returns
/// Element representing the entry row
#[allow(clippy::needless_pass_by_value)] // details is consumed
#[allow(clippy::needless_pass_by_ref_mut)] // cx will be used for event handlers
pub fn render_entry<V: 'static>(details: EntryDetails, cx: &mut gpui::Context<V>) -> impl IntoElement {
    let theme = cx.theme();
    #[allow(clippy::cast_precision_loss)] // depth will never exceed f32 precision
    let indent_width = details.depth as f32 * 16.0;

    div()
        .h(gpui::px(24.0))
        .w_full()
        .flex()
        .items_center()
        .gap_1()
        .px_2()
        .when(details.is_selected, |d| {
            d.bg(theme.colors().element_selected)
        })
        .when(details.is_marked && !details.is_selected, |d| {
            d.bg(theme.colors().element_hover)
        })
        // Indentation
        .child(div().w(gpui::px(indent_width)))
        // Expand/collapse indicator for directories
        .child(
            div()
                .w(gpui::px(16.0))
                .flex()
                .items_center()
                .justify_center()
                .when(details.is_dir, |d| {
                    d.child(if details.is_expanded { "v" } else { ">" })
                }),
        )
        // Filename
        .child(
            div()
                .flex_1()
                .text_sm()
                .text_color(theme.colors().text)
                .child(details.filename),
        )
}

/// Get git status color for an entry
///
/// # Arguments
/// * `status` - Git status string (e.g., "M", "A", "D", "?")
/// * `cx` - GPUI context
///
/// # Returns
/// Color for the git status
#[allow(dead_code)]
pub fn git_status_color<V: 'static>(status: &str, cx: &gpui::Context<V>) -> gpui::Hsla {
    let theme = cx.theme();
    let status_colors = theme.status();

    match status {
        "A" | "?" => status_colors.created,   // Added/Untracked - green
        "M" => status_colors.modified,        // Modified - yellow
        "D" => status_colors.deleted,         // Deleted - red
        "C" => status_colors.conflict,        // Conflict - purple
        _ => theme.colors().text,             // Default
    }
}

/// Get git status indicator character
///
/// # Arguments
/// * `status` - Git status string
///
/// # Returns
/// Single character indicator
#[must_use]
pub fn git_status_char(status: &str) -> &str {
    match status {
        "A" => "+",
        "M" => "~",
        "D" => "-",
        "?" => "?",
        "C" => "!",
        _ => "",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn entry_details_from_path() {
        let path = PathBuf::from("src/main.rs");
        let details = EntryDetails::from_path(1, path, false, 1);

        assert_eq!(details.id, 1);
        assert_eq!(details.filename, "main.rs");
        assert_eq!(details.depth, 1);
        assert!(!details.is_dir);
        assert!(!details.is_expanded);
        assert!(!details.is_selected);
    }

    #[test]
    fn entry_details_from_path_directory() {
        let path = PathBuf::from("src/file_browser");
        let details = EntryDetails::from_path(2, path, true, 1);

        assert_eq!(details.filename, "file_browser");
        assert!(details.is_dir);
    }

    #[test]
    fn git_status_char_mappings() {
        assert_eq!(git_status_char("A"), "+");
        assert_eq!(git_status_char("M"), "~");
        assert_eq!(git_status_char("D"), "-");
        assert_eq!(git_status_char("?"), "?");
        assert_eq!(git_status_char("C"), "!");
        assert_eq!(git_status_char("X"), "");
    }
}
