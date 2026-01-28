# Phase 3 Sprint 1 Design: File Browser Implementation

**Version:** 1.0
**Created:** 2026-01-27
**Status:** Design Complete
**Estimated Duration:** 8-10 hours
**Target Branch:** `feature/sprint-3-1-wave3`
**Worktree Path:** `/Users/randlee/Documents/github/terminalg-worktrees/feature/sprint-3-1-wave3`

---

## 1. Executive Summary

Sprint 3.1 migrates Zed's `project_panel` to TerminalG as a file browser pane, providing tree-based file navigation with git status indicators, keyboard controls, context menu operations, and integration with the workspace system.

### Key Deliverables
1. **FileBrowserPane** - Tree view with expand/collapse, selection, git status
2. **Context Menu** - File operations (new, rename, delete, copy, paste, reveal, open in terminal)
3. **Inline Editing** - Rename/create operations with validation
4. **Workspace Integration** - Left pane placement, state persistence
5. **Terminal Integration** - "Open in Terminal" action

### Strategic Decisions
- **Use Zed's `project` crate as git dependency** (like terminal)
- **Project/worktree owned by WorkspaceView** and lazily initialized on first selection
- **Pause file watching when hidden** and full refresh on re-show (debounced ~500ms)
- **Adapt project_panel patterns**, not copy wholesale
- **Include most features** - Don't over-simplify
- **Defer search/diff** - Requires additional infrastructure

---

## 2. Pattern Analysis: Existing TerminalG Code

### 2.1 TerminalPane Pattern (Reference Implementation)

**File:** `/Users/randlee/Documents/github/terminalg/src/terminal/pane.rs`

**Key Patterns Identified:**
```rust
// Pattern 1: Per-workspace state management
pub struct TerminalPane {
    tabs_by_workspace: HashMap<String, Vec<TerminalTab>>,
    active_workspace_id: String,
    active_tab_by_workspace: HashMap<String, usize>,
    working_directory_by_workspace: HashMap<String, Option<PathBuf>>,
    focus_handle: FocusHandle,
}

// Pattern 2: Workspace switching
pub fn set_active_workspace(&mut self, workspace_id: String, cx: &mut Context<Self>) {
    self.active_workspace_id = workspace_id.clone();
    if !self.tabs_by_workspace.contains_key(&workspace_id) {
        // Lazy load
        self.spawn_terminal(workspace_id.clone(), working_dir, cx);
    }
    cx.notify();
}

// Pattern 3: Event emitter
impl EventEmitter<TerminalPaneEvent> for TerminalPane {}

// Pattern 4: Focusable
impl Focusable for TerminalPane {
    fn focus_handle(&self, _cx: &mut App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

// Pattern 5: Render with theme colors
fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();
    div()
        .bg(theme.colors().panel_background)
        .child(...)
}
```

**Insights for FileBrowserPane:**
- Use HashMap for per-workspace state
- Emit events for workspace integration
- Use theme colors from `cx.theme()`
- Focus handle for keyboard navigation
- Lazy loading on workspace switch

### 2.2 WorkspaceView Pattern (Integration Point)

**File:** `/Users/randlee/Documents/github/terminalg/src/ui/workspace.rs`

**Key Patterns Identified:**
```rust
pub struct WorkspaceView {
    // Pane visibility
    file_browser_visible: bool,
    terminal_visible: bool,
    document_viewer_visible: bool,

    // Pane ratios for layout
    pane_ratios: [f32; 3],

    // Pane instances
    terminal_pane: Option<Entity<TerminalPane>>,

    // Resize drag state
    resize_drag_state: Option<ResizeDragState>,
}

// Three-pane layout with dividers
fn render_three_pane_layout(&self, ...) -> impl IntoElement {
    h_flex()
        .child(file_browser_pane)
        .child(divider)
        .child(terminal_pane)
        .child(divider)
        .child(document_viewer_pane)
}
```

**Insights for Integration:**
- FileBrowserPane goes in left pane (already has placeholder)
- Use `Entity<FileBrowserPane>` stored in WorkspaceView
- Visibility controlled by `file_browser_visible` flag
- Width ratio already in `pane_ratios[0]`

### 2.3 WorkspaceConfig Pattern (State Persistence)

**File:** `/Users/randlee/Documents/github/terminalg/src/ui/workspace_config.rs`

**Key Patterns Identified:**
```rust
#[derive(Serialize, Deserialize)]
pub struct WorkspaceConfig {
    pub id: String,
    pub file_browser_visible: bool,
    // Auto-saved on changes with 200ms debounce
}

impl WorkspaceConfigStore {
    pub fn update_and_save(&mut self, config: WorkspaceConfig) -> Result<()> {
        // Debounced save to .terminalg/workspace.json
    }
}
```

**Insights for FileBrowser:**
- Add `file_browser_state: Option<FileBrowserPersistedState>` to WorkspaceConfig (best-effort restore)
- Serialize expanded directories, scroll position, selected path
- Auto-save on expand/collapse/selection changes

---

## 3. Pattern Analysis: Zed's ProjectPanel

### 3.1 Core Data Structures

**From:** `/Users/randlee/Documents/github/zed/crates/project_panel/src/project_panel.rs`

```rust
// Key structure: Flattened tree per worktree
struct VisibleEntriesForWorktree {
    worktree_id: WorktreeId,
    entries: Vec<GitEntry>,  // Flattened, sorted tree
    index: OnceCell<HashSet<Arc<RelPath>>>,
}

struct State {
    visible_entries: Vec<VisibleEntriesForWorktree>,
    expanded_dir_ids: HashMap<WorktreeId, Vec<ProjectEntryId>>,  // Binary searched
    selection: Option<SelectedEntry>,
    edit_state: Option<EditState>,
    unfolded_dir_ids: HashSet<ProjectEntryId>,  // Auto-fold override
}

pub struct ProjectPanel {
    project: Entity<Project>,
    fs: Arc<dyn Fs>,
    focus_handle: FocusHandle,
    scroll_handle: UniformListScrollHandle,
    filename_editor: Entity<Editor>,
    context_menu: Option<(Entity<ContextMenu>, Point<Pixels>, Subscription)>,
    state: State,
}
```

**Key Insights:**
1. **Flattened tree** - Not recursive rendering, flattened Vec<GitEntry>
2. **Binary search** - expanded_dir_ids kept sorted for fast lookup
3. **GitEntry** - Provided by `project` crate, includes git status
4. **uniform_list** - Virtualized rendering for performance
5. **Background computation** - Tree updates via `cx.spawn()`, not synchronous

### 3.2 Tree Update Flow

```rust
fn update_visible_entries(
    &mut self,
    new_selected_entry: Option<(WorktreeId, ProjectEntryId)>,
    focus_filename_editor: bool,
    autoscroll: bool,
    window: &mut Window,
    cx: &mut Context<Self>,
) {
    // Spawn background task
    self.update_visible_entries_task._visible_entries_task = cx.spawn_in(window, |this, cx| async move {
        // Build flattened tree from project worktrees
        let entries = build_visible_entries(project, expanded_dirs).await;

        // Update state on UI thread
        this.update(cx, |this, cx| {
            this.state.visible_entries = entries;
            cx.notify();
        });
    });
}
```

**Key Pattern:**
- **Non-blocking** - Tree computation in background
- **Notification** - `cx.notify()` triggers re-render
- **Task tracking** - Store Task<()> to prevent overlapping updates

### 3.3 Rendering with uniform_list

```rust
fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    let item_count = self.state.visible_entries.iter()
        .map(|w| w.entries.len())
        .sum();

    v_flex().child(
        uniform_list("entries", item_count, {
            cx.processor(|this, range: Range<usize>, window, cx| {
                let mut items = Vec::new();
                this.for_each_visible_entry(range, window, cx, |id, details, window, cx| {
                    items.push(this.render_entry(id, details, window, cx));
                });
                items
            })
        })
    )
}
```

**Key Pattern:**
- **Virtualization** - Only render visible range
- **Processor closure** - Builds elements on-demand
- **Scrolling** - UniformListScrollHandle for keyboard navigation

### 3.4 Context Menu Pattern

```rust
fn deploy_context_menu(&mut self, position: Point<Pixels>, entry_id: ProjectEntryId, window: &mut Window, cx: &mut Context<Self>) {
    let context_menu = ContextMenu::build(window, cx, |menu, _, _| {
        menu.entry("New File", None, cx.listener(|this, window, cx| { ... }))
            .entry("New Folder", None, cx.listener(|this, window, cx| { ... }))
            .entry("Rename", None, cx.listener(|this, window, cx| { ... }))
            .separator()
            .entry("Delete", None, cx.listener(|this, window, cx| { ... }))
    });

    window.focus(&context_menu.focus_handle(cx), cx);
    let subscription = cx.subscribe(&context_menu, |this, _, _: &DismissEvent, cx| {
        this.context_menu.take();
    });

    self.context_menu = Some((context_menu, position, subscription));
}
```

**Key Pattern:**
- **Entity lifecycle** - ContextMenu is separate entity
- **Subscription** - Auto-cleanup on dismiss
- **Focus management** - Focus menu, restore on close

### 3.5 Inline Editing Pattern

```rust
struct EditState {
    worktree_id: WorktreeId,
    entry_id: ProjectEntryId,
    leaf_entry_id: Option<ProjectEntryId>,  // None = new entry
    is_dir: bool,
    depth: usize,
    processing_filename: Option<Arc<RelPath>>,
    validation_state: ValidationState,
}

fn new_file(&mut self, _: &NewFile, window: &mut Window, cx: &mut Context<Self>) {
    self.state.edit_state = Some(EditState { ... });
    self.filename_editor.update(cx, |editor, cx| {
        editor.set_text("", cx);
    });
    self.update_visible_entries(None, true, false, window, cx);  // focus_filename_editor = true
}

fn confirm(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>) {
    let filename = self.filename_editor.read(cx).text(cx);
    // Validate filename
    // Create file/folder via project.create_entry()
    self.state.edit_state = None;
    self.update_visible_entries(None, false, false, window, cx);
}
```

**Key Pattern:**
- **EditState** - Tracks current edit operation
- **Editor entity** - Reused for all inline edits
- **Validation** - Before confirming operation
- **Project API** - Delegate to project.create_entry(), project.rename_entry()

---

## 4. Architecture Decisions

### Decision 1: Zed `project` Crate Dependency

**Decision:** Add Zed's `project` crate as git dependency (tag v0.220.3)

**Rationale:**
- Provides `Project`, `Worktree`, `GitEntry` types
- Includes git status tracking (GitSummary)
- File watching and updates
- Proven in Zed's production use
- Follows established pattern from terminal integration

**Trade-offs:**
- Saves weeks of implementation time
- Battle-tested git integration
- Consistent with terminal approach
- Adds GPL-3.0 dependency (already GPL from terminal)
- Couples to Zed's API (manageable with tag pinning)

**Rejected Alternative:** Build custom file tree + git tracking
- Would take 15-20 hours just for git status
- High bug risk (edge cases, race conditions)
- Reinventing proven solution

### Decision 2: Adapt, Don't Copy

**Decision:** Adapt Zed's patterns to TerminalG's architecture, don't wholesale copy

**Rationale:**
- TerminalG has different workspace model (no multi-folder projects)
- Already has workspace state persistence system
- Simpler use case (single root per workspace)

**What to Adapt:**
- Flattened tree structure (Vec<GitEntry>)
- uniform_list virtualization
- Binary search for expanded_dir_ids
- Background tree computation
- Context menu pattern
- Inline editor pattern

**What to Simplify:**
- Remove multi-worktree complexity (TerminalG = single root)
- Remove "Remove from Project" action (not applicable)
- Simplify drag-and-drop to essential use cases
- Remove sticky scroll (optional, future enhancement)

### Decision 3: Feature Scope

**Include:**
- Tree view with expand/collapse
- Keyboard navigation (arrows, enter, space, backspace)
- Single + multi-selection (shift-click, cmd-click)
- Git status indicators
- Auto-fold single-child directories
- Drag-and-drop file/folder moving
- uniform_list virtualization
- Context menu (new, rename, delete, copy/paste, reveal, open in terminal)
- Inline editing with validation

**Defer to Future:**
- Find in Folder (needs search infrastructure)
- File History (needs git diff UI)
- Compare files (needs diff viewer)
- Sticky scroll (nice-to-have)
- Indent guides (nice-to-have)

### Decision 4: State Persistence

**Decision:** Extend WorkspaceConfig with FileBrowserPersistedState (paths only)

**Structure:**
```rust
#[derive(Serialize, Deserialize)]
pub struct FileBrowserPersistedState {
    pub expanded_dirs: Vec<PathBuf>,  // Relative to workspace root
    pub scroll_offset: f32,
    pub selected_path: Option<PathBuf>,
}
```

**Rationale:**
- Consistent with existing workspace persistence
- Simple, JSON-serializable
- Per-workspace state isolation
- Auto-saved via existing debounce mechanism
- **Best-effort restore**: if paths no longer exist (rename/move/delete), drop them silently

### Decision 5: "Open in Terminal" Integration

**Decision:** Add `FileBrowserPaneEvent::OpenInTerminal(PathBuf)` event

**Flow:**
1. User right-clicks folder in file browser
2. Selects "Open in Terminal" from context menu
3. FileBrowserPane emits `OpenInTerminal(folder_path)` event
4. WorkspaceView subscribes, handles event
5. WorkspaceView calls `terminal_pane.open_in_directory(path, cx)`
6. TerminalPane spawns new terminal with cwd = path

**Rationale:**
- Follows existing event-driven pattern
- Clean separation of concerns
- Reuses workspace event subscription pattern

---

## 5. Component Design

### 5.1 Module Structure

```
src/file_browser/
├── mod.rs                     # Module exports, types, actions
├── pane.rs                    # FileBrowserPane (main component)
├── state.rs                   # Tree state management
├── render.rs                  # Entry rendering helpers
└── context_menu.rs            # Context menu builder
```

### 5.1.1 Project/Worktree Ownership (WorkspaceView)

- `WorkspaceView` owns a per-workspace `Project` instance.
- `Project`/`Worktree` are **lazily created** the first time a workspace is selected.
- `FileBrowserPane` is created with the workspace’s `Project` entity and does not share it across workspaces.
- File watching is **paused/disabled** while the file browser pane is hidden to avoid background churn.
- When visibility is restored, `FileBrowserPane` performs a **full refresh** of visible entries, debounced to ~500ms.

### 5.2 FileBrowserPane

**File:** `src/file_browser/pane.rs`

**Responsibility:** Main file browser component, orchestrates tree, selection, editing

**Structure:**
```rust
use project::{Project, ProjectEntryId, WorktreeId, GitEntry};
use gpui::{Entity, FocusHandle, UniformListScrollHandle};

pub struct FileBrowserPane {
    // Project integration (provided by WorkspaceView on creation)
    project: Entity<Project>,
    fs: Arc<dyn Fs>,

    // Per-workspace runtime state (not persisted)
    state_by_workspace: HashMap<String, FileBrowserRuntimeState>,
    active_workspace_id: String,

    // UI state
    focus_handle: FocusHandle,
    scroll_handle: UniformListScrollHandle,

    // Inline editing
    filename_editor: Entity<Editor>,

    // Context menu
    context_menu: Option<(Entity<ContextMenu>, Point<Pixels>, Subscription)>,

    // Background tasks
    update_tree_task: Task<()>,

    // Clipboard
    clipboard: Option<ClipboardEntry>,
}

struct FileBrowserRuntimeState {
    // Tree structure (flattened)
    visible_entries: Vec<GitEntry>,

    // Expansion state
    expanded_dir_ids: Vec<ProjectEntryId>,  // Kept sorted for binary search

    // Selection
    selection: Option<ProjectEntryId>,
    marked_entries: Vec<ProjectEntryId>,  // For multi-selection

    // Editing
    edit_state: Option<EditState>,

    // Scroll position
    scroll_offset: f32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct FileBrowserPersistedState {
    expanded_dirs: Vec<PathBuf>,
    scroll_offset: f32,
    selected_path: Option<PathBuf>,
}

#[derive(Clone, Debug)]
enum ClipboardEntry {
    Copied(Vec<ProjectEntryId>),
    Cut(Vec<ProjectEntryId>),
}

#[derive(Clone, Debug)]
struct EditState {
    entry_id: ProjectEntryId,
    leaf_entry_id: Option<ProjectEntryId>,  // None = new entry
    is_dir: bool,
    depth: usize,
    validation_state: ValidationState,
}
```

**Key Methods:**
```rust
impl FileBrowserPane {
    // Lifecycle
    pub fn new(project: Entity<Project>, workspace_id: String, cx: &mut Context<Self>) -> Self;

    // Workspace switching
    pub fn set_active_workspace(
        &mut self,
        workspace_id: String,
        project: Entity<Project>,
        cx: &mut Context<Self>,
    );
    pub fn set_visible(&mut self, visible: bool, window: &mut Window, cx: &mut Context<Self>);
    pub fn save_state(&self) -> FileBrowserPersistedState;
    pub fn load_state(&mut self, state: FileBrowserPersistedState, cx: &mut Context<Self>);

    // Tree management
    fn update_visible_entries(&mut self, autoscroll: bool, window: &mut Window, cx: &mut Context<Self>);
    fn build_flattened_tree(&self, expanded_ids: &[ProjectEntryId]) -> Vec<GitEntry>;

    // Selection
    fn select_entry(&mut self, entry_id: ProjectEntryId, cx: &mut Context<Self>);
    fn toggle_marked(&mut self, entry_id: ProjectEntryId, cx: &mut Context<Self>);

    // Expand/collapse
    fn toggle_expanded(&mut self, entry_id: ProjectEntryId, window: &mut Window, cx: &mut Context<Self>);
    fn expand_entry(&mut self, entry_id: ProjectEntryId, window: &mut Window, cx: &mut Context<Self>);
    fn collapse_entry(&mut self, entry_id: ProjectEntryId, window: &mut Window, cx: &mut Context<Self>);
    fn collapse_all(&mut self, window: &mut Window, cx: &mut Context<Self>);

    // File operations (via context menu)
    fn new_file(&mut self, _: &NewFile, window: &mut Window, cx: &mut Context<Self>);
    fn new_directory(&mut self, _: &NewDirectory, window: &mut Window, cx: &mut Context<Self>);
    fn rename(&mut self, _: &Rename, window: &mut Window, cx: &mut Context<Self>);
    fn delete(&mut self, _: &Delete, window: &mut Window, cx: &mut Context<Self>);
    fn copy(&mut self, _: &Copy, window: &mut Window, cx: &mut Context<Self>);
    fn cut(&mut self, _: &Cut, window: &mut Window, cx: &mut Context<Self>);
    fn paste(&mut self, _: &Paste, window: &mut Window, cx: &mut Context<Self>);
    fn reveal_in_finder(&mut self, _: &RevealInFinder, window: &mut Window, cx: &mut Context<Self>);
    fn open_in_terminal(&mut self, _: &OpenInTerminal, window: &mut Window, cx: &mut Context<Self>);

    // Inline editing
    fn confirm_edit(&mut self, _: &Confirm, window: &mut Window, cx: &mut Context<Self>);
    fn cancel_edit(&mut self, _: &Cancel, window: &mut Window, cx: &mut Context<Self>);
    fn validate_filename(&self, filename: &str, is_dir: bool) -> ValidationState;

    // Context menu
    fn deploy_context_menu(&mut self, position: Point<Pixels>, entry_id: ProjectEntryId, window: &mut Window, cx: &mut Context<Self>);

    // Rendering
    fn render_entry(&self, entry: &GitEntry, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement;
}

// Events
#[derive(Clone, Debug)]
pub enum FileBrowserPaneEvent {
    OpenFile(PathBuf),
    OpenInTerminal(PathBuf),
    SelectionChanged(Option<PathBuf>),
}

impl EventEmitter<FileBrowserPaneEvent> for FileBrowserPane {}
impl Focusable for FileBrowserPane { ... }
impl Render for FileBrowserPane { ... }
```

**Estimated Size:** ~800-1000 lines

### 5.3 State Management Module

**File:** `src/file_browser/state.rs`

**Responsibility:** Tree state computation, flattening, binary search utilities

**Structure:**
```rust
use project::{Project, ProjectEntryId, GitEntry, Worktree};

/// Build flattened tree from worktree, respecting expanded directories
pub fn build_flattened_tree(
    worktree: &Worktree,
    expanded_dir_ids: &[ProjectEntryId],
    auto_fold_dirs: bool,
) -> Vec<GitEntry> {
    // Traverse worktree depth-first
    // Include entry if parent is expanded
    // Auto-fold single-child directories if enabled
    // Filter hidden files only if global setting is enabled
}

/// Binary search in sorted expanded_dir_ids
pub fn is_expanded(entry_id: ProjectEntryId, expanded_dir_ids: &[ProjectEntryId]) -> bool {
    expanded_dir_ids.binary_search(&entry_id).is_ok()
}

/// Insert entry_id into sorted Vec, maintaining sort order
pub fn expand_dir(entry_id: ProjectEntryId, expanded_dir_ids: &mut Vec<ProjectEntryId>) {
    if let Err(ix) = expanded_dir_ids.binary_search(&entry_id) {
        expanded_dir_ids.insert(ix, entry_id);
    }
}

/// Remove entry_id from sorted Vec
pub fn collapse_dir(entry_id: ProjectEntryId, expanded_dir_ids: &mut Vec<ProjectEntryId>) {
    if let Ok(ix) = expanded_dir_ids.binary_search(&entry_id) {
        expanded_dir_ids.remove(ix);
    }
}

/// Calculate depth for rendering indentation
pub fn calculate_depth(entry: &GitEntry, worktree: &Worktree) -> usize {
    entry.path.components().count() - 1
}
```

**Workspace switch behavior:**
- `set_active_workspace` swaps the `Project` handle, loads persisted state if available, and triggers `update_visible_entries`.

**Estimated Size:** ~200-300 lines

### 5.4 Render Module

**File:** `src/file_browser/render.rs`

**Responsibility:** Entry rendering logic, icons, git status colors

**Structure:**
```rust
use gpui::{IntoElement, div, h_flex};
use ui::{Icon, IconName, Label, Color};
use project::{GitEntry, EntryKind};

pub struct EntryDetails {
    pub filename: String,
    pub icon: Option<IconName>,
    pub depth: usize,
    pub kind: EntryKind,
    pub is_expanded: bool,
    pub is_selected: bool,
    pub is_marked: bool,
    pub is_editing: bool,
    pub git_status: GitSummary,
}

impl EntryDetails {
    pub fn from_git_entry(
        entry: &GitEntry,
        is_expanded: bool,
        is_selected: bool,
        is_marked: bool,
        is_editing: bool,
    ) -> Self { ... }
}

/// Render a single file/folder entry
pub fn render_entry(details: EntryDetails, cx: &mut Context) -> impl IntoElement {
    h_flex()
        .gap_1()
        .px_2()
        .py_1()
        .when(details.is_selected, |this| this.bg(cx.theme().colors().element_selected))
        .child(
            // Indentation
            div().w(px(details.depth as f32 * 20.0))
        )
        .child(
            // Expand/collapse icon
            if details.kind == EntryKind::Directory {
                Icon::new(if details.is_expanded { IconName::ChevronDown } else { IconName::ChevronRight })
            } else {
                div().w(px(16.0))
            }
        )
        .child(
            // File/folder icon
            if let Some(icon) = details.icon {
                Icon::new(icon)
            } else {
                div()
            }
        )
        .child(
            // Filename with git status color
            Label::new(details.filename)
                .color(git_status_color(details.git_status))
        )
        .child(
            // Git status indicator
            if details.git_status != GitSummary::default() {
                Label::new(git_status_char(details.git_status))
            } else {
                div()
            }
        )
}

fn git_status_color(status: GitSummary) -> Color {
    // Map git status to theme colors
}

fn git_status_char(status: GitSummary) -> &'static str {
    // "M", "A", "D", "?", etc.
}
```

**Estimated Size:** ~150-200 lines

### 5.5 Context Menu Module

**File:** `src/file_browser/context_menu.rs`

**Responsibility:** Build context menu for file/folder operations

**Structure:**
```rust
use gpui::{Entity, Context, Window};
use ui::ContextMenu;

pub fn build_context_menu(
    entry_id: ProjectEntryId,
    is_dir: bool,
    pane: &FileBrowserPane,
    window: &mut Window,
    cx: &mut Context<FileBrowserPane>,
) -> Entity<ContextMenu> {
    ContextMenu::build(window, cx, |menu, _, _| {
        menu
            .entry("New File", None, cx.listener(|this, window, cx| {
                this.new_file(&NewFile, window, cx);
            }))
            .entry("New Folder", None, cx.listener(|this, window, cx| {
                this.new_directory(&NewDirectory, window, cx);
            }))
            .separator()
            .entry("Rename", Some("F2"), cx.listener(|this, window, cx| {
                this.rename(&Rename, window, cx);
            }))
            .entry("Delete", Some("Delete"), cx.listener(|this, window, cx| {
                this.delete(&Delete, window, cx);
            }))
            .separator()
            .entry("Cut", Some("Cmd+X"), cx.listener(|this, window, cx| {
                this.cut(&Cut, window, cx);
            }))
            .entry("Copy", Some("Cmd+C"), cx.listener(|this, window, cx| {
                this.copy(&Copy, window, cx);
            }))
            .entry("Paste", Some("Cmd+V"), cx.listener(|this, window, cx| {
                this.paste(&Paste, window, cx);
            }))
            .separator()
            .entry("Copy Path", None, cx.listener(|this, window, cx| {
                this.copy_path(&CopyPath, window, cx);
            }))
            .entry("Copy Relative Path", None, cx.listener(|this, window, cx| {
                this.copy_relative_path(&CopyRelativePath, window, cx);
            }))
            .separator()
            .entry("Reveal in Finder", None, cx.listener(|this, window, cx| {
                this.reveal_in_finder(&RevealInFinder, window, cx);
            }))
            .when(is_dir, |menu| {
                menu.entry("Open in Terminal", None, cx.listener(|this, window, cx| {
                    this.open_in_terminal(&OpenInTerminal, window, cx);
                }))
            })
            .separator()
            .entry("Collapse All", None, cx.listener(|this, window, cx| {
                this.collapse_all(&CollapseAll, window, cx);
            }))
    })
}
```

**Estimated Size:** ~100 lines

---

## 6. Data Flow

### 6.1 Tree Update Flow

```
User Action (expand/collapse/new file)
    |
    v
FileBrowserPane method (e.g., expand_entry)
    |
    v
Modify expanded_dir_ids (binary search insert/remove)
    |
    v
Call update_visible_entries()
    |
    v
Spawn background task: cx.spawn_in(...)
    |
    v
    Build flattened tree from Project worktree
        |
        v
        Traverse worktree depth-first
        |
        v
        Include entries where parent is expanded
        |
        v
        Apply auto-fold for single-child dirs
    |
    v
Update state on UI thread
    |
    v
    state.visible_entries = new_tree
    |
    v
    cx.notify() -> triggers re-render
    |
    v
Render with uniform_list (virtualized)
    |
    v
Only render visible range (e.g., rows 10-30)
```

### 6.2 Selection Flow

```
User clicks entry
    |
    v
Mouse down handler
    |
    v
Check modifiers:
    - None: select_entry(id)
    - Cmd: toggle_marked(id)
    - Shift: select_range(from, to)
    |
    v
Update state.selection / state.marked_entries
    |
    v
Emit FileBrowserPaneEvent::SelectionChanged
    |
    v
WorkspaceView subscribes, updates document viewer if needed
    |
    v
cx.notify() -> re-render with highlight
```

### 6.3 Context Menu Flow

```
User right-clicks entry
    |
    v
Mouse down handler (MouseButton::Right)
    |
    v
Call deploy_context_menu(position, entry_id)
    |
    v
Build ContextMenu entity with actions
    |
    v
Focus context menu
    |
    v
Subscribe to DismissEvent
    |
    v
Store (menu_entity, position, subscription)
    |
    v
User selects action -> listener fires
    |
    v
Execute action (new_file, rename, delete, etc.)
    |
    v
Context menu dismissed -> subscription cleanup
```

### 6.4 Inline Editing Flow

```
User selects "New File" or "Rename"
    |
    v
Set edit_state = Some(EditState { ... })
    |
    v
Update filename_editor text
    |
    v
Call update_visible_entries(focus_editor = true)
    |
    v
Re-render with inline editor at entry depth
    |
    v
User types -> validate filename on each keystroke
    |
    v
validation_state = ValidationState::Warning/Error/None
    |
    v
User presses Enter -> confirm_edit()
    |
    v
    If valid:
        - Call project.create_entry() or project.rename_entry()
        - Clear edit_state
        - Update tree
    |
    v
    If invalid:
        - Show error toast
        - Keep editing
```

### 6.5 "Open in Terminal" Flow

```
User right-clicks folder
    |
    v
Selects "Open in Terminal" from context menu
    |
    v
open_in_terminal() method
    |
    v
Emit FileBrowserPaneEvent::OpenInTerminal(folder_path)
    |
    v
WorkspaceView subscription handler
    |
    v
terminal_pane.spawn_terminal(workspace_id, Some(path), cx)
    |
    v
New terminal tab opens with cwd = folder_path
    |
    v
Switch to terminal pane (set terminal_visible = true)
```

---

## 7. Workspace Integration

### 7.1 WorkspaceView Changes

**File:** `src/ui/workspace.rs`

**Changes Required:**

```rust
pub struct WorkspaceView {
    // Add file browser pane
    file_browser_pane: Entity<FileBrowserPane>,

    // Per-workspace project instances (lazy)
    project_by_workspace: HashMap<String, Entity<Project>>,

    // Existing fields...
    terminal_pane: Entity<TerminalPane>,
}

impl WorkspaceView {
    pub fn new(cx: &mut Context<Self>) -> Self {
        // Create project for active workspace (others are lazy)
        let workspace_id = config_store.active_workspace().id.clone();
        let root = config_store.workspace_root().to_path_buf();
        let project = cx.new(|cx| Project::local(root, cx));
        let mut project_by_workspace = HashMap::default();
        project_by_workspace.insert(workspace_id.clone(), project.clone());

        // Create file browser pane
        let file_browser_pane =
            cx.new(|cx| FileBrowserPane::new(project.clone(), workspace_id, cx));

        // Subscribe to file browser events
        cx.subscribe(&file_browser_pane, Self::handle_file_browser_event);

        Self {
            file_browser_pane,
            project_by_workspace,
            ...
        }
    }

    fn ensure_project_for_workspace(
        &mut self,
        workspace_id: &str,
        cx: &mut Context<Self>,
    ) -> Entity<Project> {
        if let Some(project) = self.project_by_workspace.get(workspace_id) {
            return project.clone();
        }
        let root = self.config_store.workspace_root().to_path_buf();
        let project = cx.new(|cx| Project::local(root, cx));
        self.project_by_workspace
            .insert(workspace_id.to_string(), project.clone());
        project
    }

    fn handle_file_browser_event(
        &mut self,
        _pane: Entity<FileBrowserPane>,
        event: &FileBrowserPaneEvent,
        window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        match event {
            FileBrowserPaneEvent::OpenFile(path) => {
                // Future: open in document viewer
            }
            FileBrowserPaneEvent::OpenInTerminal(path) => {
                self.terminal_pane.update(cx, |pane, cx| {
                    pane.spawn_terminal(
                        self.active_workspace_id.clone(),
                        Some(path.clone()),
                        cx,
                    );
                });
                self.terminal_visible = true;
                cx.notify();
            }
            FileBrowserPaneEvent::SelectionChanged(_path) => {
                // Future: preview in document viewer
            }
        }
    }

    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let file_browser_content = self.file_browser_pane.clone().into_any_element();

        h_flex()
            .when(self.file_browser_visible, |flex| {
                flex.child(
                    div()
                        .w(file_browser_width)
                        .child(file_browser_content)
                )
            })
            // ... rest of layout
    }
}
```

**Visibility handling:**
- On `file_browser_visible = false`, call `file_browser_pane.set_visible(false, ...)` to pause watchers.
- On `true`, call `set_visible(true, ...)` to resume watchers and trigger a debounced full refresh (~500ms).
**Workspace switching:**
- On workspace change, call `ensure_project_for_workspace()` and then `file_browser_pane.set_active_workspace(workspace_id, project, cx)`.

**Estimated Changes:** ~100 lines added/modified

### 7.2 WorkspaceConfig Extension

**File:** `src/ui/workspace_config.rs`

**Changes Required:**

```rust
#[derive(Serialize, Deserialize)]
pub struct WorkspaceConfig {
    // Existing fields...

    // Add file browser state
    #[serde(default)]
    pub file_browser_state: Option<FileBrowserPersistedState>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct FileBrowserPersistedState {
    /// Expanded directory paths (relative to workspace root)
    pub expanded_dirs: Vec<PathBuf>,

    /// Scroll offset (pixels from top)
    pub scroll_offset: f32,

    /// Currently selected path (relative to workspace root)
    pub selected_path: Option<PathBuf>,
}

impl Default for FileBrowserPersistedState {
    fn default() -> Self {
        Self {
            expanded_dirs: vec![],
            scroll_offset: 0.0,
            selected_path: None,
        }
    }
}
```

**Restore behavior:** Best-effort; if persisted paths are missing on load (rename/move/delete), drop them silently.

**Estimated Changes:** ~30 lines added

---

### 7.3 Global Settings (Hidden Files)

- Add a global setting `file_browser.hide_hidden_files: bool` (default `false`).
- Settings source of truth should live in the existing settings system (Zed settings adapter).
- `build_flattened_tree` checks this setting; when `true`, filter entries with filenames starting with `.`.

---

## 8. Zed Crate Dependencies

### 8.1 New Cargo.toml Additions

```toml
# Add to [dependencies]

# Project management (GPL-3.0)
project = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }

# File system abstraction (GPL-3.0)
fs = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }

# Worktree management (GPL-3.0)
worktree = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }

# Git integration (GPL-3.0)
git = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }

# File icons (GPL-3.0)
file_icons = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }

# Editor component (for inline editing)
editor = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
```

### 8.2 Transitive Dependencies

These crates will be pulled in automatically:
- `text` - Text buffer management
- `language` - Language detection, syntax
- `rpc` - Remote protocol (used internally by project)
- `client` - Client types (used by project)
- `paths` - Path utilities

### 8.3 Build Impact

**Estimated Additional Build Time:**
- Clean build: +2-3 minutes
- Incremental: +10-20 seconds

**Binary Size Impact:** +5-8 MB

---

## 9. Implementation Map

### 9.1 Files to Create

| File | Lines | Description |
|------|-------|-------------|
| `src/file_browser/mod.rs` | 50 | Module exports, action definitions |
| `src/file_browser/pane.rs` | 800-1000 | Main FileBrowserPane component |
| `src/file_browser/state.rs` | 200-300 | Tree state utilities |
| `src/file_browser/render.rs` | 150-200 | Entry rendering helpers |
| `src/file_browser/context_menu.rs` | 100 | Context menu builder |

**Total New Code:** ~1300-1650 lines

### 9.2 Files to Modify

| File | Changes | Lines Changed |
|------|---------|---------------|
| `Cargo.toml` | Add project, fs, worktree, git, file_icons deps | +10 |
| `src/main.rs` | Initialize project, register file browser | +20 |
| `src/ui/workspace.rs` | Add file_browser_pane, event subscription | +100 |
| `src/ui/workspace_config.rs` | Add FileBrowserPersistedState | +30 |
| `src/terminal/pane.rs` | Add spawn_terminal_with_directory method | +10 |

**Total Modified Code:** ~170 lines

### 9.3 Detailed File Breakdown

#### `src/file_browser/mod.rs` (~50 lines)

```rust
mod pane;
mod state;
mod render;
mod context_menu;

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
```

#### `src/file_browser/pane.rs` (~800-1000 lines)

**Sections:**
1. Imports and type definitions (50 lines)
2. FileBrowserPane struct (40 lines)
3. FileBrowserPersistedState struct (30 lines)
4. EditState, ClipboardEntry enums (20 lines)
5. Lifecycle methods (new, set_active_workspace) (60 lines)
6. Tree management (update_visible_entries, build_flattened_tree) (150 lines)
7. Selection methods (select_entry, toggle_marked) (80 lines)
8. Expand/collapse methods (120 lines)
9. File operation handlers (new_file, rename, delete, copy/paste, etc.) (200 lines)
10. Inline editing (confirm_edit, cancel_edit, validate_filename) (80 lines)
11. Context menu (deploy_context_menu) (40 lines)
12. Event emitter, Focusable, Render implementations (150 lines)

#### `src/file_browser/state.rs` (~200-300 lines)

**Sections:**
1. build_flattened_tree (100 lines)
2. Binary search utilities (30 lines)
3. Auto-fold logic (50 lines)
4. Depth calculation (20 lines)
5. Tests (50 lines)

#### `src/file_browser/render.rs` (~150-200 lines)

**Sections:**
1. EntryDetails struct (30 lines)
2. render_entry function (80 lines)
3. Git status helpers (40 lines)
4. Icon mapping (30 lines)

#### `src/file_browser/context_menu.rs` (~100 lines)

**Sections:**
1. build_context_menu function (80 lines)
2. Menu item helpers (20 lines)

---

## 10. Build Sequence (Wave-Based Implementation)

### Wave 1: Foundation (2-3 hours)

**Goal:** Basic tree rendering, no interactions

- [ ] Add Zed crates: project, worktree, git, fs, editor, file_icons (and any UI helpers used by context menus) to Cargo.toml
- [ ] Create `src/file_browser/mod.rs` with action definitions
- [ ] Create `src/file_browser/state.rs` with build_flattened_tree stub
- [ ] Create `src/file_browser/pane.rs` with minimal FileBrowserPane
  - [ ] Struct definition with project, state fields
  - [ ] new() constructor
  - [ ] Empty render() returning placeholder
- [ ] Verify `cargo build` succeeds
- [ ] Initialize project in `src/main.rs`
- [ ] Add file_browser_pane to WorkspaceView
- [ ] Run app, verify placeholder renders in left pane

**Checkpoint:** App runs, left pane shows "File Browser" placeholder

### Wave 2: Tree Rendering (2-3 hours)

**Goal:** Display static tree with icons, no interactions

- [ ] Implement build_flattened_tree in state.rs
  - [ ] Traverse worktree depth-first
  - [ ] Include all entries (filter hidden files only if global setting enabled)
  - [ ] Return Vec<GitEntry>
- [ ] Create `src/file_browser/render.rs`
  - [ ] EntryDetails struct
  - [ ] render_entry function with icons
  - [ ] Git status color mapping
- [ ] Update FileBrowserPane.render()
  - [ ] Call build_flattened_tree
  - [ ] Use uniform_list with processor
  - [ ] Render each entry via render_entry
- [ ] Test: Tree displays with correct icons and git status colors

**Checkpoint:** Tree renders with files/folders, icons, git status indicators

### Wave 3: Expand/Collapse (1-2 hours)

**Goal:** Interactive tree with expand/collapse

- [ ] Add expanded_dir_ids to FileBrowserRuntimeState
- [ ] Implement binary search utilities in state.rs
  - [ ] is_expanded
  - [ ] expand_dir
  - [ ] collapse_dir
- [ ] Update build_flattened_tree to respect expanded_dir_ids
- [ ] Implement toggle_expanded in pane.rs
- [ ] Add mouse click handler for expand/collapse icon
- [ ] Add keyboard handlers (arrows, enter)
- [ ] Test: Click chevron expands/collapses directory
- [ ] Test: Keyboard navigation works

**Checkpoint:** Tree expand/collapse functional via mouse and keyboard

### Wave 4: Selection (1 hour)

**Goal:** Single and multi-selection with visual feedback

- [ ] Add selection, marked_entries to FileBrowserRuntimeState
- [ ] Implement select_entry, toggle_marked in pane.rs
- [ ] Update render_entry to highlight selected/marked entries
- [ ] Add mouse click handlers with modifier detection
- [ ] Add keyboard handlers (up/down arrows move selection)
- [ ] Emit FileBrowserPaneEvent::SelectionChanged
- [ ] Test: Click selects entry
- [ ] Test: Cmd+click toggles mark
- [ ] Test: Shift+click range selection
- [ ] Test: Keyboard navigation moves selection

**Checkpoint:** Selection works via mouse and keyboard

### Wave 5: Context Menu (1 hour)

**Goal:** Right-click context menu with stub actions

- [ ] Create `src/file_browser/context_menu.rs`
- [ ] Implement build_context_menu
- [ ] Add deploy_context_menu to pane.rs
- [ ] Add mouse right-click handler
- [ ] Add subscription cleanup on dismiss
- [ ] Implement stub action handlers (log only)
- [ ] Test: Right-click shows menu
- [ ] Test: Select action logs message
- [ ] Test: Click outside dismisses menu

**Checkpoint:** Context menu displays and dismisses correctly

### Wave 6: File Operations (2-3 hours)

**Goal:** New file, new folder, delete with confirmation

- [ ] Add filename_editor entity to FileBrowserPane
- [ ] Implement new_file action
  - [ ] Set edit_state
  - [ ] Focus filename_editor
  - [ ] Render inline editor
- [ ] Implement confirm_edit
  - [ ] Validate filename
  - [ ] Call project.create_entry()
  - [ ] Clear edit_state
  - [ ] Update tree
- [ ] Implement new_directory (same pattern)
- [ ] Implement delete action
  - [ ] Show confirmation dialog
  - [ ] Call project.delete_entry()
  - [ ] Update tree
- [ ] Test: New file creates file
- [ ] Test: New folder creates folder
- [ ] Test: Delete removes file/folder
- [ ] Test: Validation rejects invalid names

**Checkpoint:** New file, new folder, delete working

### Wave 7: Copy/Paste, Rename (1-2 hours)

**Goal:** Copy/cut/paste and rename operations

- [ ] Add clipboard field to FileBrowserPane
- [ ] Implement copy action (store in clipboard)
- [ ] Implement cut action (store + mark as cut)
- [ ] Implement paste action
  - [ ] Call project.copy_entry() or project.move_entry()
  - [ ] Update tree
- [ ] Implement rename action (reuse inline editing)
- [ ] Test: Copy/paste duplicates file
- [ ] Test: Cut/paste moves file
- [ ] Test: Rename changes filename

**Checkpoint:** Copy/paste, rename working

### Wave 8: Additional Actions (1 hour)

**Goal:** Copy path, reveal in finder, collapse all

- [ ] Implement copy_path (absolute path to clipboard)
- [ ] Implement copy_relative_path (relative to workspace root)
- [ ] Implement reveal_in_finder (open system file manager)
- [ ] Implement collapse_all (clear expanded_dir_ids)
- [ ] Test: Copy path works
- [ ] Test: Reveal in finder opens Finder
- [ ] Test: Collapse all collapses tree

**Checkpoint:** All context menu actions working

### Wave 9: "Open in Terminal" Integration (30 min)

**Goal:** Open folder in terminal

- [ ] Add OpenInTerminal action to file_browser actions
- [ ] Implement open_in_terminal in pane.rs
  - [ ] Emit FileBrowserPaneEvent::OpenInTerminal(path)
- [ ] Add event subscription in WorkspaceView
- [ ] Handle event: spawn terminal with cwd = path
- [ ] Test: Right-click folder -> Open in Terminal -> terminal opens with correct cwd

**Checkpoint:** "Open in Terminal" functional

### Wave 10: State Persistence (1 hour)

**Goal:** Save/restore expanded dirs, scroll position, selection

- [ ] Add FileBrowserPersistedState to WorkspaceConfig
- [ ] Implement save_state in FileBrowserPane
- [ ] Implement load_state in FileBrowserPane
- [ ] Call save_state on expand/collapse/selection changes
- [ ] Call load_state on workspace switch
- [ ] Hook into WorkspaceConfigStore auto-save
- [ ] Pause file watching when pane hidden; on show, debounce a full refresh (~500ms)
- [ ] Test: Expand folders, switch workspace, switch back -> folders still expanded
- [ ] Test: Scroll position persists

**Checkpoint:** File browser state persists across workspace switches

### Wave 11: Auto-Fold & Polish (1 hour)

**Goal:** Auto-fold single-child directories, final polish

- [ ] Implement auto-fold logic in build_flattened_tree
- [ ] Add unfolded_dir_ids to FileBrowserRuntimeState (override auto-fold)
- [ ] Add unfold_directory, fold_directory actions
- [ ] Test: Single-child directories auto-fold
- [ ] Test: Unfold action disables auto-fold
- [ ] Final testing pass
- [ ] Performance testing with large directory (1000+ files)
- [ ] Clean up debug logging
- [ ] Documentation pass

**Checkpoint:** Sprint complete, all features working

---

## 11. Critical Implementation Details

### 11.1 Binary Search for Performance

**Why:** O(log n) lookup vs O(n) for HashSet with sorted Vec

```rust
// Maintain sorted order
fn expand_dir(entry_id: ProjectEntryId, expanded_dir_ids: &mut Vec<ProjectEntryId>) {
    if let Err(ix) = expanded_dir_ids.binary_search(&entry_id) {
        expanded_dir_ids.insert(ix, entry_id);
    }
}

// Fast lookup
fn is_expanded(entry_id: ProjectEntryId, expanded_dir_ids: &[ProjectEntryId]) -> bool {
    expanded_dir_ids.binary_search(&entry_id).is_ok()
}
```

### 11.2 Background Tree Computation

**Why:** Prevent UI blocking on large directories

```rust
fn update_visible_entries(&mut self, window: &mut Window, cx: &mut Context<Self>) {
    let project = self.project.clone();
    let expanded_ids = self.get_active_state().expanded_dir_ids.clone();
    let workspace_id = self.active_workspace_id.clone();

    self.update_tree_task = cx.spawn_in(window, |this, cx| async move {
        // Build tree in background
        let entries = build_flattened_tree_async(&project, &expanded_ids).await;

        // Update UI on main thread
        this.update(cx, |this, cx| {
            if let Some(state) = this.state_by_workspace.get_mut(&workspace_id) {
                state.visible_entries = entries;
            }
            cx.notify();
        }).ok();
    });
}
```

**Visibility throttling:**
- When the pane is hidden, pause project watching (or ignore updates).
- When the pane becomes visible, trigger a full refresh with a ~500ms debounce to collapse bursts of FS events.

### 11.3 Filename Validation

```rust
fn validate_filename(&self, filename: &str, is_dir: bool) -> ValidationState {
    if filename.is_empty() {
        return ValidationState::Error("Filename cannot be empty".to_string());
    }

    if filename.contains('/') || filename.contains('\\') {
        return ValidationState::Error("Filename cannot contain / or \\".to_string());
    }

    if filename.starts_with('.') {
        return ValidationState::Warning("Filename starts with . (hidden file)".to_string());
    }

    // Check if file already exists
    if self.entry_exists(filename) {
        return ValidationState::Error(format!("{} already exists", if is_dir { "Folder" } else { "File" }));
    }

    ValidationState::None
}
```

### 11.4 Git Status Color Mapping

```rust
fn git_status_color(status: GitSummary, cx: &Context) -> Color {
    let theme = cx.theme();

    if status.added > 0 {
        Color::Success  // Green for new files
    } else if status.modified > 0 {
        Color::Modified  // Yellow for modified
    } else if status.deleted > 0 {
        Color::Error  // Red for deleted
    } else if status.conflicts > 0 {
        Color::Conflict  // Purple for conflicts
    } else {
        Color::Default  // Normal text color
    }
}
```

### 11.5 Auto-Fold Logic

```rust
fn should_auto_fold(entry: &GitEntry, worktree: &Worktree, unfolded_ids: &HashSet<ProjectEntryId>) -> bool {
    // Don't auto-fold if explicitly unfolded
    if unfolded_ids.contains(&entry.id) {
        return false;
    }

    // Only fold directories
    if entry.kind != EntryKind::Directory {
        return false;
    }

    // Get children
    let children: Vec<_> = worktree.child_entries(entry.id).collect();

    // Auto-fold if exactly one child and it's a directory
    children.len() == 1 && children[0].kind == EntryKind::Directory
}
```

### 11.6 Hidden Files Filtering

- Default: show hidden files/folders.
- If global setting `file_browser.hide_hidden_files` is enabled, filter entries whose filename starts with `.`.
- Apply filtering during `build_flattened_tree` so selection/expand logic operates only on visible entries.

---

## 12. Testing Strategy

### 12.1 Unit Tests

**File:** `src/file_browser/state.rs`

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_binary_search_expand() {
        let mut expanded = vec![1, 3, 5];
        expand_dir(2, &mut expanded);
        assert_eq!(expanded, vec![1, 2, 3, 5]);
    }

    #[test]
    fn test_binary_search_is_expanded() {
        let expanded = vec![1, 3, 5];
        assert!(is_expanded(3, &expanded));
        assert!(!is_expanded(2, &expanded));
    }

    #[test]
    fn test_collapse_dir() {
        let mut expanded = vec![1, 2, 3, 5];
        collapse_dir(2, &mut expanded);
        assert_eq!(expanded, vec![1, 3, 5]);
    }

    #[test]
    fn test_auto_fold_single_child() {
        // Test auto-fold logic
    }
}
```

### 12.2 Integration Tests

**File:** `tests/file_browser_tests.rs`

```rust
#[gpui::test]
async fn test_file_browser_expand_collapse(cx: &mut TestAppContext) {
    // Create test workspace with file tree
    // Verify expand/collapse updates visible entries
}

#[gpui::test]
async fn test_new_file_creation(cx: &mut TestAppContext) {
    // Create file browser
    // Trigger new_file action
    // Verify file created on disk
}

#[gpui::test]
async fn test_rename_validation(cx: &mut TestAppContext) {
    // Create file browser with existing file
    // Attempt rename to invalid name
    // Verify validation error
}
```

### 12.3 Manual Testing Checklist

- [ ] Tree displays files and folders correctly
- [ ] Expand/collapse works via mouse click
- [ ] Expand/collapse works via keyboard (arrows, enter)
- [ ] Selection works (click, cmd+click, shift+click)
- [ ] Keyboard navigation (up/down arrows)
- [ ] Right-click shows context menu
- [ ] New file creates file with validation
- [ ] New folder creates folder with validation
- [ ] Rename changes filename with validation
- [ ] Delete removes file/folder with confirmation
- [ ] Copy/paste duplicates file
- [ ] Cut/paste moves file
- [ ] Copy path copies to clipboard
- [ ] Reveal in finder opens system file manager
- [ ] Open in terminal spawns terminal with correct cwd
- [ ] Collapse all collapses entire tree
- [ ] Git status indicators show correctly
- [ ] Git status colors match git state
- [ ] Auto-fold single-child directories
- [ ] State persists across workspace switches
- [ ] Hide file browser, modify files, re-show -> full refresh within ~500ms
- [ ] Performance acceptable with 1000+ files
- [ ] No crashes or errors in logs

---

## 13. Performance Considerations

### 13.1 Virtualization (uniform_list)

**Expected Performance:**
- **10 files:** Instantaneous
- **100 files:** < 16ms frame time
- **1000 files:** < 16ms frame time (virtualized)
- **10,000 files:** < 50ms initial load, < 16ms scrolling

**Measurement:** Profile with `RUST_LOG=trace` and large test directory

### 13.2 Tree Computation

**Expected Performance:**
- **100 files:** < 1ms
- **1000 files:** < 10ms
- **10,000 files:** < 100ms (acceptable for background task)

**Strategy:** If > 100ms, show loading indicator during computation

### 13.3 Git Status Updates

**Expected Performance:**
- **Initial load:** < 500ms for typical project
- **Incremental updates:** < 50ms per file change

**Handled by:** Zed's project crate (battle-tested)

---

## 14. Security Considerations

### 14.1 Filename Validation

**Threats:**
- Path traversal (../../etc/passwd)
- Shell injection ($(rm -rf /))
- Invalid characters (NUL bytes)

**Mitigations:**
```rust
fn validate_filename(filename: &str) -> Result<()> {
    // Reject path separators
    if filename.contains('/') || filename.contains('\\') {
        return Err(anyhow!("Filename cannot contain path separators"));
    }

    // Reject parent directory references
    if filename.contains("..") {
        return Err(anyhow!("Filename cannot contain .."));
    }

    // Reject control characters
    if filename.chars().any(|c| c.is_control()) {
        return Err(anyhow!("Filename cannot contain control characters"));
    }

    Ok(())
}
```

### 14.2 File Operations

**Threats:**
- Symlink attacks (create symlink outside workspace)
- Race conditions (file created between validation and operation)

**Mitigations:**
- Use Zed's `Fs` abstraction (handles symlinks safely)
- Validate paths are within workspace root
- Use project API (has built-in safety checks)

---

## 15. Future Enhancements (Deferred)

### 15.1 Find in Folder

**Requirements:**
- Search infrastructure (grep, ripgrep integration)
- Search results UI
- Search settings (case sensitive, regex, etc.)

**Estimated Effort:** 4-6 hours

### 15.2 File History

**Requirements:**
- Git diff UI component
- Commit history view
- Integration with git crate

**Estimated Effort:** 6-8 hours

### 15.3 Compare Files

**Requirements:**
- Diff viewer component
- Side-by-side or inline diff
- Navigation between changes

**Estimated Effort:** 8-10 hours

### 15.4 Drag-and-Drop File Moving

**Requirements:**
- DragMoveEvent handlers
- Visual feedback during drag
- Drop target validation
- File move via project API

**Estimated Effort:** 3-4 hours

**Note:** Basic drag-and-drop included in Sprint 3.1, advanced features (multi-file, external files) deferred

---

## 16. Open Questions & Decisions Needed

### Q1: File Icons

**Question:** Use Zed's file_icons crate or implement custom icon mapping?

**Recommendation:** Use Zed's file_icons crate
- Consistent with Zed UI
- Supports 100+ file types
- Includes folder icons
- Maintained by Zed team

**Decision:** Use file_icons crate

### Q2: Drag-and-Drop Scope

**Question:** Include drag-and-drop in Sprint 3.1 or defer to Sprint 3.2?

**Recommendation:** Include basic drag-and-drop in Sprint 3.1
- Core feature, not "nice-to-have"
- Pattern already in Zed (copy implementation)
- ~2 hours additional effort

**Decision:** Include drag-and-drop in Sprint 3.1

### Q3: Multi-Folder Projects

**Question:** Support multiple workspace roots like Zed?

**Recommendation:** Defer to future (not v1.0)
- TerminalG = single workspace root (simpler model)
- Terminal pane assumes single root
- Can add later without breaking changes

**Decision:** Single root only for v1.0

### Q4: Hidden Files Default

**Question:** Show hidden files by default?

**Recommendation:** Show by default, allow hiding via global setting
- Matches terminal-centric workflows (dotfiles visible)
- Simple to implement as a global toggle
- Avoids “where did my file go?” confusion

**Decision:** Show hidden files by default; add a global app setting to hide

---

## 17. Success Criteria

### Sprint 3.1 Complete When:

**Functional Requirements:**
- [ ] Tree view displays files and folders
- [ ] Expand/collapse works (mouse + keyboard)
- [ ] Selection works (single + multi)
- [ ] Git status indicators display
- [ ] Context menu shows all actions
- [ ] New file/folder creates entries
- [ ] Rename changes filename
- [ ] Delete removes entries
- [ ] Copy/paste duplicates entries
- [ ] Cut/paste moves entries
- [ ] Copy path copies to clipboard
- [ ] Reveal in finder opens system file manager
- [ ] Open in terminal spawns terminal with cwd
- [ ] Collapse all collapses tree
- [ ] Auto-fold single-child directories
- [ ] State persists across workspace switches

**Non-Functional Requirements:**
- [ ] All unit tests passing
- [ ] All integration tests passing
- [ ] Manual testing checklist complete
- [ ] No clippy warnings
- [ ] Performance acceptable (< 16ms frame time with 1000 files)
- [ ] No crashes or errors
- [ ] Code documented
- [ ] Design document updated with lessons learned

**Integration Requirements:**
- [ ] WorkspaceView integrates file browser pane
- [ ] Terminal integration works ("Open in Terminal")
- [ ] Workspace config saves/loads file browser state
- [ ] Theme colors applied correctly

---

## 18. References

### Zed Source Files (Reference)

- `/Users/randlee/Documents/github/zed/crates/project_panel/src/project_panel.rs` - Main implementation
- `/Users/randlee/Documents/github/zed/crates/project_panel/src/project_panel_settings.rs` - Settings
- `/Users/randlee/Documents/github/zed/crates/project/src/project.rs` - Project API
- `/Users/randlee/Documents/github/zed/crates/worktree/src/worktree.rs` - Worktree types
- `/Users/randlee/Documents/github/zed/crates/git/src/repository.rs` - Git integration

### TerminalG Architecture Docs

- `/Users/randlee/Documents/github/terminalg/docs/ARCHITECTURE.md` - System architecture
- `/Users/randlee/Documents/github/terminalg/docs/MASTER-PLAN.md` - Phase 3 plan
- `/Users/randlee/Documents/github/terminalg/.claude/skills/rust-development/guidelines.txt` - Rust guidelines
- `/Users/randlee/Documents/github/terminalg/.claude/skills/rust-development/gpui-zed-guidelines.md` - GPUI guidelines

### Related Sprint Designs

- `/Users/randlee/Documents/github/terminalg/docs/sprints/phase-1-sprint-5-design.md` - Workspace tabs
- `/Users/randlee/Documents/github/terminalg/docs/sprints/phase-2-sprint-2-design.md` - Terminal integration

---

**Document Status:** Complete
**Next Steps:** Review design -> Create worktree -> Begin Wave 1 implementation
**Estimated Total Effort:** 8-10 hours
