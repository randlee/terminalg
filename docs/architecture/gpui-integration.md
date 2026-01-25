# GPUI Integration Architecture

**Status:** Phase 1 - Foundation (Planning)
**Author:** ARCH-RTERM
**Date:** 2026-01-25
**Zed Version Target:** v0.220.3

---

## Executive Summary

This document defines the architectural strategy for integrating GPUI (Zed's GPU-accelerated UI framework) into TerminalG. It covers dependency management, component hierarchy, integration patterns, and risk mitigation.

**Key Decision:** Use git-based tag pinning to Zed's stable releases for reproducible, battle-tested builds.

**Related Documents:**
- `docs/architecture/zed-reuse-strategy.md` - Complete Zed crate dependency strategy (settings, theme, terminal, markdown)
- `docs/architecture/settings-system.md` - Phase 1 custom settings (will migrate to Zed crates in Phase 2)

---

## 1. Dependency Strategy

### 1.1 GPUI Publication Status

**Finding (2025-01-25):**
- GPUI **IS published to crates.io** as `gpui` (v0.2.x)
- Also maintained as workspace crate in Zed monorepo: `crates/gpui`
- Latest crates.io version: **v0.2.2** (Oct 22, 2025)
- Latest stable Zed release: **v0.220.3** (Jan 22, 2025)

**Why we use git tag instead of crates.io:**
- Phase 2 requires Zed's `terminal`, `settings`, and `theme` crates (NOT on crates.io)
- All Zed crates must be from the same version for compatibility
- Git tag pinning ensures version parity across all Zed dependencies

### 1.2 Dependency Specification

**Recommended approach:**

```toml
[dependencies]
gpui = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
```

**Rationale:**
- ✓ **Reproducible builds** - Exact version pinning via git tag
- ✓ **Stability** - Uses tested Zed release, not bleeding-edge main
- ✓ **Reference alignment** - Can study Zed's codebase at same version
- ✓ **Controlled upgrades** - We choose when to update, test compatibility
- ✓ **Clear audit trail** - Git tags map to Zed release notes

### 1.3 Alternative Approaches (Rejected)

| Approach | Pros | Cons | Verdict |
|----------|------|------|---------|
| `branch = "main"` | Latest features | Breaking changes, non-reproducible | ❌ Too unstable |
| `rev = "commit"` | Specific commit | No semantic versioning | ⚠️ Use only for patches |
| Vendored copy | Full control | High maintenance burden | ❌ Not viable long-term |
| Local path | Fast iteration | Not portable | ⚠️ Dev-only |

### 1.4 Upgrade Cadence

**Monitor:** Zed releases (~every 2-4 weeks)
**Evaluate:** New features, bug fixes, breaking changes
**Upgrade triggers:**
1. **Critical bugs** in GPUI affecting TerminalG
2. **New features** we need (text rendering, layout improvements)
3. **Major milestones** (e.g., v0.3.0, v1.0.0)
4. **Security patches** in dependencies

**Process:**
1. Review Zed release notes
2. Check GPUI crate changes (`git diff v0.220.3..v0.221.0 -- crates/gpui`)
3. Test in feature branch
4. Update tag in Cargo.toml
5. Document breaking changes in CHANGELOG.md

---

## 2. GPUI Integration Architecture

### 2.1 Component Hierarchy

```
┌─────────────────────────────────────────────┐
│         TerminalGApp (GPUI App)             │
│  - Settings management                      │
│  - Theme system                             │
│  - Global keybindings                       │
└──────────────────┬──────────────────────────┘
                   │
                   │ spawns
                   ▼
┌─────────────────────────────────────────────┐
│      WorkspaceView (Root Container)         │
│  - Tab management (workspace-level)         │
│  - Layout state (pane sizes, positions)     │
│  - Status bar                               │
└─────┬───────────────────────────────────────┘
      │
      │ renders
      ▼
┌─────────────────────────────────────────────┐
│         Pane Grid (Split Layout)            │
│  ┌──────────────┬───────────────────────┐   │
│  │              │                       │   │
│  │ TerminalPane │   ViewerPane          │   │
│  │              │   (Markdown/Image)    │   │
│  │              │                       │   │
│  ├──────────────┴───────────────────────┤   │
│  │         FileBrowserPane              │   │
│  └──────────────────────────────────────┘   │
└─────────────────────────────────────────────┘
```

### 2.2 Core GPUI Components

#### TerminalGApp
**Responsibilities:**
- GPUI app initialization
- Settings loading/watching
- Theme management
- Global action handling
- Window creation

**GPUI Patterns:**
- Implements `gpui::App` trait (or uses `gpui::App::new()`)
- Registers global actions with `cx.bind_keys()`
- Uses `cx.observe_global()` for settings changes
- Spawns background tasks with `cx.spawn()`

**File:** `src/main.rs`

```rust
// Conceptual structure (not final code)
struct TerminalGApp {
    settings: Arc<Settings>,
    theme: Arc<Theme>,
}

impl TerminalGApp {
    fn new(cx: &mut AppContext) -> Self {
        // Load settings
        // Register actions
        // Initialize theme
        // Spawn settings watcher
    }

    fn open_workspace(&mut self, cx: &mut AppContext) {
        cx.open_window(|cx| WorkspaceView::new(cx))
    }
}
```

#### WorkspaceView
**Responsibilities:**
- Workspace-level tab management
- Pane layout and resizing
- Status bar rendering
- Tab switching logic

**GPUI Patterns:**
- Implements `gpui::View` trait
- Uses `div()` for layout with flex
- Handles resize events
- Manages child view lifecycle

**File:** `src/ui/workspace.rs`

```rust
// Conceptual structure
struct WorkspaceView {
    active_tab: usize,
    tabs: Vec<WorkspaceTab>,
    pane_split_ratio: f32,
}

impl Render for WorkspaceView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .child(self.render_tab_bar(cx))
            .child(self.render_pane_grid(cx))
            .child(self.render_status_bar(cx))
    }
}
```

#### TerminalPane
**Responsibilities:**
- PTY lifecycle management
- Terminal grid rendering (alacritty_terminal integration)
- Input handling (keyboard → PTY)
- Output processing (PTY → terminal grid → GPUI render)

**GPUI Patterns:**
- Custom element rendering (grid cells → styled divs)
- Background task for PTY reading
- Event handlers for keyboard input
- `cx.notify()` on terminal state changes

**File:** `src/terminal/pane.rs`

**Critical integration point:** Bridging alacritty_terminal's grid to GPUI rendering.

```rust
// Conceptual structure
struct TerminalPane {
    pty: Pty,
    terminal: alacritty_terminal::Term,
    grid_renderer: TerminalGridRenderer,
}

impl Render for TerminalPane {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        self.grid_renderer.render_grid(&self.terminal, cx)
    }
}
```

### 2.3 Critical Integration Points

#### Point 1: PTY → GPUI Threading Model

**Challenge:** PTY I/O is blocking, GPUI render loop must stay responsive.

**Solution:**
1. Spawn `smol` task for PTY read loop
2. Use `mpsc::channel` to send output to GPUI thread
3. Call `cx.notify()` to trigger re-render on data arrival

```rust
// Conceptual pattern
cx.spawn(|view, mut cx| async move {
    let mut pty_reader = pty.reader();
    loop {
        let data = pty_reader.read().await?;
        cx.update(|view, cx| {
            view.terminal.process_bytes(&data);
            cx.notify(); // Trigger re-render
        })?;
    }
}).detach();
```

#### Point 2: alacritty_terminal Grid → GPUI Elements

**Challenge:** alacritty_terminal maintains character grid, GPUI needs visual elements.

**Solution:**
1. Iterate terminal grid cells
2. Group consecutive cells with same style
3. Render as styled `div()` with text

```rust
// Conceptual rendering loop
fn render_grid(term: &Term, cx: &mut ViewContext) -> impl IntoElement {
    let mut rows = Vec::new();
    for line in term.grid().display_iter() {
        let mut row = div().flex();
        for cell in line.cells() {
            row = row.child(
                div()
                    .text_color(cell.fg)
                    .bg(cell.bg)
                    .child(cell.c.to_string())
            );
        }
        rows.push(row);
    }
    div().flex_col().children(rows)
}
```

#### Point 3: Settings → GPUI State Management

**Challenge:** Settings changes must propagate to active views.

**Solution:**
1. Store settings in GPUI `Global` state
2. Views observe settings with `cx.observe_global()`
3. Call `cx.notify()` on settings reload

```rust
// Conceptual pattern
cx.observe_global::<SettingsStore>(|this, cx| {
    this.apply_settings(cx.global::<SettingsStore>());
    cx.notify();
}).detach();
```

---

## 3. GPUI API Patterns (v0.220.3)

### 3.1 Application Lifecycle

```rust
use gpui::*;

fn main() {
    App::new().run(|cx: &mut AppContext| {
        // Register global state
        cx.set_global(Settings::load());

        // Open initial window
        cx.open_window(WindowOptions::default(), |cx| {
            WorkspaceView::new(cx)
        });
    });
}
```

### 3.2 View Creation

```rust
struct MyView {
    state: Model<MyState>,
}

impl MyView {
    fn new(cx: &mut WindowContext) -> View<Self> {
        cx.new_view(|cx| {
            let state = cx.new_model(|_| MyState::default());
            Self { state }
        })
    }
}
```

### 3.3 Rendering with Styled Divs

```rust
impl Render for MyView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .gap_2()
            .p_4()
            .bg(cx.theme().background)
            .child(
                div()
                    .text_lg()
                    .font_bold()
                    .child("Hello GPUI")
            )
    }
}
```

### 3.4 Event Handling

```rust
div()
    .on_click(cx.listener(|this, event: &ClickEvent, cx| {
        this.handle_click(event, cx);
    }))
    .child("Clickable")
```

### 3.5 Background Tasks

```rust
cx.spawn(|view, mut cx| async move {
    let result = expensive_computation().await;
    cx.update(|view, cx| {
        view.update_with_result(result);
        cx.notify();
    })
}).detach();
```

---

## 4. Phase 1 Implementation Plan

### 4.1 Milestone 1: Minimal GPUI App (Empty Window)

**Goal:** Prove GPUI integration, open a window with title bar.

**Tasks:**
1. Update `Cargo.toml` with GPUI dependency (tag = v0.220.3)
2. Create minimal `main.rs` with GPUI app
3. Open window with "TerminalG" title
4. Test on macOS (primary platform)

**Acceptance criteria:**
- `cargo build` succeeds
- `cargo run` opens window
- Window has correct title
- Window closes cleanly

**Files changed:**
- `Cargo.toml` (add gpui dependency)
- `src/main.rs` (GPUI app skeleton)

**LOC estimate:** ~50 lines

### 4.2 Milestone 2: Settings Integration

**Goal:** Load settings, apply theme, display in window.

**Tasks:**
1. Connect settings module to GPUI global state
2. Load theme on app start
3. Render colored background based on theme
4. Display settings info in window (debug view)

**Acceptance criteria:**
- Settings loaded from `~/.config/terminalg/settings.json`
- Theme colors applied to window background
- Settings changes reflected on app restart

**Files changed:**
- `src/main.rs` (settings integration)
- `src/ui/mod.rs` (theme application)

**LOC estimate:** ~100 lines

### 4.3 Milestone 3: Workspace View Skeleton

**Goal:** Render WorkspaceView with placeholder panes.

**Tasks:**
1. Create `WorkspaceView` struct
2. Implement `Render` with flex layout
3. Add placeholder panes (colored divs)
4. Implement resizable split (drag handle)

**Acceptance criteria:**
- Window shows 2-pane layout
- Panes have different background colors
- Resize handle works (drag to adjust split)
- Layout persists during window resize

**Files changed:**
- `src/ui/workspace.rs` (new file)
- `src/ui/mod.rs` (module exports)
- `src/main.rs` (spawn WorkspaceView)

**LOC estimate:** ~200 lines

### 4.4 Milestone 4: Tab Management

**Goal:** Multiple workspace tabs, switching, persistence.

**Tasks:**
1. Add tab bar to WorkspaceView
2. Implement tab switching logic
3. Save/restore active tab in settings
4. Add keyboard shortcuts (Cmd+1, Cmd+2, etc.)

**Acceptance criteria:**
- Tab bar shows multiple tabs
- Clicking tab switches active pane
- Active tab persists across restarts
- Keyboard shortcuts work

**Files changed:**
- `src/ui/workspace.rs` (tab management)
- `src/ui/tab_bar.rs` (new file)

**LOC estimate:** ~150 lines

---

## 5. Risk Assessment & Mitigation

### 5.1 High-Risk Areas

#### Risk 1: GPUI API Instability
**Likelihood:** Medium
**Impact:** High (may require significant refactoring)

**Mitigation:**
- Pin to stable release tag (v0.220.3)
- Study Zed's usage patterns extensively before writing code
- Create abstraction layer if GPUI changes frequently
- Budget time for API adaptation during upgrades

**Monitoring:**
- Watch Zed's GPUI changelog on each release
- Track GPUI API breakage in Zed's commit history

#### Risk 2: PTY Thread Synchronization
**Likelihood:** High
**Impact:** High (crashes, deadlocks, UI freezes)

**Mitigation:**
- Use proven patterns from Zed's terminal implementation
- Extensive testing with high-throughput scenarios
- Use `smol` for async, avoid blocking GPUI thread
- Implement back-pressure if terminal output overwhelms renderer

**Testing strategy:**
- Stress test: `cat large_file.txt`
- Rapid output: `yes | head -n 100000`
- Interactive latency: keystroke → echo → render

#### Risk 3: Cross-Platform PTY Differences
**Likelihood:** High (Windows especially)
**Impact:** Medium (Windows support delayed)

**Mitigation:**
- Start with macOS/Linux (POSIX PTY)
- Research Windows ConPTY API thoroughly before implementation
- Consider using Zed's PTY abstraction layer
- Budget separate time for Windows testing

**Deferral strategy:**
- Phase 1: macOS only
- Phase 2: Add Linux support
- Phase 3: Windows ConPTY (separate effort)

### 5.2 Medium-Risk Areas

#### Risk 4: GPUI Performance at High Terminal Throughput
**Likelihood:** Medium
**Impact:** Medium (sluggish UI, dropped frames)

**Mitigation:**
- Profile early with GPUI's built-in profiler
- Implement render throttling if needed
- Use dirty region tracking (only re-render changed cells)
- Consider terminal buffer optimization (Zed's approach)

#### Risk 5: Complex Resize Logic
**Likelihood:** Low
**Impact:** Medium (broken layouts, janky UX)

**Mitigation:**
- Use GPUI's flex layout (battle-tested in Zed)
- Copy Zed's pane resize patterns
- Test with various window sizes
- Add constraints (minimum pane sizes)

### 5.3 Low-Risk Areas

#### Risk 6: Theme Application
**Likelihood:** Low
**Impact:** Low (visual inconsistency)

**Status:** Already mitigated (theme system implemented)

#### Risk 7: Settings Hot-Reload
**Likelihood:** Low
**Impact:** Low (requires restart)

**Mitigation:**
- Use `notify` crate (already in deps)
- Implement file watcher in Phase 3
- Fall back to restart if watching fails

---

## 6. Learning Resources

### 6.1 GPUI Documentation
- **Official docs:** https://docs.rs/gpui (limited)
- **Source code:** https://github.com/zed-industries/zed/tree/v0.220.3/crates/gpui
- **Examples:** https://github.com/zed-industries/zed/tree/v0.220.3/crates/gpui/examples

### 6.2 Zed Codebase Reference Points
Study these files for patterns:

| Component | Zed File Path | What to Learn |
|-----------|--------------|---------------|
| App initialization | `src/main.rs` | GPUI app setup, window creation |
| Terminal pane | `crates/terminal/src/terminal.rs` | PTY integration, grid rendering |
| Settings system | `crates/settings/src/settings.rs` | Settings loading, observation |
| Theme application | `crates/theme/src/theme.rs` | Color schemes, styling |
| Workspace layout | `crates/workspace/src/workspace.rs` | Tab management, pane splits |
| Async tasks | `crates/gpui/src/executor.rs` | Background task patterns |

### 6.3 Terminal Emulation References
- **Alacritty docs:** https://docs.rs/alacritty_terminal
- **VTE spec:** https://invisible-island.net/xterm/ctlseqs/ctlseqs.html
- **PTY guide:** https://blog.nelhage.com/2009/12/a-brief-introduction-to-termios/

---

## 7. Upgrade Path

### 7.1 Zed Release Monitoring

**Frequency:** Check biweekly (Zed releases ~every 2 weeks)

**Process:**
1. Visit https://github.com/zed-industries/zed/releases
2. Review release notes for GPUI changes
3. Check if changes affect TerminalG:
   - Breaking API changes in GPUI
   - New features we want (text rendering, layout)
   - Bug fixes in areas we use

4. Decision matrix:

| Change Type | Action |
|-------------|--------|
| Critical bug in GPUI | Upgrade immediately |
| New feature we need | Evaluate → test → upgrade |
| Breaking API change | Assess impact → adapt → upgrade |
| Minor fixes | Defer until batch upgrade |

### 7.2 Upgrade Procedure

**Steps:**
1. **Create branch:** `git checkout -b upgrade/zed-v0.221.0`
2. **Update Cargo.toml:**
   ```toml
   gpui = { git = "https://github.com/zed-industries/zed", tag = "v0.221.0" }
   ```
3. **Review changes:**
   ```bash
   # Check GPUI diff
   git clone --depth 1 --branch v0.220.3 https://github.com/zed-industries/zed zed-old
   git clone --depth 1 --branch v0.221.0 https://github.com/zed-industries/zed zed-new
   diff -ru zed-old/crates/gpui zed-new/crates/gpui > gpui-changes.diff
   ```
4. **Build and test:**
   ```bash
   cargo clean
   cargo build
   cargo test
   cargo run  # Manual testing
   ```
5. **Update docs:**
   - This document (version references)
   - CHANGELOG.md (note GPUI upgrade)
   - README.md (if user-facing changes)
6. **Commit and merge:**
   ```
   git commit -m "deps: Upgrade GPUI to Zed v0.221.0

   - Update GPUI dependency tag
   - Adapt to API changes in [affected areas]
   - Fixes: [any bugs]

   Zed release: https://github.com/zed-industries/zed/releases/tag/v0.221.0"
   ```

### 7.3 Breaking Change Adaptation

**Common breaking patterns (from Zed history):**

1. **Render trait changes:**
   - Old: `fn render(&mut self, cx: &mut ViewContext) -> Element`
   - New: `fn render(&mut self, cx: &mut ViewContext) -> impl IntoElement`
   - **Fix:** Update return types, use `div()` builders

2. **Context method renames:**
   - Example: `cx.notify()` → `cx.emit()`
   - **Fix:** Find-and-replace with testing

3. **Styling API changes:**
   - Example: Tailwind-style classes → typed builders
   - **Fix:** Update render code, may be extensive

**Strategy:** Budget 2-4 hours per major GPUI upgrade for adaptation.

---

## 8. Success Metrics

### 8.1 Phase 1 Complete When:
- [ ] GPUI app launches with window
- [ ] Settings loaded and applied
- [ ] Theme colors visible
- [ ] WorkspaceView renders with 2-pane layout
- [ ] Resize handle adjusts split
- [ ] Window state persists (size, position)
- [ ] No crashes or memory leaks
- [ ] Build time under 5 minutes (clean build)

### 8.2 Quality Gates:
- [ ] `cargo build` succeeds without warnings
- [ ] `cargo clippy` passes
- [ ] `cargo test` passes all tests
- [ ] Manual testing checklist completed
- [ ] Works on macOS 10.15+

### 8.3 Performance Targets:
- Window launch: < 2 seconds
- Render frame time: < 16ms (60 FPS)
- Memory usage (idle): < 100 MB
- Binary size (release): < 50 MB

---

## 9. Next Steps

### 9.1 Immediate Actions (Before Coding)
1. ✅ Document dependency strategy (this doc)
2. ⏳ Study Zed's `main.rs` and workspace setup
3. ⏳ Extract GPUI patterns from Zed examples
4. ⏳ Create detailed implementation checklist

### 9.2 Phase 1.5: GPUI Bootstrap (Next Session)
1. Update `Cargo.toml` with GPUI dependency
2. Write minimal `main.rs` (empty window)
3. Verify build on macOS
4. Create `WorkspaceView` skeleton

### 9.3 Documentation Tasks
- [ ] Create `docs/GPUI-PATTERNS.md` (extracted from Zed)
- [ ] Update `docs/IMPLEMENTATION-PLAN.md` (detailed checklist)
- [ ] Add GPUI version to `README.md`

---

## Appendix A: Dependency Lockfile Strategy

**Question:** Should we commit `Cargo.lock`?

**Answer:** **YES** for application crates.

**Rationale:**
- Ensures reproducible builds across machines
- Locks transitive dependencies (GPUI's deps)
- Critical for debugging (know exact versions)

**In `.gitignore`:** Remove `Cargo.lock` if present.

**In git:** `git add Cargo.lock && git commit -m "chore: Track Cargo.lock for reproducibility"`

---

## Appendix B: Build Optimization

### Build Time Optimization
```toml
# .cargo/config.toml (create if needed)
[build]
# Use faster linker (macOS)
rustflags = ["-C", "link-arg=-fuse-ld=lld"]

# Parallel codegen
[profile.dev]
codegen-units = 16

# Faster incremental builds
incremental = true
```

### Caching Strategy
- Use `sccache` for CI builds
- Keep `target/` directory between sessions
- Consider `cargo-chef` for Docker builds (future)

---

## Revision History

| Version | Date | Author | Changes |
|---------|------|--------|---------|
| 1.0 | 2025-01-24 | ARCH-RTERM | Initial architecture document |
| | | | Zed v0.220.3 baseline established |

---

**Document Status:** ✅ Complete - Ready for review
**Next Review:** After Phase 1 implementation (update with lessons learned)
