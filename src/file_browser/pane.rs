//! File browser pane component
//!
//! Main file browser component that orchestrates tree display, selection,
//! and file operations. This is Wave 2 of Sprint 3.1, implementing the core
//! `FileBrowserPane` with runtime state management and virtualized rendering.

use collections::HashMap;
use gpui::{
    div, prelude::*, px, uniform_list, App, Context, EventEmitter, FocusHandle, Focusable,
    IntoElement, KeyContext, KeyDownEvent, MouseButton, MouseDownEvent, Render, Styled,
    UniformListScrollHandle, Window,
};
use std::path::PathBuf;
use theme::ActiveTheme;

use crate::file_browser::render::{render_entry, EntryDetails};
use crate::file_browser::state::{collapse_dir, expand_dir, is_expanded, Entry, ProjectEntryId};

/// Events emitted by the file browser pane
#[derive(Clone, Debug)]
pub enum FileBrowserPaneEvent {
    /// Request to open a file
    OpenFile(PathBuf),
    /// Request to open a terminal in a directory
    OpenInTerminal(PathBuf),
    /// Selection changed
    SelectionChanged(Option<PathBuf>),
}

/// Per-workspace runtime state for the file browser
///
/// This state is not persisted and is rebuilt when switching workspaces.
/// Persisted state (expanded dirs, scroll position) is handled separately
/// via `WorkspaceConfig`.
#[derive(Clone, Debug, Default)]
struct FileBrowserRuntimeState {
    /// Flattened tree of visible entries
    visible_entries: Vec<Entry>,

    /// Sorted vector of expanded directory IDs for O(log n) lookups
    expanded_dir_ids: Vec<ProjectEntryId>,

    /// Currently selected entry ID
    selection: Option<ProjectEntryId>,

    /// Marked entries for multi-selection
    marked_entries: Vec<ProjectEntryId>,

    /// Scroll offset in pixels
    scroll_offset: f32,
}

/// File browser pane component
///
/// Displays a tree view of files and folders for the active workspace.
/// Supports expand/collapse, selection, keyboard navigation, and emits
/// events for file operations.
pub struct FileBrowserPane {
    /// Focus handle for keyboard input
    focus_handle: FocusHandle,

    /// Scroll handle for virtualized list
    scroll_handle: UniformListScrollHandle,

    /// Active workspace ID
    active_workspace_id: String,

    /// Per-workspace runtime state
    state_by_workspace: HashMap<String, FileBrowserRuntimeState>,
}

impl FileBrowserPane {
    /// Create a new file browser pane
    #[must_use]
    #[allow(clippy::needless_pass_by_ref_mut)] // cx will be used for subscriptions
    pub fn new(workspace_id: String, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let scroll_handle = UniformListScrollHandle::new();
        let mut state_by_workspace = HashMap::default();

        // Initialize state with placeholder entries for demonstration
        let initial_state = FileBrowserRuntimeState {
            visible_entries: Self::create_placeholder_tree(),
            ..Default::default()
        };

        state_by_workspace.insert(workspace_id.clone(), initial_state);

        Self {
            focus_handle,
            scroll_handle,
            active_workspace_id: workspace_id,
            state_by_workspace,
        }
    }

    /// Create placeholder tree for demonstration
    fn create_placeholder_tree() -> Vec<Entry> {
        vec![
            Entry {
                id: 1,
                path: PathBuf::from("src"),
                is_dir: true,
                depth: 0,
            },
            Entry {
                id: 2,
                path: PathBuf::from("src/main.rs"),
                is_dir: false,
                depth: 1,
            },
            Entry {
                id: 3,
                path: PathBuf::from("src/lib.rs"),
                is_dir: false,
                depth: 1,
            },
            Entry {
                id: 4,
                path: PathBuf::from("src/file_browser"),
                is_dir: true,
                depth: 1,
            },
            Entry {
                id: 5,
                path: PathBuf::from("Cargo.toml"),
                is_dir: false,
                depth: 0,
            },
            Entry {
                id: 6,
                path: PathBuf::from("README.md"),
                is_dir: false,
                depth: 0,
            },
        ]
    }

    /// Get the active runtime state
    fn get_active_state(&mut self) -> &mut FileBrowserRuntimeState {
        self.state_by_workspace
            .entry(self.active_workspace_id.clone())
            .or_insert_with(|| FileBrowserRuntimeState {
                visible_entries: Self::create_placeholder_tree(),
                ..Default::default()
            })
    }

    /// Switch to a different workspace
    pub fn set_active_workspace(&mut self, workspace_id: String, cx: &mut Context<Self>) {
        self.active_workspace_id = workspace_id;
        self.get_active_state();
        cx.notify();
    }

    /// Toggle expansion state of a directory
    fn toggle_expanded(&mut self, entry_id: ProjectEntryId, cx: &mut Context<Self>) {
        let state = self.get_active_state();

        if is_expanded(entry_id, &state.expanded_dir_ids) {
            collapse_dir(entry_id, &mut state.expanded_dir_ids);
        } else {
            expand_dir(entry_id, &mut state.expanded_dir_ids);
        }

        // TODO: Rebuild visible_entries based on new expansion state
        // This will be implemented in Wave 3 with actual filesystem integration

        cx.notify();
    }

    /// Select an entry
    fn select_entry(&mut self, entry_id: ProjectEntryId, cx: &mut Context<Self>) {
        let state = self.get_active_state();
        state.selection = Some(entry_id);
        state.marked_entries.clear();

        let path = state
            .visible_entries
            .iter()
            .find(|e| e.id == entry_id)
            .map(|e| e.path.clone());

        cx.emit(FileBrowserPaneEvent::SelectionChanged(path));
        cx.notify();
    }

    /// Move selection up
    fn move_selection_up(&mut self, cx: &mut Context<Self>) {
        // Extract needed data from state first to avoid borrow conflicts
        let new_entry_id = {
            let state = self.get_active_state();
            if state.visible_entries.is_empty() {
                return;
            }

            let current_index = state
                .selection
                .and_then(|id| state.visible_entries.iter().position(|e| e.id == id))
                .unwrap_or(0);

            let new_index = if current_index == 0 {
                state.visible_entries.len() - 1
            } else {
                current_index - 1
            };

            state.visible_entries.get(new_index).map(|e| e.id)
        };

        if let Some(entry_id) = new_entry_id {
            self.select_entry(entry_id, cx);
        }
    }

    /// Move selection down
    fn move_selection_down(&mut self, cx: &mut Context<Self>) {
        // Extract needed data from state first to avoid borrow conflicts
        let new_entry_id = {
            let state = self.get_active_state();
            if state.visible_entries.is_empty() {
                return;
            }

            let current_index = state
                .selection
                .and_then(|id| state.visible_entries.iter().position(|e| e.id == id))
                .unwrap_or(0);

            let new_index = if current_index >= state.visible_entries.len() - 1 {
                0
            } else {
                current_index + 1
            };

            state.visible_entries.get(new_index).map(|e| e.id)
        };

        if let Some(entry_id) = new_entry_id {
            self.select_entry(entry_id, cx);
        }
    }

    /// Expand or toggle the selected entry
    fn expand_selected_entry(&mut self, cx: &mut Context<Self>) {
        // Extract action info from state first to avoid borrow conflicts
        let action = {
            let state = self.get_active_state();

            let Some(selected_id) = state.selection else {
                return;
            };

            let Some(entry) = state.visible_entries.iter().find(|e| e.id == selected_id) else {
                return;
            };

            if entry.is_dir {
                if is_expanded(entry.id, &state.expanded_dir_ids) {
                    None // Already expanded, do nothing
                } else {
                    Some((entry.id, true, None)) // (id, is_dir, path_for_open)
                }
            } else {
                Some((entry.id, false, Some(entry.path.clone())))
            }
        };

        if let Some((entry_id, is_dir, path)) = action {
            if is_dir {
                self.toggle_expanded(entry_id, cx);
            } else if let Some(p) = path {
                cx.emit(FileBrowserPaneEvent::OpenFile(p));
            }
        }
    }

    /// Collapse the selected entry
    fn collapse_selected_entry(&mut self, cx: &mut Context<Self>) {
        // Extract action info from state first to avoid borrow conflicts
        let action = {
            let state = self.get_active_state();

            let Some(selected_id) = state.selection else {
                return;
            };

            let Some(entry) = state
                .visible_entries
                .iter()
                .find(|e| e.id == selected_id)
                .cloned()
            else {
                return;
            };

            if entry.is_dir && is_expanded(entry.id, &state.expanded_dir_ids) {
                Some((entry.id, true)) // (id, should_toggle)
            } else if entry.depth > 0 {
                // Find parent directory
                let parent_depth = entry.depth - 1;
                state
                    .visible_entries
                    .iter()
                    .rev()
                    .find(|e| e.is_dir && e.depth == parent_depth)
                    .map(|parent| (parent.id, false)) // (id, should_toggle=false means select)
            } else {
                None
            }
        };

        if let Some((entry_id, should_toggle)) = action {
            if should_toggle {
                self.toggle_expanded(entry_id, cx);
            } else {
                self.select_entry(entry_id, cx);
            }
        }
    }

    /// Handle keyboard input
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event.keystroke.key.as_str() {
            "up" => {
                self.move_selection_up(cx);
                cx.stop_propagation();
            }
            "down" => {
                self.move_selection_down(cx);
                cx.stop_propagation();
            }
            "left" => {
                self.collapse_selected_entry(cx);
                cx.stop_propagation();
            }
            "right" | "enter" | " " => {
                self.expand_selected_entry(cx);
                cx.stop_propagation();
            }
            _ => {}
        }
    }

    /// Handle mouse click on an entry
    fn handle_entry_click(
        &mut self,
        entry_id: ProjectEntryId,
        is_dir: bool,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if is_dir {
            self.toggle_expanded(entry_id, cx);
        }
        self.select_entry(entry_id, cx);
    }

    /// Render a single entry row
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.listener requires &mut
    fn render_entry_row(&self, entry: &Entry, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state_by_workspace.get(&self.active_workspace_id);

        let is_selected = state
            .and_then(|s| s.selection) == Some(entry.id);

        let is_expanded_entry = state.is_some_and(|s| is_expanded(entry.id, &s.expanded_dir_ids));

        let is_marked = state.is_some_and(|s| s.marked_entries.contains(&entry.id));

        let details = EntryDetails {
            id: entry.id,
            filename: entry
                .path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string(),
            depth: entry.depth,
            is_dir: entry.is_dir,
            is_expanded: is_expanded_entry,
            is_selected,
            is_marked,
            is_editing: false,
            path: entry.path.clone(),
        };

        let entry_id = entry.id;
        let is_dir = entry.is_dir;

        div()
            .id(("file-entry", entry_id))
            .cursor_pointer()
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, _event: &MouseDownEvent, window, cx| {
                    this.handle_entry_click(entry_id, is_dir, window, cx);
                }),
            )
            .child(render_entry(details, cx))
    }

    /// Render the file tree using `uniform_list`
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.listener requires &mut
    fn render_tree(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let state = self.state_by_workspace.get(&self.active_workspace_id);
        let entry_count = state.map_or(0, |s| s.visible_entries.len());

        if entry_count == 0 {
            return div()
                .flex_1()
                .flex()
                .items_center()
                .justify_center()
                .text_color(cx.theme().colors().text_muted)
                .child("No files to display")
                .into_any_element();
        }

        let entries: Vec<Entry> = state
            .map(|s| s.visible_entries.clone())
            .unwrap_or_default();

        div()
            .flex_1()
            .overflow_hidden()
            .child(
                uniform_list("file-tree", entry_count, {
                    cx.processor(move |this, range: std::ops::Range<usize>, _window, cx| {
                        range
                            .filter_map(|ix| {
                                entries.get(ix).map(|entry| {
                                    this.render_entry_row(entry, cx).into_any_element()
                                })
                            })
                            .collect()
                    })
                })
                .track_scroll(&self.scroll_handle),
            )
            .into_any_element()
    }
}

impl EventEmitter<FileBrowserPaneEvent> for FileBrowserPane {}

impl Focusable for FileBrowserPane {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for FileBrowserPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let mut key_context = KeyContext::default();
        key_context.add("FileBrowserPane");

        div()
            .key_context(key_context)
            .track_focus(&self.focus_handle)
            .on_key_down(cx.listener(Self::handle_key_down))
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors().panel_background)
            .border_r_1()
            .border_color(theme.colors().border)
            .child(
                // Header
                div()
                    .h(px(32.0))
                    .w_full()
                    .flex()
                    .items_center()
                    .px_2()
                    .border_b_1()
                    .border_color(theme.colors().border)
                    .child(
                        div()
                            .text_sm()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .text_color(theme.colors().text)
                            .child("Files"),
                    ),
            )
            .child(self.render_tree(cx))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn placeholder_tree_has_entries() {
        let entries = FileBrowserPane::create_placeholder_tree();
        assert!(!entries.is_empty());
        assert!(entries.iter().any(|e| e.is_dir));
        assert!(entries.iter().any(|e| !e.is_dir));
    }

    #[test]
    fn runtime_state_default() {
        let state = FileBrowserRuntimeState::default();
        assert!(state.visible_entries.is_empty());
        assert!(state.expanded_dir_ids.is_empty());
        assert!(state.selection.is_none());
        assert!(state.marked_entries.is_empty());
        assert!((state.scroll_offset - 0.0).abs() < f32::EPSILON);
    }
}
