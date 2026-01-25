# Phase 1 Sprint 5: Workspace Tabs & Configuration

**Duration:** 4-6 hours
**Dependencies:** Sprint 1.4 (GPUI Bootstrap) ✅
**Parallel:** No (final sprint in Phase 1)
**Status:** Not Started

> **Note:** This sprint uses TerminalG's custom settings and theme systems (Phase 1 approach).
> Phase 2 will migrate to Zed's `settings` and `theme` crates as git dependencies.
> See `docs/architecture/zed-reuse-strategy.md` for the complete dependency strategy.
>
> The WorkspaceView implementation is **custom by design** - Zed's workspace crate is too
> tightly coupled to collaboration features. Our workspace will remain custom through MVP.

---

## 1. Objectives

- Create workspace configuration system (save/load state)
- Implement workspace tab bar UI (top tabs)
- Implement workspace switching
- Create placeholder pane layout (3 empty panes)
- Add pane visibility controls (hide/show buttons)
- Auto-save workspace config on changes
- Default workspace: file browser + terminal visible

---

## 2. Prerequisites

- [x] GPUI window opens and renders correctly (Sprint 1.4)
- [x] Settings system working (Sprint 1.2)
- [x] Theme system working (Sprint 1.3)
- [x] Project compiles without errors

---

## 3. Pattern Analysis

### 3.1 Existing TerminalG Patterns

**Settings Pattern (src/settings/mod.rs):**
- `SettingsStore` struct with `settings: Settings` and `settings_path: PathBuf`
- `Global` trait implementation for GPUI global state
- Platform-specific config paths via `dirs::config_dir()`
- JSON serialization with serde
- `load_from_file()`, `save()`, `reload()` methods
- Error handling with `anyhow::Result`

**Theme Pattern (src/theme/mod.rs):**
- `Global` trait for GPUI global state
- `by_name()` lookup with fallback to default
- Colors stored as simple structs

**Main.rs Pattern:**
- Load settings/theme before GPUI app start
- Store in global state via `cx.set_global()`
- Access via `cx.global::<T>()` in views
- Window close handling with `cx.on_window_closed()`

### 3.2 Zed Patterns to Adapt

**Tab Bar (crates/ui/src/components/tab_bar.rs):**
- Horizontal flex container: start | scrollable tabs | end
- Fixed height with DynamicSpacing
- Tab components with selected state

**State Persistence:**
- 200ms debounce for auto-save
- Serialize workspace state to storage
- Restore on app launch

**Pane Layout:**
- Conditional rendering with `when()` / `child_when()`
- Flex layout for 3-pane structure

---

## 4. Architecture Design

### 4.1 Component Hierarchy

```
TerminalGApp
└── WorkspaceView (replaces EmptyView)
    ├── WorkspaceTabBar (top: workspace tabs)
    │   ├── Tab("default") [active]
    │   └── Tab("debug")
    │
    └── WorkspaceContent (h_flex with 3 panes)
        ├── FileBrowserPlaceholder (conditional)
        │   └── HideButton
        ├── TerminalPlaceholder (always visible)
        │   └── HideButton
        └── DocumentViewerPlaceholder (conditional)
            └── HideButton
```

### 4.2 State Management

- **Global state:** `SettingsStore`, `Theme` (existing)
- **View state:** `WorkspaceView` struct holds workspace config
- **No new globals needed** - workspace config in view state

### 4.3 Settings Locations

- **User settings:** `~/.config/terminalg/settings.json`
- **Workspace settings:** `.terminalg/workspace.json` or `.terminalg/workspace-<name>.json` (repo-local, typically committed)

---

## 5. Data Structures

### 5.1 WorkspaceConfig (src/ui/workspace_config.rs)

```rust
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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

    /// Pane width ratios (file_browser, terminal, document_viewer)
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
```

### 5.2 WorkspaceConfigStore

```rust
pub struct WorkspaceConfigStore {
    config: WorkspacesConfig,
    config_path: PathBuf,
}

impl WorkspaceConfigStore {
    pub fn new() -> Result<Self>;
    pub fn load() -> Result<Self>;
    pub fn save(&self) -> Result<()>;
    pub fn config(&self) -> &WorkspacesConfig;
    pub fn config_mut(&mut self) -> &mut WorkspacesConfig;
    pub fn active_workspace(&self) -> &WorkspaceConfig;
    pub fn active_workspace_mut(&mut self) -> &mut WorkspaceConfig;
    pub fn switch_workspace(&mut self, index: usize);
    fn get_config_path() -> Result<PathBuf>;
}
```

### 5.3 WorkspaceView (src/ui/workspace.rs)

```rust
use gpui::{prelude::*, Model};
use crate::ui::workspace_config::{WorkspaceConfigStore, WorkspaceConfig};

pub struct WorkspaceView {
    /// Workspace configuration store
    config_store: WorkspaceConfigStore,

    /// Debounce timer for auto-save (task handle)
    save_task: Option<Task<()>>,
}

impl WorkspaceView {
    pub fn new(cx: &mut Context<Self>) -> Self;

    /// Switch to workspace at index
    fn switch_workspace(&mut self, index: usize, cx: &mut Context<Self>);

    /// Toggle pane visibility
    fn toggle_pane(&mut self, pane: PaneType, cx: &mut Context<Self>);

    /// Schedule debounced save (200ms)
    fn schedule_save(&mut self, cx: &mut Context<Self>);

    /// Render workspace tab bar
    fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement;

    /// Render three-pane content area
    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement;

    /// Render a single placeholder pane
    fn render_pane(&self, pane: PaneType, visible: bool, cx: &mut Context<Self>) -> impl IntoElement;
}

#[derive(Debug, Clone, Copy)]
pub enum PaneType {
    FileBrowser,
    Terminal,
    DocumentViewer,
}

impl Render for WorkspaceView {
    fn render(&mut self, window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme_bg_color(theme))
            .child(self.render_tab_bar(cx))
            .child(self.render_content(cx))
    }
}
```

---

## 6. Implementation Map

### 6.1 Files to Create

| File | Purpose | Est. Lines |
|------|---------|------------|
| `src/ui/workspace_config.rs` | Config structs + persistence | ~150 |
| `src/ui/workspace.rs` | WorkspaceView component | ~250 |

### 6.2 Files to Modify

| File | Changes |
|------|---------|
| `src/ui/mod.rs` | Add module exports, remove placeholder |
| `src/main.rs` | Replace `EmptyView` with `WorkspaceView` |

---

## 7. Detailed Build Sequence

### Phase 1: Workspace Config Foundation (1-1.5 hours)

**Tasks:**
- [ ] Create `src/ui/workspace_config.rs`
- [ ] Implement `WorkspaceConfig` struct with serde
- [ ] Implement `WorkspacesConfig` struct with defaults
- [ ] Implement `WorkspaceConfigStore` with load/save
- [ ] Use repo-local path: `.terminalg/workspace.json` or `.terminalg/workspace-<name>.json`
- [ ] Add unit tests for config serialization

**Test checkpoint:**
```bash
cargo test workspace_config
```

### Phase 2: WorkspaceView Structure (1-1.5 hours)

**Tasks:**
- [ ] Create `src/ui/workspace.rs`
- [ ] Implement `WorkspaceView` struct
- [ ] Implement `WorkspaceView::new()` loading config
- [ ] Implement basic `Render` trait (empty render first)
- [ ] Update `src/ui/mod.rs` with exports
- [ ] Wire up in `main.rs` (replace EmptyView)

**Test checkpoint:**
```bash
cargo build && cargo run
# Window should open (may be empty/basic)
```

### Phase 3: Workspace Tab Bar UI (1 hour)

**Tasks:**
- [ ] Implement `render_tab_bar()` method
- [ ] Create tab elements with workspace names
- [ ] Style active tab differently (background color)
- [ ] Add click handlers for tab switching
- [ ] Apply theme colors

**Render pattern:**
```rust
fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.global::<Theme>();
    let workspaces = &self.config_store.config().workspaces;
    let active_idx = self.config_store.config().active_workspace_index;

    div()
        .h(px(40.0))
        .w_full()
        .flex()
        .items_center()
        .bg(tab_bar_bg_color(theme))
        .border_b_1()
        .border_color(border_color(theme))
        .children(
            workspaces.iter().enumerate().map(|(idx, ws)| {
                let is_active = idx == active_idx;
                self.render_tab(idx, &ws.name, is_active, cx)
            })
        )
}

fn render_tab(&self, index: usize, name: &str, is_active: bool, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.global::<Theme>();

    div()
        .id(ElementId::NamedInteger("workspace-tab".into(), index))
        .px_3()
        .py_1()
        .cursor_pointer()
        .when(is_active, |d| d.bg(active_tab_bg(theme)))
        .when(!is_active, |d| d.bg(inactive_tab_bg(theme)))
        .child(name.to_string())
        .on_click(cx.listener(move |this, _, _, cx| {
            this.switch_workspace(index, cx);
        }))
}
```

**Test checkpoint:**
```bash
cargo run
# Should see tabs at top, click should log switch (add tracing)
```

### Phase 4: Three-Pane Layout (1 hour)

**Tasks:**
- [ ] Implement `render_content()` method
- [ ] Create horizontal flex container
- [ ] Render three placeholder panes with colors
- [ ] Apply conditional visibility
- [ ] Add pane labels (File Browser, Terminal, Document Viewer)

**Render pattern:**
```rust
fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let ws = self.config_store.active_workspace();

    div()
        .flex()
        .flex_1()
        .w_full()
        .child_when(ws.file_browser_visible, self.render_pane(PaneType::FileBrowser, cx))
        .child(self.render_pane(PaneType::Terminal, cx)) // Always visible
        .child_when(ws.document_viewer_visible, self.render_pane(PaneType::DocumentViewer, cx))
}

fn render_pane(&self, pane_type: PaneType, cx: &mut Context<Self>) -> impl IntoElement {
    let (label, color) = match pane_type {
        PaneType::FileBrowser => ("File Browser", rgb(0x2D4A6E)),      // Blue-ish
        PaneType::Terminal => ("Terminal", rgb(0x3A3A3A)),             // Gray
        PaneType::DocumentViewer => ("Document Viewer", rgb(0x4A2D6E)), // Purple-ish
    };

    div()
        .flex_1()
        .flex()
        .flex_col()
        .m_1()
        .p_2()
        .bg(color)
        .rounded_md()
        .child(
            div()
                .flex()
                .justify_between()
                .child(div().text_lg().child(label))
                .child(self.render_hide_button(pane_type, cx))
        )
        .child(
            div()
                .flex_1()
                .items_center()
                .justify_center()
                .child(format!("{} Placeholder", label))
        )
}
```

**Test checkpoint:**
```bash
cargo run
# Should see 3 colored panes (or 2 with default config)
```

### Phase 5: Pane Visibility Controls (0.5-1 hour)

**Tasks:**
- [ ] Implement `render_hide_button()` method
- [ ] Add click handlers for hide buttons
- [ ] Implement `toggle_pane()` method
- [ ] Update state and trigger re-render
- [ ] Call `schedule_save()` after toggle

**Render pattern:**
```rust
fn render_hide_button(&self, pane_type: PaneType, cx: &mut Context<Self>) -> impl IntoElement {
    div()
        .id(ElementId::NamedInteger("hide-pane".into(), pane_type as usize))
        .px_2()
        .py_1()
        .cursor_pointer()
        .bg(rgb(0x555555))
        .rounded_sm()
        .text_sm()
        .child("Hide")
        .on_click(cx.listener(move |this, _, _, cx| {
            this.toggle_pane(pane_type, cx);
        }))
}

fn toggle_pane(&mut self, pane_type: PaneType, cx: &mut Context<Self>) {
    let ws = self.config_store.active_workspace_mut();
    match pane_type {
        PaneType::FileBrowser => ws.file_browser_visible = !ws.file_browser_visible,
        PaneType::Terminal => ws.terminal_visible = !ws.terminal_visible,
        PaneType::DocumentViewer => ws.document_viewer_visible = !ws.document_viewer_visible,
    }
    cx.notify();
    self.schedule_save(cx);
}
```

**Test checkpoint:**
```bash
cargo run
# Click "Hide" buttons - panes should disappear/reappear
```

### Phase 6: Auto-Save Implementation (0.5-1 hour)

**Tasks:**
- [ ] Implement `schedule_save()` with 200ms debounce
- [ ] Use GPUI `cx.spawn()` for background task
- [ ] Handle errors gracefully (log, don't crash)
- [ ] Cancel previous save task if new changes come in

**Implementation pattern:**
```rust
fn schedule_save(&mut self, cx: &mut Context<Self>) {
    // Cancel any pending save
    self.save_task.take();

    // Clone config for async save
    let config = self.config_store.config().clone();
    let config_path = self.config_store.config_path().clone();

    // Schedule save after 200ms debounce
    self.save_task = Some(cx.spawn(async move |_, _| {
        smol::Timer::after(std::time::Duration::from_millis(200)).await;

        // Save to disk
        if let Err(e) = save_config_to_file(&config, &config_path) {
            tracing::error!("Failed to save workspace config: {}", e);
        } else {
            tracing::debug!("Workspace config saved");
        }
    }));
}
```

**Test checkpoint:**
```bash
cargo run
# Make changes, close app, reopen - changes should persist
```

### Phase 7: Integration & Polish (0.5-1 hour)

**Tasks:**
- [ ] Run `cargo fmt`
- [ ] Run `cargo clippy -- -D warnings`
- [ ] Run `cargo test`
- [ ] Manual testing (see checklist below)
- [ ] Update docs if needed

---

## 8. Testing

### 8.1 Unit Tests

```rust
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
}
```

### 8.2 Manual Testing Checklist

- [ ] App launches without errors
- [ ] Workspace tabs visible at top
- [ ] "Default" tab is active on first launch
- [ ] Click "Debug" tab switches workspace
- [ ] Tab styling changes to show active state
- [ ] Three panes visible (with default config: 2 visible)
- [ ] Pane colors distinguish each pane
- [ ] "Hide" button visible on each pane
- [ ] Click "Hide" hides the pane
- [ ] Click "Hide" on terminal (middle pane) works
- [ ] Switch workspace shows different visibility
- [ ] Close and reopen app - state persists
- [ ] Check `.terminalg/workspace.json` created in repo root
- [ ] No crashes during normal usage
- [ ] Console shows appropriate log messages

---

## 9. Completion Criteria

### 9.1 Functional Requirements

- [ ] Workspace configuration loads on startup
- [ ] Workspace configuration saves on changes
- [ ] Config file created at `.terminalg/workspace.json`
- [ ] Workspace tabs render at top of window
- [ ] Clicking tabs switches active workspace
- [ ] Active tab has distinct visual style
- [ ] Three placeholder panes render in content area
- [ ] Panes have distinct colors for identification
- [ ] Each pane has a "Hide" button
- [ ] Hide buttons toggle pane visibility
- [ ] Layout automatically adjusts when panes hide/show
- [ ] Changes persist across app restarts
- [ ] Default workspace shows file browser + terminal

### 9.2 Quality Requirements

- [ ] `cargo build` succeeds without warnings
- [ ] `cargo test` passes (100%)
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo fmt -- --check` passes
- [ ] No crashes during normal operation

---

## 10. References

### 10.1 TerminalG Documents

- **Architecture:** `docs/ARCHITECTURE.md` - Section 3.2 (WorkspaceView)
- **Settings Pattern:** `src/settings/mod.rs`
- **GPUI Integration:** `docs/architecture/gpui-integration.md`

### 10.2 Zed Reference (v0.220.3)

- **Tab Bar:** `crates/ui/src/components/tab_bar.rs`
- **Tab Component:** `crates/ui/src/components/tab.rs`
- **Pane Management:** `crates/workspace/src/pane.rs` (lines 369-425, 2597-2670)
- **Persistence Model:** `crates/workspace/src/persistence/model.rs`

### 10.3 External

- **GPUI Docs:** Zed codebase examples
- **Serde:** https://serde.rs/

---

## 11. Risk Mitigation

| Risk | Mitigation |
|------|------------|
| GPUI API changes | Pinned to v0.220.3, tested patterns |
| Complex state management | Simple struct-based state, no globals for workspace |
| Auto-save performance | 200ms debounce, background task |
| Config migration | Schema versioning in JSON (add in future if needed) |

---

## 12. Post-Sprint

**On completion:**
1. Mark Sprint 1.5 complete in MASTER-PLAN.md
2. Run code review (`rust-code-reviewer` agent)
3. Run QA validation (`rust-qa-agent`)
4. Create Phase 1 release (v0.1.0)

**Ready for Phase 2:** Zed Terminal Integration

---

**Document Status:** Ready for Implementation
**Created:** 2026-01-25
**Author:** rust-architect agent + manual review
