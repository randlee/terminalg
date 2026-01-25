# TerminalG Architecture

**Version:** 1.1
**Last Updated:** 2026-01-25
**Target GPUI Version:** Zed v0.220.3
**Dependency Strategy:** See `docs/architecture/zed-reuse-strategy.md`

---

## 1. System Overview

TerminalG is a GPU-accelerated terminal application built on GPUI, combining terminal emulation with integrated artifact viewing in a unified workspace interface.

### 1.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│              TerminalGApp (GPUI App)                        │
│  - App-wide settings management                             │
│  - Workspace configuration system                           │
│  - Theme system                                             │
│  - Global keybindings                                       │
└──────────────────────┬──────────────────────────────────────┘
                       │
                       │ spawns
                       ▼
┌─────────────────────────────────────────────────────────────┐
│          WorkspaceView (Root Container)                     │
│  ┌─────────────────────────────────────────────────────┐   │
│  │  Workspace Tabs (Top) - [Work][Project][Test]      │   │
│  └─────────────────────────────────────────────────────┘   │
│  ┌───────────┬─────────────────────┬───────────────────┐   │
│  │           │                     │                   │   │
│  │  File/    │    TerminalPane     │  DocumentViewer   │   │
│  │  Folder   │    (Zed-based)      │   Pane            │   │
│  │  Browser  │                     │                   │   │
│  │           │  [Tab1][Tab2][+]    │  [Doc1][Doc2][+]  │   │
│  │           │                     │                   │   │
│  │ [Hide]    │      [Hide]         │     [Hide]        │   │
│  └───────────┴─────────────────────┴───────────────────┘   │
│  - Workspace config (per-workspace state persistence)       │
│  - Pane visibility management                               │
│  - Tab management (terminals and documents)                 │
└─────────────────────────────────────────────────────────────┘
```

**Key Features:**
- **Three co-equal panes** (file browser, terminal, document viewer)
- **Workspace tabs at top** (switch entire context)
- **Terminal tabs at bottom** (multiple terminals per workspace)
- **Document tabs at bottom** (multiple documents per workspace)
- **Independent pane visibility** (hide/show with button)

---

## 2. Technology Stack

### 2.1 Core Dependencies

| Component | Technology | Version | Source | License | Rationale |
|-----------|-----------|---------|--------|---------|-----------|
| UI Framework | GPUI | v0.220.3 | git (Zed tag) | Apache-2.0 | GPU-accelerated, proven in Zed |
| Terminal Emulation | alacritty_terminal | 0.12+ | crates.io | Apache-2.0 | Battle-tested VTE implementation |
| Async Runtime | smol | 2.0 | crates.io | Apache-2.0 | Lightweight, used by Zed |
| Configuration | serde/serde_json | 1.0 | crates.io | MIT | Standard Rust serialization |
| Logging | tracing | 0.1 | crates.io | MIT | Structured logging |
| File Watching | notify | 6.0 | crates.io | MIT | Settings hot-reload |

**Phase 2+ Dependencies (Zed Crates):**

| Component | Zed Crate | Version | License | Phase |
|-----------|-----------|---------|---------|-------|
| Terminal Core | `terminal` | v0.220.3 | GPL-3.0 | Phase 2 |
| Settings | `settings` | v0.220.3 | GPL-3.0 | Phase 2 |
| Theme | `theme` | v0.220.3 | GPL-3.0 | Phase 2 |
| UI Components | `ui` | v0.220.3 | GPL-3.0 | Phase 2 |
| Markdown | `markdown` | v0.220.3 | GPL-3.0 | Phase 4 |

**Note:** Phase 2+ dependencies require TerminalG to be GPL-3.0 licensed. See `docs/architecture/zed-reuse-strategy.md` for details and future flexibility options.

### 2.2 Dependency Strategy

**GPUI Version Management:**
- Use git dependency with tag pinning
- Pin to stable Zed releases (currently v0.220.3)
- Upgrade deliberately after testing
- See `docs/architecture/gpui-integration.md` for details

**Rust Version:**
- Minimum: 1.93.0
- Target: Latest stable

---

## 3. Component Architecture

### 3.1 TerminalGApp (Application Root)

**Responsibilities:**
- GPUI app initialization
- Settings loading and watching
- Theme management
- Global action registration
- Window lifecycle management

**Key Patterns:**
```rust
struct TerminalGApp {
    settings: Arc<Settings>,
    theme: Arc<Theme>,
}

impl TerminalGApp {
    fn new(cx: &mut AppContext) -> Self {
        // Load settings, register actions, initialize theme
    }

    fn open_workspace(&mut self, cx: &mut AppContext) {
        cx.open_window(|cx| WorkspaceView::new(cx))
    }
}
```

**Files:** `src/main.rs`

---

### 3.2 WorkspaceView (Root UI Container)

**Responsibilities:**
- Workspace-level tab management (top tabs)
- Three-pane layout management
- Pane visibility controls (hide/show buttons)
- Terminal tab management (bottom of terminal pane)
- Document tab management (bottom of document viewer pane)
- Workspace configuration persistence
- Pane layout and resizing
- Status bar rendering
- Keyboard shortcuts

**State:**
```rust
struct WorkspaceView {
    workspace_id: String,
    active_workspace_tab: usize,
    workspace_tabs: Vec<WorkspaceTab>,

    // Pane visibility
    file_browser_visible: bool,
    terminal_visible: bool,
    document_viewer_visible: bool,

    // Pane sizes
    pane_ratios: [f32; 3],  // Ratios for 3 panes

    // Terminal tabs
    terminal_tabs: Vec<TerminalTab>,
    active_terminal_tab: usize,

    // Document tabs
    document_tabs: Vec<DocumentTab>,
    active_document_tab: usize,

    // Workspace config
    config: WorkspaceConfig,
}
```

**Files:** `src/ui/workspace.rs`, `src/ui/workspace_config.rs`

---

### 3.3 TerminalPane (Terminal Emulation)

**Integration Strategy:** Use Zed's `terminal` crate as a git dependency, write custom view.

**Approach:**
- Use Zed's `terminal` crate as git dependency (NOT copied/vendored)
- Terminal crate wraps `alacritty_terminal` with PTY management
- Write custom `TerminalPane` view for TerminalG's artifact-focused UI
- Use Zed's `settings` and `theme` crates (required by terminal crate)
- Add URL recognition and clicking on top

**See:** `docs/architecture/zed-reuse-strategy.md` Section 3.5 for complete dependency strategy.

**Responsibilities:**
- PTY lifecycle management (all platforms: macOS, Linux, Windows)
- Terminal grid rendering (alacritty_terminal → GPUI)
- Input handling (keyboard, mouse → PTY)
- Output processing (PTY → terminal grid → render)
- Scrollback buffer management
- Copy/paste, selection
- Search in terminal output
- Shell integration
- Working directory tracking
- **NEW: URL recognition and clicking**

**Critical Integration:**

**PTY Threading Model:**
1. Spawn smol task for PTY read loop
2. Use mpsc channel to send output to GPUI thread
3. Call `cx.notify()` to trigger re-render

```rust
cx.spawn(|view, mut cx| async move {
    let mut pty_reader = pty.reader();
    loop {
        let data = pty_reader.read().await?;
        cx.update(|view, cx| {
            view.terminal.process_bytes(&data);
            cx.notify();
        })?;
    }
}).detach();
```

**Grid Rendering:**
1. Iterate alacritty_terminal grid cells
2. Group consecutive cells with same style
3. Render as styled GPUI `div()` elements

**Files:** `src/terminal/pane.rs`, `src/terminal/pty.rs`

**Details:** See `docs/architecture/gpui-integration.md` Section 2.3

---

### 3.4 FileBrowserPane (File Navigation)

**Responsibilities:**
- Display file/folder tree (rooted at workspace folder)
- Handle directory expand/collapse
- File selection (open in document viewer)
- File operations (refresh, navigate)
- Sync with workspace configuration

**State:**
```rust
struct FileBrowserPane {
    root_path: PathBuf,
    tree: FileTree,
    selected: Option<PathBuf>,
    expanded_dirs: HashSet<PathBuf>,
}
```

**Files:** `src/ui/file_browser.rs`

---

### 3.5 DocumentViewerPane (Artifact Display)

**Responsibilities:**
- Manage multiple document tabs (bottom tabs)
- Switch between viewer types based on file type
- Detect file types automatically
- Coordinate viewer lifecycle
- Integration with file browser (open file → new tab)

**Viewer Types:**
- **MarkdownViewer:** Parse and render markdown (Phase 4)
- **MarkdownEditor:** Edit markdown with preview (Phase 5)
- **ImageViewer:** Display images with zoom/pan
- **HTMLViewer:** (Future) Inline HTML rendering

**State:**
```rust
struct DocumentViewerPane {
    tabs: Vec<DocumentTab>,
    active_tab: usize,
}

struct DocumentTab {
    path: PathBuf,
    viewer_type: ViewerType,
    viewer: Box<dyn Viewer>,
}
```

**Files:** `src/viewer/pane.rs`, `src/viewer/markdown.rs`, `src/viewer/image.rs`, `src/viewer/editor.rs`

---

### 3.6 WorkspaceConfig (Configuration Management)

**Responsibilities:**
- Load/save workspace configuration
- Auto-save on state changes
- Manage workspace-specific settings

**Workspace Configuration:**
```rust
struct WorkspaceConfig {
    workspace_id: String,
    root_path: PathBuf,

    // Pane visibility
    file_browser_visible: bool,
    terminal_visible: bool,
    document_viewer_visible: bool,

    // Pane sizes
    pane_ratios: [f32; 3],

    // Open terminals
    terminal_tabs: Vec<TerminalTabConfig>,

    // Open documents
    document_tabs: Vec<DocumentTabConfig>,

    // Active selections
    active_terminal_tab: usize,
    active_document_tab: usize,
}
```

**Storage:**
- Near-term: platform config dir, e.g. macOS `~/Library/Application Support/terminalg/workspaces/<workspace-id>.json`, Linux `~/.config/terminalg/workspaces/<workspace-id>.json`, Windows `%APPDATA%\\terminalg\\workspaces\\<workspace-id>.json`
- Workspace-local: `.terminalg/<workspace>.json` in project repos

**Indexing:**
- App settings track all available workspaces and their config paths

**Files:** `src/settings/workspace.rs`

---

## 4. Data Flow

### 4.1 Settings Flow

```
settings.json (disk)
    ↓ load
SettingsStore (Rust struct)
    ↓ cx.set_global()
GPUI Global State
    ↓ cx.global::<Settings>()
Views (apply settings)
```

**Hot Reload:**
1. File watcher detects settings.json change
2. SettingsStore reloads from disk
3. Update global state
4. Call `cx.notify()` on all views
5. Views apply new settings on next render

**Schema Versioning:**
- Settings include a schema version field
- On load, migrate older versions to the latest schema
- Unknown fields are ignored for forward compatibility
- Migration failures surface a user-facing error and fall back to defaults

**Details:** See `docs/architecture/settings-system.md`

---

### 4.2 Theme Flow

```
Theme::by_name(settings.ui.theme)
    ↓
Theme struct (colors)
    ↓ cx.set_global()
GPUI Global State
    ↓ cx.global::<Theme>()
Views (apply theme colors)
```

**Theme Switching:**
1. Update settings.json theme field
2. Settings reload (see above)
3. Load new theme by name
4. Update global theme
5. Re-render all views with new colors

**Files:** `src/theme/mod.rs`

---

### 4.3 Terminal Data Flow

```
User Keyboard Input
    ↓
GPUI Event Handler
    ↓
PTY Write (non-blocking)
    ↓
Shell Process
    ↓
PTY Read (async task)
    ↓
alacritty_terminal.process_bytes()
    ↓
Terminal Grid Update
    ↓
cx.notify() → GPUI Re-render
    ↓
TerminalPane.render() → Grid → GPUI Elements
```

**Details:** See `docs/architecture/gpui-integration.md` Section 2.2

---

## 5. Module Structure

```
src/
├── main.rs                    # App entry point, TerminalGApp
├── settings/
│   ├── mod.rs                 # SettingsStore, load/save/reload
│   ├── terminal.rs            # TerminalSettings struct
│   └── ui.rs                  # UiSettings struct
├── theme/
│   └── mod.rs                 # Color, Theme, built-in themes
├── terminal/
│   ├── mod.rs                 # Terminal component
│   ├── pane.rs                # TerminalPane (GPUI view)
│   ├── pty.rs                 # PTY management
│   └── grid_renderer.rs       # alacritty → GPUI rendering
├── ui/
│   ├── mod.rs                 # UI module exports
│   ├── workspace.rs           # WorkspaceView
│   ├── tab_bar.rs             # Tab management UI
│   ├── file_browser.rs        # FileBrowserPane
│   └── status_bar.rs          # Status bar component
└── viewer/
    ├── mod.rs                 # Viewer module exports
    ├── pane.rs                # ViewerPane (switches viewers)
    ├── markdown.rs            # MarkdownViewer
    └── image.rs               # ImageViewer
```

---

## 6. State Management

### 6.1 Global State (GPUI)

Stored in GPUI's global context, accessible from any view:

```rust
cx.set_global(settings);   // Store
let settings = cx.global::<SettingsStore>();  // Retrieve
```

**Global State:**
- `SettingsStore` - Application settings
- `Theme` - Current theme colors

### 6.2 View-Local State

Each view manages its own state:

**WorkspaceView:**
- Active tab index
- Tab list
- Pane sizes

**TerminalPane:**
- PTY handle
- Terminal grid (alacritty_terminal::Term)
- Scrollback position

**ViewerPane:**
- Active viewer type
- Current file path
- Viewer-specific state (zoom level, scroll position)

---

## 7. Event Handling

### 7.1 Keyboard Input

**Terminal:**
```rust
div().on_key_down(cx.listener(|this, event, cx| {
    this.handle_key_input(event, cx);
}))
```

**Global Shortcuts:**
- Registered in TerminalGApp
- Handled before view-specific handlers
- Examples: Cmd+T (new tab), Cmd+W (close tab)

### 7.2 Mouse Input

**Clickable URLs:**
1. Detect URLs in terminal output (regex)
2. Store URL → screen region mapping
3. Handle click events on regions
4. Open URL in browser or copy to clipboard

**Pane Resizing:**
1. Render drag handle between panes
2. Handle mouse down/move/up events
3. Update pane size state
4. Persist to settings

---

## 8. Threading Model

### 8.1 GPUI Main Thread

**Runs:**
- All GPUI rendering
- Event handling
- View updates
- UI logic

**Must not block:** Keep operations fast (<16ms for 60 FPS)

### 8.2 Background Tasks (smol)

**PTY Reading:**
- Spawned as smol task
- Reads from PTY asynchronously
- Sends data to GPUI thread via channel

**File Watching:**
- Spawned as smol task
- Watches settings.json for changes
- Triggers reload on GPUI thread

**File Operations:**
- Directory scanning (file browser)
- File loading (markdown, images)

**Pattern:**
```rust
cx.spawn(|view, mut cx| async move {
    let result = expensive_operation().await;
    cx.update(|view, cx| {
        view.apply_result(result);
        cx.notify();
    })
}).detach();
```

---

## 9. Error Handling

### 9.1 Strategy

**User-Facing Errors:**
- Display in UI (status bar, modal, inline)
- Log with `tracing::error!()`
- Provide actionable messages

**Internal Errors:**
- Log with `tracing::warn!()` or `tracing::error!()`
- Graceful degradation where possible
- Fail safely (don't crash UI)

### 9.2 Critical Failures

**PTY Failure:**
- Show error in terminal pane
- Allow user to retry or close tab

**Settings Load Failure:**
- Use defaults
- Warn user in UI
- Continue operation

**File Load Failure:**
- Show error in viewer pane
- Provide option to retry or select different file

---

## 10. Performance Considerations

### 10.1 Terminal Rendering

**Optimization:**
- Dirty region tracking (only re-render changed cells)
- Batch rendering (group cells with same style)
- Texture caching where possible (GPUI)

**Target:**
- Handle 100k lines of scrollback
- Render at 60 FPS even with rapid output

### 10.2 File Browser

**Optimization:**
- Lazy loading (only scan visible directories)
- Virtual scrolling for large directories
- Cache directory contents

### 10.3 Viewer Performance

**Markdown:**
- Parse once, cache result
- Re-parse only on file change

**Images:**
- Load and decode off-thread
- Cache decoded image data
- Implement progressive loading for large images

---

## 11. Platform-Specific Details

### 11.1 macOS (Primary Development Platform)

**PTY:** POSIX via Zed terminal (openpty, fork, execv)
**Config Path:** `~/Library/Application Support/terminalg/`
**Shell:** Default to `$SHELL` or `/bin/zsh`

### 11.2 Linux

**PTY:** POSIX via Zed terminal (same as macOS)
**Config Path:** `~/.config/terminalg/`
**Shell:** Default to `$SHELL` or `/bin/bash`
**Display:** Support X11 and Wayland (GPUI handles)

### 11.3 Windows

**PTY:** ConPTY API via Zed terminal (Windows 10+)
**Config Path:** `%APPDATA%\terminalg\`
**Shell:** Default to PowerShell or `cmd.exe`

**Note:** Phase 1 validates the bootstrap app window on macOS, Linux, and Windows. Full cross-platform terminal functionality arrives in Phase 2 after Zed terminal integration. Primary development and testing on macOS, CI/testing on all platforms.

---

## 12. Security Considerations

### 12.1 Terminal Security

- Do not log sensitive terminal output
- Respect terminal privacy modes
- Do not persist terminal history by default

### 12.2 File Operations

- Validate file paths (prevent directory traversal)
- Respect file permissions
- Handle symlinks safely

### 12.3 Settings

- Validate settings on load (schema validation)
- Use safe defaults for invalid values
- Sanitize user-provided paths

---

## 13. Testing Strategy

### 13.1 Unit Tests

- Settings serialization/deserialization
- Theme color calculations
- URL regex matching
- File path validation

### 13.2 Integration Tests

- PTY creation and communication
- Terminal grid rendering
- Settings load/save/reload
- File browser directory scanning

### 13.3 Manual Testing

- Full UI workflows
- Cross-platform compatibility
- Performance under load
- Edge cases (large files, rapid output)

---

## 14. Build and Deployment

### 14.1 Build Process

```bash
# Development
cargo build

# Release (optimized)
cargo build --release
```

### 14.2 Binary Distribution

**macOS:**
- Create app bundle (.app)
- Sign and notarize (future)
- DMG distribution (future)

**Linux:**
- Distribute binary + .desktop file
- Package for distributions (deb, rpm) (future)

**Windows:**
- MSI installer (future)

---

## 15. Monitoring and Diagnostics

### 15.1 Logging

**Levels:**
- `TRACE` - Detailed execution flow
- `DEBUG` - Debugging information
- `INFO` - General informational messages
- `WARN` - Unexpected but handled situations
- `ERROR` - Error conditions

**Usage:**
```bash
RUST_LOG=info cargo run
RUST_LOG=terminalg=debug cargo run
```

### 15.2 Performance Profiling

- Use GPUI's built-in profiler (when available)
- `cargo flamegraph` for CPU profiling
- Memory profiling with `valgrind` (Linux)

---

## 16. Design Principles

1. **Leverage Zed's Patterns** - Use proven approaches from Zed codebase
2. **Performance First** - GPU acceleration, efficient rendering
3. **User Control** - Configurable, keyboard-driven
4. **Fail Gracefully** - Never crash, always provide feedback
5. **Platform Native** - Respect platform conventions
6. **Minimal Dependencies** - Only add what's necessary
7. **Clear Separation** - Modular components, clear boundaries

---

## 17. Future Architecture Considerations

### 17.1 Plugin System

- Define plugin API boundary
- Sandboxed plugin execution
- Plugin discovery and loading

### 17.2 MCP Layer

- Protocol definition
- Message routing between components
- Structured data parsing from terminal output

### 17.3 Remote Sessions

- Connect to remote terminal sessions
- Sync state across machines
- Collaborative features

---

## 18. References

### 18.1 Supporting Documents

- **Zed Reuse Strategy:** `docs/architecture/zed-reuse-strategy.md` - Source of truth for Zed crate dependencies
- **GPUI Integration:** `docs/architecture/gpui-integration.md`
- **Settings System:** `docs/architecture/settings-system.md` (Phase 1 only - see zed-reuse-strategy.md for Phase 2 migration)

### 18.2 External Resources

- **GPUI Source:** https://github.com/zed-industries/zed/tree/v0.220.3/crates/gpui
- **Zed Terminal:** https://github.com/zed-industries/zed/tree/v0.220.3/crates/terminal
- **alacritty_terminal:** https://docs.rs/alacritty_terminal
- **Zed Architecture:** Study Zed's workspace and terminal implementations

### 18.3 Related Documents

- **Requirements:** `docs/REQUIREMENTS.md`
- **Master Plan:** `docs/MASTER-PLAN.md`

---

**Document Status:** ✅ Approved
**Next Review:** After Phase 2 complete (update with terminal architecture lessons)
