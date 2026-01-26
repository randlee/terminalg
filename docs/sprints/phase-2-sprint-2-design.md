# Phase 2 Sprint 2 Design: Terminal Integration

**Version:** 1.1
**Created:** 2026-01-25
**Updated:** 2026-01-25
**Status:** Sprint 2.1 ✅ | Sprint 2.2 ✅ | Sprint 2.3 Pending
**Branch:** `feature/sprint-2-terminal-integration`
**Worktree:** `/Users/randlee/Documents/github/terminalg-worktrees/feature/sprint-2-terminal-integration`
**PR:** https://github.com/randlee/terminalg/pull/14

---

## 1. Overview

Sprint 2 integrates Zed's terminal crate into TerminalG, replacing custom settings/theme systems with Zed's mature implementations.

### Key Deliverables
1. PTY spawning and management
2. Terminal input/output handling
3. GPU-accelerated terminal rendering with theme colors
4. Basic scrollback buffer
5. Runtime-only terminal sessions (no PTY restore on restart)

### Sprint Breakdown
| Sprint | Focus | Duration |
|--------|-------|----------|
| **2.1** | Add Zed Dependencies & Migrate Settings/Theme | 6-8 hrs |
| **2.2** | Terminal Pane Integration | 8-12 hrs |
| **2.3** | URL Recognition & Clicking | 6-10 hrs |

---

## 2. Architecture Decisions

### 2.1 Zed Dependencies Strategy
- Use Zed crates as **git dependencies** (NOT copied/vendored)
- Pin to `tag = "v0.220.3"` matching existing GPUI version
- **License Impact:** TerminalG becomes GPL-3.0-or-later

### 2.2 Settings Migration
- Replace custom `SettingsStore` with Zed's `SettingsStore`
- Create `TerminalGSettings` implementing Zed's `Settings` trait
- WorkspaceConfigStore remains separate (Sprint 1.5 implementation preserved)

### 2.3 Theme Migration
- Replace custom `Theme` with Zed's `Theme` and `ThemeRegistry`
- Use Zed's built-in themes initially
- WorkspaceView updated to use `cx.theme()` instead of `cx.global::<Theme>()`

### 2.4 Terminal Integration
- Create custom `TerminalPane` wrapping Zed's `Terminal` struct
- Integrate with existing WorkspaceView three-pane layout
- PTY lifecycle fully managed by Zed's terminal crate
- Terminal sessions persist only while the app is running (no session restore on restart)
- Terminal history restore is a nice-to-have, not required in Sprint 2
- Future: track Claude sessions run inside terminals for optional restore (out of scope)

---

## 3. Component Architecture

```
TerminalGApp (main.rs)
├── Zed SettingsStore (global) ← replaces custom
├── Zed ThemeRegistry (global) ← replaces custom
├── WorkspaceConfigStore (existing) ← unchanged
└── WorkspaceView (existing from Sprint 1.5)
    ├── WorkspaceTabBar (existing)
    ├── FileBrowserPlaceholder (existing)
    ├── TerminalPane (NEW - Sprint 2.2)
    │   ├── Zed Terminal (wrapped)
    │   ├── Terminal tab management
    │   └── PTY lifecycle
    └── DocumentViewerPlaceholder (existing)
```

---

## 4. Implementation Map

### Sprint 2.1: Add Zed Dependencies & Migrate Settings/Theme

**Files to Create:**
| File | Lines | Purpose |
|------|-------|---------|
| `src/settings_adapter.rs` | ~200 | Bridge to Zed settings, `TerminalGSettings` |
| `src/theme_adapter.rs` | ~150 | Theme initialization, color utilities |

**Files to Modify:**
| File | Changes |
|------|---------|
| `Cargo.toml` | Add Zed git dependencies (settings, theme, ui, collections) |
| `src/main.rs` | Replace custom settings/theme with Zed systems |
| `src/ui/workspace.rs` | Update theme access to `cx.theme()` |
| `src/settings/` | Mark deprecated |
| `src/theme/` | Mark deprecated |

**Cargo.toml Additions:**
```toml
[dependencies]
settings = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
theme = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
ui = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
collections = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }

[package]
license = "GPL-3.0-or-later"
```

### Sprint 2.2: Terminal Pane Integration

**Files to Create:**
| File | Lines | Purpose |
|------|-------|---------|
| `src/terminal/pane.rs` | ~400 | `TerminalPane` wrapping Zed Terminal |
| `src/terminal/tab.rs` | ~150 | `TerminalTab` state management |

**Files to Modify:**
| File | Changes |
|------|---------|
| `Cargo.toml` | Add `terminal` git dependency |
| `src/terminal/mod.rs` | Replace placeholder with module exports |
| `src/ui/workspace.rs` | Replace Terminal placeholder with `TerminalPane` |
| `src/ui/workspace_config.rs` | Add terminal tab persistence |

---

## 5. Data Flow

### Terminal I/O Flow
```
User Keystroke
    ↓ GPUI Event
TerminalPane.handle_key_input()
    ↓
Zed Terminal.input()
    ↓
PTY Write (non-blocking)
    ↓
Shell Process
    ↓
PTY Read (async via Zed)
    ↓
Zed Terminal processes bytes
    ↓
cx.notify() → Re-render
    ↓
TerminalPane.render() → GPU
```

---

## 6. Build Sequence

### Phase 1: Zed Dependencies (2 hours) ✅ COMPLETE
- [x] Update Cargo.toml with Zed git dependencies
- [x] Update license to GPL-3.0-or-later
- [x] Run `cargo check` - resolve conflicts
- [x] Document dependency versions

### Phase 2: Settings Migration (2-3 hours) ✅ COMPLETE
- [x] Create `src/settings_adapter.rs`
- [x] Initialize Zed SettingsStore in main.rs
- [x] Test settings loading
- [x] Old settings/ kept for reference (not deprecated yet)

### Phase 3: Theme Migration (2-3 hours) ✅ COMPLETE
- [x] Create `src/theme_adapter.rs`
- [x] Initialize ThemeRegistry in main.rs
- [x] Update WorkspaceView to use `cx.theme()`
- [x] Test theme application

### Phase 4: Terminal Pane Structure (3-4 hours) ✅ COMPLETE
- [x] Add terminal git dependency
- [x] Create `src/terminal/pane.rs` (~360 lines)
- [x] Create `src/terminal/tab.rs` (~70 lines)
- [x] Implement basic TerminalPane
- [x] Test terminal spawns and renders

### Phase 5: Terminal Tab Management (2-3 hours) ✅ COMPLETE
- [x] Implement multiple terminal tabs
- [x] Add tab switching UI (bottom tabs)
- [x] Wire up tab creation/closing
- [x] Test multiple terminals

### Phase 6: WorkspaceView Integration (2-3 hours) ✅ COMPLETE
- [x] Replace Terminal placeholder with TerminalPane
- [x] Connect terminal to workspace
- [x] Test visibility toggle
- [x] Workspace root used as terminal working directory

### Phase 7: State Persistence (1-2 hours) - DEFERRED
- [ ] Update workspace_config.rs for terminal tabs
- [ ] Save/restore working directories
- [ ] Test persistence across restarts
- [x] Confirmed: no PTY/session restore on restart (runtime-only)

---

## 7. Implementation Patterns

### Settings Adapter
```rust
// src/settings_adapter.rs
use settings::{Settings, SettingsStore};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TerminalGSettings {
    pub terminal: TerminalSettings,
    pub ui: UiSettings,
}

impl Settings for TerminalGSettings {
    const KEY: Option<&'static str> = Some("terminalg");
    // ...
}
```

### Terminal Pane
```rust
// src/terminal/pane.rs
use terminal::Terminal as ZedTerminal;
use gpui::{div, prelude::*, Render};

pub struct TerminalPane {
    terminals: Vec<TerminalTab>,
    active_tab: usize,
}

impl Render for TerminalPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex().flex_col().size_full()
            .child(self.render_terminal(cx))
            .child(self.render_tabs(cx))
    }
}
```

---

## 8. Risk Mitigation

| Risk | Mitigation |
|------|-----------|
| Zed API incompatibilities | Pin to v0.220.3, study existing examples |
| Settings migration breaks workspace config | Keep WorkspaceConfigStore separate |
| Complex PTY threading | Let Zed handle all PTY lifecycle |
| Platform-specific PTY issues | Rely on Zed's cross-platform support |

---

## 9. Success Criteria

### Sprint 2.1 Complete When: ✅ COMPLETE
- [x] Cargo builds with Zed dependencies (no warnings)
- [x] Zed SettingsStore initialized
- [x] Zed ThemeRegistry initialized
- [x] WorkspaceView uses Zed theme colors
- [x] All existing tests pass (60/60)
- [x] App launches without errors

### Sprint 2.2 Complete When: ✅ COMPLETE
- [x] Terminal spawns in TerminalPane
- [x] Terminal renders shell prompt and output
- [x] Terminal accepts keyboard input
- [x] Multiple terminal tabs functional
- [x] Terminal integrates with WorkspaceView
- [x] Terminal visibility toggle works
- [x] Terminal tabs persist while app is running (no session restore on restart)
- [x] No crashes or memory leaks
- [x] 60/60 tests passing, clippy clean

---

## 10. References

- **Zed Reuse Strategy:** `docs/architecture/zed-reuse-strategy.md`
- **Sprint 1.5 Design:** `docs/sprints/phase-1-sprint-5-design.md`
- **Zed Terminal Crate:** `github.com/zed-industries/zed/crates/terminal/`
- **Zed Settings Crate:** `github.com/zed-industries/zed/crates/settings/`

---

**Document Status:** Ready for Implementation
