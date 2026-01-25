# Zed/GPUI Reuse Strategy

**Version:** 1.0
**Last Updated:** 2025-01-25
**Status:** Draft

---

## 1. Overview

This document is the **source of truth** for TerminalG's dependency on Zed crates. It defines which crates we use, how we use them, and our strategy for long-term maintenance.

### 1.1 Zed Repository Reference

| Property | Value |
|----------|-------|
| Repository | `https://github.com/zed-industries/zed` |
| Local Path | `../zed/` (relative to TerminalG repo) |
| Crates Path | `../zed/crates/` |
| Current Version | `v0.220.3` (git tag) |
| Version Policy | Pin to stable release tags, upgrade deliberately |

**Important:** The local Zed checkout at `../zed/` MUST be at tag `v0.220.3`. AI agents exploring Zed patterns must use this pinned version, not `main`, to avoid confusion from API differences.

### 1.2 License Implications

| Crate Category | License | Implication |
|----------------|---------|-------------|
| GPUI, collections, util, refineable | Apache-2.0 | No restrictions |
| settings, theme, terminal, markdown, ui | GPL-3.0 | TerminalG must be GPL-3.0 |

**Decision:** TerminalG is licensed under **GPL-3.0-or-later** due to dependencies on Zed's application crates.

---

## 2. Architecture for Future Flexibility

To preserve the option to remove GPL dependencies in the future, TerminalG uses a **clean architecture** with abstraction layers:

```
┌─────────────────────────────────────────────────────────┐
│  TerminalG Core (our code, our copyright)               │
│  - Application logic, state management                  │
│  - Abstract traits for terminal, theme, settings        │
│  - Can be relicensed if GPL deps removed                │
└─────────────────────┬───────────────────────────────────┘
                      │ trait implementations
┌─────────────────────┴───────────────────────────────────┐
│  Zed Adapter Layer (our code, GPL-3.0 required)         │
│  - Implements traits using Zed crates                   │
│  - Isolated, replaceable                                │
└─────────────────────┬───────────────────────────────────┘
                      │ dependencies
┌─────────────────────┴───────────────────────────────────┐
│  Zed Crates (Zed's code, GPL-3.0)                       │
│  - gpui, terminal, theme, settings, markdown, ui        │
└─────────────────────────────────────────────────────────┘
```

**Future escape path:** Replace Zed Adapter Layer with implementations using non-GPL alternatives (see Section 6).

---

## 3. Subsystem Strategy

### 3.1 GPUI (UI Framework)

| Property | Value |
|----------|-------|
| Zed Crate | `gpui` |
| Crate Path | `../zed/crates/gpui/` |
| License | Apache-2.0 |
| Strategy | **Use as git dependency** |
| Justification | Standalone framework, zero Zed coupling, well-documented |

**Usage:**
```toml
gpui = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
```

**Notes:**
- GPUI is designed as a standalone GPU-accelerated UI framework
- Published to crates.io as `gpui` v0.2.x
- No dependency on Zed application logic
- Safe to use without GPL implications

---

### 3.2 Settings

| Property | Value |
|----------|-------|
| Zed Crates | `settings`, `settings_json`, `settings_macros` |
| Crate Path | `../zed/crates/settings/` |
| License | GPL-3.0 |
| Strategy | **Use as git dependency** |
| Current State | Custom implementation exists (~150 lines) |
| Decision | Migrate to Zed crates for compatibility |

**Justification:**
- Zed's `terminal` crate depends on `settings` - we cannot use our own
- Zed's settings system is more mature (hot-reload, schema validation, keymaps)
- Migration required when we adopt Zed's terminal crate

**Migration Plan:**
1. Phase 1 (Sprint 1.5): Keep custom settings for workspace config
2. Phase 2 (Terminal Integration): Adopt Zed settings, wrap workspace config

**Zed Settings Features:**
- `SettingsStore` with registration pattern
- JSON schema validation
- Hot-reload on file change
- Keymap file handling
- Platform-specific paths

---

### 3.3 Theme

| Property | Value |
|----------|-------|
| Zed Crate | `theme` |
| Crate Path | `../zed/crates/theme/` |
| License | GPL-3.0 |
| Strategy | **Use as git dependency** |
| Current State | Custom implementation exists (~100 lines) |
| Decision | Migrate to Zed crates for compatibility |

**Justification:**
- Zed's `terminal` crate depends on `theme` for colors
- Zed's theme system includes font management, syntax highlighting
- Terminal colors must match theme system

**Migration Plan:**
1. Phase 1: Keep custom theme for basic UI
2. Phase 2: Adopt Zed theme, provide TerminalG-specific themes

**Zed Theme Features:**
- `ThemeRegistry` for loading/switching themes
- ANSI color mappings for terminal
- Syntax highlighting colors
- UI color tokens

---

### 3.4 Logging

| Property | Value |
|----------|-------|
| Zed Crate | `zlog` (optional) |
| Crate Path | `../zed/crates/zlog/` |
| License | GPL-3.0 |
| Strategy | **Use standard Rust ecosystem** |
| Current State | `tracing` + `tracing-subscriber` |
| Decision | Keep current approach |

**Justification:**
- `zlog` is a thin wrapper around standard `log` crate
- No benefit to switching
- Avoids unnecessary GPL dependency

**Current Implementation:**
```rust
tracing_subscriber::fmt()
    .with_env_filter(EnvFilter::from_default_env())
    .init();
```

---

### 3.5 Terminal

| Property | Value |
|----------|-------|
| Zed Crates | `terminal` (core), `terminal_view` (UI) |
| Crate Paths | `../zed/crates/terminal/`, `../zed/crates/terminal_view/` |
| License | GPL-3.0 |
| Strategy | **Use `terminal` as dependency, write custom view** |

**Core Terminal (`terminal`):**
- Wraps `alacritty_terminal` for VT emulation
- PTY management (Unix + Windows ConPTY)
- Depends on: `settings`, `theme`, `gpui`
- **Reusable** - no Zed UI coupling

**Terminal View (`terminal_view`):**
- Integrates with Zed's workspace, editor, project
- Depends on: `workspace`, `editor`, `project`, `language`
- **Not reusable** - too Zed-specific

**Decision:**
```toml
terminal = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
# terminal_view - DO NOT USE, write custom
```

**TerminalG Implementation:**
- Use `terminal` crate for PTY and emulation
- Write custom `TerminalPane` view for our artifact-focused UI
- Integrate with our workspace, not Zed's

---

### 3.6 File Browser

| Property | Value |
|----------|-------|
| Zed Crate | `project_panel` |
| Crate Path | `../zed/crates/project_panel/` |
| License | GPL-3.0 |
| Strategy | **Write custom** |

**Justification:**
- `project_panel` depends on: `project`, `workspace`, `editor`, `client`, `git`
- Extreme coupling to Zed's collaboration and project model
- TerminalG needs simpler file tree without collaboration

**TerminalG Implementation:**
- Custom `FileBrowserPane` using GPUI
- Use `fs` crate for file operations (if needed)
- Simple tree view, no multi-project support

**Patterns to Reference:**
- `../zed/crates/project_panel/src/project_panel.rs` - UI patterns
- Uniform list rendering for performance
- Keyboard navigation

---

### 3.7 Markdown

| Property | Value |
|----------|-------|
| Zed Crates | `markdown` (renderer), `markdown_preview` (UI) |
| Crate Paths | `../zed/crates/markdown/`, `../zed/crates/markdown_preview/` |
| License | GPL-3.0 |
| Strategy | **Use `markdown` as dependency, write custom preview** |

**Markdown Renderer (`markdown`):**
- Parses markdown to GPUI elements
- Uses `pulldown-cmark` internally
- Depends on: `gpui`, `theme`, `language` (for syntax highlighting)
- **Reusable** - clean separation

**Markdown Preview (`markdown_preview`):**
- Integrates with Zed's workspace/editor
- Split-view with editor sync
- **Not reusable** - different use case

**Decision:**
```toml
markdown = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
# markdown_preview - DO NOT USE, write custom
```

---

### 3.8 Workspace / Tabs

| Property | Value |
|----------|-------|
| Zed Crate | `workspace` |
| Crate Path | `../zed/crates/workspace/` |
| License | GPL-3.0 |
| Strategy | **Write custom** |

**Justification:**
- `workspace` is ~9000 lines, tightly coupled to collaboration
- Depends on: `project`, `client`, `call`, `session`, `db`
- TerminalG workspace model is fundamentally different

**TerminalG Model:**
- One project per workspace (folder-based)
- Three panes: file browser, terminal, artifact viewer
- No collaboration, no multi-project

**Patterns to Reference:**
- `../zed/crates/workspace/src/pane.rs` - Tab management
- `../zed/crates/workspace/src/dock.rs` - Panel trait
- `../zed/crates/ui/src/components/tab_bar.rs` - Tab UI

---

### 3.9 UI Components

| Property | Value |
|----------|-------|
| Zed Crates | `ui`, `menu` |
| Crate Paths | `../zed/crates/ui/`, `../zed/crates/menu/` |
| License | GPL-3.0 (`ui`), Apache-2.0 (`menu`) |
| Strategy | **Use as git dependencies** |

**UI Components (`ui`):**
- Pre-built GPUI widgets: Button, Input, List, Tab, etc.
- Depends on: `gpui`, `theme`, `settings`
- **Reusable** - generic components

**Menu (`menu`):**
- Context menus, dropdowns
- Depends on: `gpui` only
- **Reusable** - Apache-2.0

**Decision:**
```toml
ui = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
menu = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
```

---

### 3.10 Git Integration (Future)

| Property | Value |
|----------|-------|
| Zed Crates | `git`, `git_ui` |
| Crate Paths | `../zed/crates/git/`, `../zed/crates/git_ui/` |
| License | GPL-3.0 |
| Strategy | **Evaluate for future phases** |

**Notes:**
- Not required for MVP
- `git` crate provides diff, status, blame
- `git_ui` provides visual components
- Plan to support git views post-MVP

---

### 3.11 Panel Trait

| Property | Value |
|----------|-------|
| Location | `../zed/crates/workspace/src/dock.rs` |
| License | GPL-3.0 |
| Strategy | **Adopt pattern, write custom trait** |

**Zed's Panel Trait:**
```rust
pub trait Panel: Focusable + EventEmitter<PanelEvent> + Render + Sized {
    fn persistent_name() -> &'static str;
    fn position(&self, ...) -> DockPosition;
    fn size(&self, ...) -> Pixels;
    fn icon(&self, ...) -> Option<IconName>;
    fn toggle_action(&self) -> Box<dyn Action>;
    // ... zoom, active state, remote_id (collaboration)
}
```

**TerminalG's Panel Trait (simplified):**
```rust
pub trait Panel: Focusable + Render {
    fn name(&self) -> &'static str;
    fn icon(&self) -> Option<IconName>;
    fn can_hide(&self) -> bool;
}
```

---

## 4. Dependency Summary

### 4.1 Crates to Use (Git Dependencies)

```toml
[dependencies]
# Framework (Apache-2.0)
gpui = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }

# Infrastructure (GPL-3.0) - Required for terminal/markdown
settings = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
theme = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }

# Application (GPL-3.0)
terminal = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
markdown = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
ui = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
menu = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }

# Utilities (Apache-2.0)
collections = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
util = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
```

### 4.2 Crates to Write Custom

| Component | Reason |
|-----------|--------|
| Workspace | Different model (no collaboration) |
| Terminal View | Different UI (artifact pane integration) |
| File Browser | Too coupled to Zed's project system |
| Markdown Preview | Different context (artifact viewer) |
| Settings UI | May use Zed's if generic enough |

### 4.3 Dependency Graph

```
TerminalG
├── gpui (Apache-2.0)
├── terminal (GPL-3.0)
│   ├── alacritty_terminal
│   ├── settings (GPL-3.0)
│   └── theme (GPL-3.0)
├── markdown (GPL-3.0)
│   ├── pulldown-cmark
│   ├── theme
│   └── language (for syntax)
├── ui (GPL-3.0)
│   ├── gpui
│   └── theme
├── menu (Apache-2.0)
│   └── gpui
├── collections (Apache-2.0)
└── util (Apache-2.0)
```

---

## 5. Version Upgrade Procedure

### 5.1 When to Upgrade

- New Zed stable release with features we need
- Security fixes in dependencies
- Bug fixes affecting TerminalG

### 5.2 Upgrade Process

1. **Identify new version**
   ```bash
   cd ../zed
   git fetch --tags
   git tag -l 'v0.*' | sort -V | tail -10
   ```

2. **Review changes**
   ```bash
   git log v0.220.3..v0.221.0 --oneline -- crates/terminal crates/gpui crates/settings crates/theme
   ```

3. **Update Cargo.toml**
   - Change all Zed git dependencies to new tag
   - Run `cargo update`

4. **Test thoroughly**
   ```bash
   cargo build
   cargo test
   cargo clippy
   # Manual testing of terminal, themes, settings
   ```

5. **Document changes**
   - Update this document's "Current Version"
   - Note any breaking changes in CHANGELOG

### 5.3 Breaking Change Handling

If Zed introduces breaking changes:
1. Check if change affects crates we use
2. Adapt our code to new API
3. If too disruptive, stay on older version until ready

---

## 6. Future: Removing GPL Dependencies

If business requirements change and GPL-3.0 becomes unacceptable:

### 6.1 Replacement Options

| Current (GPL) | Replacement (non-GPL) | Effort |
|---------------|----------------------|--------|
| `terminal` | Direct `alacritty_terminal` (Apache-2.0) + custom PTY | High |
| `theme` | Custom theme system (already have basic) | Medium |
| `settings` | Custom settings (already have basic) | Medium |
| `markdown` | Direct `pulldown-cmark` (MIT) + custom GPUI render | Medium |
| `ui` | Custom components or other GPUI widget library | High |

### 6.2 Preserved Components

- **GPUI** - Apache-2.0, no change needed
- **TerminalG core logic** - Our code, can relicense
- **Custom views** - Our code, can relicense

### 6.3 Estimated Effort

To fully remove GPL dependencies: **2-4 weeks** of development
- Terminal integration without Zed's wrapper
- Theme system rewrite
- UI component library

---

## 7. File Ownership Headers

For code we write, use this header to document ownership:

```rust
// Copyright 2024-2025 [Author/Organization]
// SPDX-License-Identifier: GPL-3.0-or-later
//
// This file is part of TerminalG.
// Original work by the authors. GPL-3.0 license applies due to
// dependencies on Zed crates. Authors retain copyright and may
// relicense if GPL dependencies are removed.
```

---

## 8. References

### 8.1 Zed Crate Documentation

| Crate | Key Files |
|-------|-----------|
| GPUI | `../zed/crates/gpui/README.md` |
| Settings | `../zed/crates/settings/src/settings_store.rs` |
| Theme | `../zed/crates/theme/src/registry.rs` |
| Terminal | `../zed/crates/terminal/src/terminal.rs` |
| Panel Trait | `../zed/crates/workspace/src/dock.rs` (lines 96-128) |

### 8.2 Related TerminalG Documents

- `docs/ARCHITECTURE.md` - Overall system architecture
- `docs/architecture/gpui-integration.md` - GPUI patterns
- `docs/architecture/settings-system.md` - Current settings (pre-migration)

---

**Document Status:** Draft - Pending Review
**Next Review:** After Phase 2 (Terminal Integration) begins
