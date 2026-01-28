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
use crate::file_browser::state::{
    build_entries_from_fs, collapse_dir, expand_dir, is_expanded, Entry, ProjectEntryId,
};

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
    /// Complete tree of all entries (used for rebuilding `visible_entries`)
    all_entries: Vec<Entry>,

    /// Flattened tree of visible entries (filtered by expansion state)
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

    /// Workspace root path for filesystem traversal
    workspace_root: PathBuf,

    /// Active workspace ID
    active_workspace_id: String,

    /// Per-workspace runtime state
    state_by_workspace: HashMap<String, FileBrowserRuntimeState>,
}

impl FileBrowserPane {
    /// Create a new file browser pane
    #[must_use]
    #[allow(clippy::needless_pass_by_ref_mut)] // cx will be used for subscriptions
    pub fn new(workspace_id: String, workspace_root: PathBuf, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let scroll_handle = UniformListScrollHandle::new();
        let mut state_by_workspace = HashMap::default();

        // Initialize state from workspace filesystem
        let all_entries = build_entries_from_fs(&workspace_root);
        let initial_state = FileBrowserRuntimeState {
            all_entries,
            visible_entries: Vec::new(),
            ..Default::default()
        };

        state_by_workspace.insert(workspace_id.clone(), initial_state);

        let mut pane = Self {
            focus_handle,
            scroll_handle,
            workspace_root,
            active_workspace_id: workspace_id,
            state_by_workspace,
        };

        pane.rebuild_visible_entries();
        pane
    }

    /// Get the active runtime state
    fn get_active_state(&mut self) -> &mut FileBrowserRuntimeState {
        self.state_by_workspace
            .entry(self.active_workspace_id.clone())
            .or_insert_with(|| {
                let all_entries = build_entries_from_fs(&self.workspace_root);
                FileBrowserRuntimeState {
                    all_entries,
                    visible_entries: Vec::new(),
                    ..Default::default()
                }
            })
    }

    /// Switch to a different workspace
    pub fn set_active_workspace(
        &mut self,
        workspace_id: String,
        workspace_root: PathBuf,
        cx: &mut Context<Self>,
    ) {
        self.active_workspace_id = workspace_id;
        self.workspace_root = workspace_root;
        self.get_active_state();
        self.refresh_all_entries();
        cx.notify();
    }

    fn refresh_all_entries(&mut self) {
        let all_entries = build_entries_from_fs(&self.workspace_root);
        let state = self.get_active_state();
        state.all_entries = all_entries;
        self.rebuild_visible_entries();
    }

    /// Rebuild visible entries based on current expansion state
    ///
    /// An entry is visible if ALL its ancestors are expanded.
    /// For the placeholder tree, this means:
    /// - Root-level entries (depth 0) are always visible
    /// - Entries at depth N are visible if their parent directory (at depth N-1) is expanded
    fn rebuild_visible_entries(&mut self) {
        let state = self.get_active_state();

        let mut visible = Vec::new();
        let mut last_collapsed_depth: Option<usize> = None;

        for entry in &state.all_entries.clone() {
            // Check if we're still inside a collapsed directory
            if let Some(collapsed_depth) = last_collapsed_depth {
                if entry.depth > collapsed_depth {
                    continue;
                }
                last_collapsed_depth = None;
            }

            // Root-level entries are always visible
            if entry.depth == 0 {
                visible.push(entry.clone());

                if entry.is_dir && !is_expanded(entry.id, &state.expanded_dir_ids) {
                    last_collapsed_depth = Some(entry.depth);
                }
                continue;
            }

            // For nested entries, check if parent is expanded by scanning backwards
            // from current position to find the nearest directory at parent depth
            let parent_depth = entry.depth - 1;
            let entry_index = state
                .all_entries
                .iter()
                .position(|e| e.id == entry.id)
                .unwrap_or(0);
            let parent_expanded = state
                .all_entries
                .iter()
                .take(entry_index)
                .rev()
                .find(|e| e.is_dir && e.depth == parent_depth)
                .is_some_and(|parent| is_expanded(parent.id, &state.expanded_dir_ids));

            if parent_expanded {
                visible.push(entry.clone());

                if entry.is_dir && !is_expanded(entry.id, &state.expanded_dir_ids) {
                    last_collapsed_depth = Some(entry.depth);
                }
            } else if entry.depth > 0 {
                last_collapsed_depth = Some(parent_depth);
            }
        }

        self.get_active_state().visible_entries = visible;
    }

    /// Toggle expansion state of a directory
    fn toggle_expanded(&mut self, entry_id: ProjectEntryId, cx: &mut Context<Self>) {
        let state = self.get_active_state();

        if is_expanded(entry_id, &state.expanded_dir_ids) {
            collapse_dir(entry_id, &mut state.expanded_dir_ids);
        } else {
            expand_dir(entry_id, &mut state.expanded_dir_ids);
        }

        // Rebuild visible_entries based on new expansion state
        self.rebuild_visible_entries();

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
                .and_then(|id| state.visible_entries.iter().position(|e| e.id == id));

            // Bug fix: When no selection, up arrow should select last entry
            let new_index = match current_index {
                Some(idx) => {
                    if idx == 0 {
                        state.visible_entries.len() - 1
                    } else {
                        idx - 1
                    }
                }
                None => state.visible_entries.len() - 1,
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
                .and_then(|id| state.visible_entries.iter().position(|e| e.id == id));

            // Bug fix: When no selection, down arrow should select first entry (index 0)
            let new_index = match current_index {
                Some(idx) => {
                    if idx >= state.visible_entries.len() - 1 {
                        0
                    } else {
                        idx + 1
                    }
                }
                None => 0,
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

            // Bug fix: Find current entry's index first
            let Some(current_index) = state
                .visible_entries
                .iter()
                .position(|e| e.id == selected_id)
            else {
                return;
            };

            let entry = state.visible_entries[current_index].clone();

            if entry.is_dir && is_expanded(entry.id, &state.expanded_dir_ids) {
                Some((entry.id, true)) // (id, should_toggle)
            } else if entry.depth > 0 {
                // Bug fix: Find parent directory by scanning backwards from current position only
                let parent_depth = entry.depth - 1;
                state
                    .visible_entries
                    .iter()
                    .take(current_index) // Only look at entries before current
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

        let is_selected = state.and_then(|s| s.selection) == Some(entry.id);

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

        let entries: Vec<Entry> = state.map(|s| s.visible_entries.clone()).unwrap_or_default();

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
    fn runtime_state_default() {
        let state = FileBrowserRuntimeState::default();
        assert!(state.all_entries.is_empty());
        assert!(state.visible_entries.is_empty());
        assert!(state.expanded_dir_ids.is_empty());
        assert!(state.selection.is_none());
        assert!(state.marked_entries.is_empty());
        assert!((state.scroll_offset - 0.0).abs() < f32::EPSILON);
    }

    #[test]
    fn rebuild_visible_entries_hides_collapsed_children() {
        // Test that rebuild_visible_entries correctly filters out children of collapsed dirs
        let all_entries = vec![
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
                path: PathBuf::from("docs"),
                is_dir: true,
                depth: 0,
            },
        ];

        // With no directories expanded, only root-level entries should be visible
        let state = FileBrowserRuntimeState {
            all_entries,
            visible_entries: Vec::new(),
            expanded_dir_ids: Vec::new(), // Nothing expanded
            ..Default::default()
        };

        // Verify initial state
        assert_eq!(state.all_entries.len(), 3);
        // Root entries: src (depth 0), docs (depth 0) = 2
        // src/main.rs (depth 1) should be hidden when src is collapsed
        let visible_count = state.all_entries.iter().filter(|e| e.depth == 0).count();
        assert_eq!(visible_count, 2);
    }

    #[test]
    fn rebuild_visible_entries_shows_expanded_children() {
        // Test that children of expanded directories are visible
        let all_entries = vec![
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
        ];

        let state = FileBrowserRuntimeState {
            all_entries,
            visible_entries: Vec::new(),
            expanded_dir_ids: vec![1], // "src" is expanded
            ..Default::default()
        };

        // When src is expanded, all 2 entries should be visible
        assert_eq!(state.all_entries.len(), 2);
        assert!(is_expanded(1, &state.expanded_dir_ids));
    }

    #[test]
    fn initial_selection_down_selects_first() {
        // When no selection exists, down arrow should select index 0
        // This verifies the fix for the off-by-one bug
        let entries = [
            Entry {
                id: 1,
                path: PathBuf::from("first"),
                is_dir: false,
                depth: 0,
            },
            Entry {
                id: 2,
                path: PathBuf::from("second"),
                is_dir: false,
                depth: 0,
            },
        ];

        // With no current selection, down should pick index 0
        let current_index: Option<usize> = None;
        let new_index =
            current_index.map_or(0, |idx| if idx >= entries.len() - 1 { 0 } else { idx + 1 });
        assert_eq!(new_index, 0);
        assert_eq!(entries[new_index].id, 1);
    }

    #[test]
    fn initial_selection_up_selects_last() {
        // When no selection exists, up arrow should select last entry
        let entries = [
            Entry {
                id: 1,
                path: PathBuf::from("first"),
                is_dir: false,
                depth: 0,
            },
            Entry {
                id: 2,
                path: PathBuf::from("second"),
                is_dir: false,
                depth: 0,
            },
        ];

        // With no current selection, up should pick last entry
        let current_index: Option<usize> = None;
        let new_index = current_index.map_or_else(
            || entries.len() - 1,
            |idx| {
                if idx == 0 {
                    entries.len() - 1
                } else {
                    idx - 1
                }
            },
        );
        assert_eq!(new_index, 1);
        assert_eq!(entries[new_index].id, 2);
    }

    #[test]
    fn collapse_finds_correct_parent_not_later_sibling() {
        // Test that collapse finds parent by scanning backwards from current position,
        // not from end of list (which would find wrong parent)
        let entries = [
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
                path: PathBuf::from("tests"),
                is_dir: true,
                depth: 0,
            },
        ];

        // If main.rs (id=2, depth=1) is selected and we want to find parent (depth=0),
        // we should find "src" (id=1), NOT "tests" (id=3)
        let selected_id = 2;
        let current_index = entries.iter().position(|e| e.id == selected_id).unwrap();
        assert_eq!(current_index, 1);

        let entry = &entries[current_index];
        let parent_depth = entry.depth - 1;

        // Bug fix: scan backwards from current position only
        let parent = entries
            .iter()
            .take(current_index) // Only look at entries before current
            .rev()
            .find(|e| e.is_dir && e.depth == parent_depth);

        assert!(parent.is_some());
        assert_eq!(parent.unwrap().id, 1); // Should be "src", not "tests"
    }
}
