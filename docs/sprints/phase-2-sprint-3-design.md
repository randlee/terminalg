# Phase 2 Sprint 2.3 Design: URL Recognition & Clicking

**Version:** 1.0
**Created:** 2026-01-26
**Status:** Ready for Implementation
**Prerequisite:** Sprint 2.2 Complete (Terminal Pane Integration)
**Estimated Duration:** 6-10 hours

---

## Development Environment

| Item | Value |
|------|-------|
| **Worktree Path** | `/Users/randlee/Documents/github/terminalg-worktrees/feature/sprint-2-3-url-recognition` |
| **Branch** | `feature/sprint-2-3-url-recognition` |
| **Base Branch** | `develop` |
| **Created** | 2026-01-26 |

```bash
# Navigate to worktree
cd /Users/randlee/Documents/github/terminalg-worktrees/feature/sprint-2-3-url-recognition

# Verify branch
git branch --show-current
# → feature/sprint-2-3-url-recognition
```

---

## 1. Overview

Sprint 2.3 adds URL detection and clicking functionality to TerminalG's terminal pane by leveraging Zed's existing hyperlink infrastructure. This sprint requires **minimal new code** since Zed's terminal crate already provides:

- URL regex pattern matching (`terminal_hyperlinks.rs`)
- Mouse event handling (`mouse_move`, `mouse_down`, `mouse_up`)
- Hover state tracking (`HoveredWord`, `last_hovered_word`)
- Events for opening URLs (`Event::Open`)

### Key Deliverables

1. URL regex configuration in `TerminalBuilder`
2. Mouse event wiring to Zed's terminal methods
3. Event subscription for `Open` and `NewNavigationTarget`
4. Visual feedback for hyperlinks (hover state + cursor change)
5. Browser launching for clicked URLs

### Success Criteria

- Ctrl/Cmd+hover displays URL in status bar
- Ctrl/Cmd+click opens URLs in default browser
- Works with http://, https://, git://, ssh://, file://, etc.
- Hover state visible via cursor change
- No false positives on terminal text

---

## 2. Architecture Analysis

### 2.1 Zed's URL Detection System

**Three-Layer Detection** (`terminal_hyperlinks.rs`):
```
Layer 1: ANSI Hyperlinks (OSC 8 sequences)
   ↓ Not found?
Layer 2: URL_REGEX pattern
   Pattern: (http|https|git|ssh|file|...):// + valid chars
   Sanitizes trailing punctuation (., ,, :, ;)
   ↓ Not found?
Layer 3: Path Regex (custom patterns)
   Configured via path_hyperlink_regexes
```

**URL_REGEX Pattern** (line 20 of `terminal_hyperlinks.rs`):
```rust
const URL_REGEX: &str = r#"(ipfs:|ipns:|magnet:|mailto:|gemini://|gopher://|https://|http://|news:|file://|git://|ssh:|ftp://)[^\u{0000}-\u{001F}\u{007F}-\u{009F}<>"\s{-}\^⟨⟩`']+"#;
```

### 2.2 Zed's Mouse Event Flow

```
User Mouse Move (Ctrl/Cmd held)
    ↓ GPUI MouseMoveEvent
Terminal.mouse_move()
    ↓ Checks modifiers.secondary()
    ↓ Throttles (5px spatial, 100ms temporal)
InternalEvent::FindHyperlink(position, open=false)
    ↓
terminal_hyperlinks::find_from_grid_point()
    ↓ Returns (url, is_url, match_range)
Terminal.process_hyperlink()
    ↓ Updates last_hovered_word
Event::NewNavigationTarget(Some(MaybeNavigationTarget::Url))
    ↑ Emitted to subscribers

User Mouse Down (Ctrl/Cmd held)
    ↓
Terminal.mouse_down()
    ↓ Stores hyperlink at click position
mouse_down_hyperlink = find_from_grid_point(...)

User Mouse Up (Ctrl/Cmd held)
    ↓
Terminal.mouse_up()
    ↓ Compares stored hyperlink with current
    ↓ If same, open URL
InternalEvent::ProcessHyperlink(..., open=true)
    ↓
Event::Open(MaybeNavigationTarget::Url(url))
    ↑ Emitted to subscribers
```

### 2.3 Current TerminalG State

**Rendering** (`src/terminal/pane.rs` lines 254-314):
- Extracts plain text from `content.cells`
- **Missing:** Cell styling, hyperlink underlines
- **Missing:** Access to `last_hovered_word` for hover effects

**Event Handling** (lines 122-151):
- Handles: `TitleChanged`, `BreadcrumbsChanged`, `CloseTerminal`, `Wakeup`, `Bell`
- **Missing:** `Open`, `NewNavigationTarget`

**Input Handling**:
- Keyboard: ✅ Implemented via `handle_key_down()`
- Mouse: ❌ Not implemented

**URL Configuration** (line 83):
```rust
Vec::new(), // path_hyperlink_regexes <- EMPTY!
```

---

## 3. Implementation Blueprint

### 3.1 Phase 1: URL Regex Configuration (1 hour)

**Objective:** Pass URL patterns to `TerminalBuilder` for path detection.

**File:** `src/terminal/pane.rs`

**Changes:**

1. Add default path regex patterns (near top of file after imports):

```rust
// Add near top of file (after imports)
const DEFAULT_PATH_REGEXES: &[&str] = &[
    // File paths with optional line:col
    r"[a-zA-Z0-9._\-~/]+/[a-zA-Z0-9._\-~/]+(?::\d+)?(?::\d+)?",
    // GitHub-style file references
    r"[\w\-/\.]+\.(?:rs|js|ts|py|go|java|c|cpp|h|md|txt)",
];
```

2. In `spawn_terminal()` method, replace line 83:

```rust
// Before line 75, prepare regex patterns
let path_hyperlink_regexes: Vec<String> = DEFAULT_PATH_REGEXES
    .iter()
    .map(|s| s.to_string())
    .collect();

// Replace line 83 with:
path_hyperlink_regexes, // Use configured patterns (instead of Vec::new())
```

**Testing:**
- Terminal spawns without errors
- No change to existing behavior yet

---

### 3.2 Phase 2: Mouse Event Wiring (2-3 hours)

**Objective:** Connect GPUI mouse events to Zed's terminal mouse handlers.

**File:** `src/terminal/pane.rs`

**Changes:**

1. **Add mouse event handlers** (after `handle_key_down()` at line 203):

```rust
/// Handle mouse move events
fn handle_mouse_move(
    &mut self,
    event: &gpui::MouseMoveEvent,
    _window: &mut Window,
    cx: &mut Context<Self>,
) {
    if let Some(tab) = self.active_tab_mut() {
        tab.terminal.update(cx, |terminal, cx| {
            terminal.mouse_move(event, cx);
        });
        cx.notify();
    }
}

/// Handle mouse down events
fn handle_mouse_down(
    &mut self,
    event: &gpui::MouseDownEvent,
    _window: &mut Window,
    cx: &mut Context<Self>,
) {
    if let Some(tab) = self.active_tab_mut() {
        tab.terminal.update(cx, |terminal, cx| {
            terminal.mouse_down(event, cx);
        });
        cx.notify();
    }
}

/// Handle mouse up events
fn handle_mouse_up(
    &mut self,
    event: &gpui::MouseUpEvent,
    _window: &mut Window,
    cx: &mut Context<Self>,
) {
    if let Some(tab) = self.active_tab_mut() {
        tab.terminal.update(cx, |terminal, cx| {
            terminal.mouse_up(event, cx);
        });
        cx.notify();
    }
}
```

2. **Wire events to render** (modify `render()` method):

```rust
fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    div()
        .track_focus(&self.focus_handle)
        .flex()
        .flex_col()
        .size_full()
        .on_key_down(cx.listener(Self::handle_key_down))
        .on_mouse_move(cx.listener(Self::handle_mouse_move))  // ADD
        .on_mouse_down(cx.listener(Self::handle_mouse_down))  // ADD
        .on_mouse_up(cx.listener(Self::handle_mouse_up))      // ADD
        .child(self.render_terminal_content(cx))
        .child(self.render_tabs(cx))
}
```

**Testing:**
- Mouse events logged via `tracing::debug!`
- Ctrl/Cmd+hover triggers `FindHyperlink` (check logs)
- No crashes or panics

---

### 3.3 Phase 3: Event Handling - Open URLs (2-3 hours)

**Objective:** Subscribe to `Open` events and launch browser.

**File:** `src/terminal/pane.rs`

**Changes:**

1. **Extend event handler** (modify `handle_terminal_event()`):

```rust
fn handle_terminal_event(&mut self, event: &TerminalEvent, cx: &mut Context<Self>) {
    match event {
        TerminalEvent::TitleChanged | TerminalEvent::BreadcrumbsChanged => {
            cx.emit(TerminalPaneEvent::TitleChanged);
            cx.notify();
        }
        TerminalEvent::CloseTerminal => {
            // ... existing code ...
        }
        TerminalEvent::Wakeup => {
            cx.notify();
        }
        TerminalEvent::Bell => {
            tracing::debug!("Terminal bell");
        }
        // ADD: Handle URL open events
        TerminalEvent::Open(target) => {
            self.handle_open_target(target, cx);
        }
        // ADD: Handle hover state changes
        TerminalEvent::NewNavigationTarget(target) => {
            self.handle_navigation_target(target, cx);
        }
        _ => {}
    }
}
```

2. **Add helper methods** (after `handle_terminal_event()`):

```rust
/// Handle opening a URL or path
fn handle_open_target(
    &mut self,
    target: &terminal::MaybeNavigationTarget,
    cx: &mut Context<Self>,
) {
    match target {
        terminal::MaybeNavigationTarget::Url(url) => {
            tracing::info!("Opening URL: {}", url);
            if let Err(e) = open::that(url) {
                tracing::error!("Failed to open URL {}: {}", url, e);
            }
        }
        terminal::MaybeNavigationTarget::PathLike(path_target) => {
            // Future: implement path navigation
            tracing::info!("Path navigation not yet implemented: {}", path_target.maybe_path);
        }
    }
}

/// Handle navigation target hover state
fn handle_navigation_target(
    &mut self,
    target: &Option<terminal::MaybeNavigationTarget>,
    cx: &mut Context<Self>,
) {
    // Future: show tooltip or status indicator
    if let Some(terminal::MaybeNavigationTarget::Url(url)) = target {
        tracing::debug!("Hovering URL: {}", url);
    }
    cx.notify();
}
```

3. **Add `open` crate dependency** (`Cargo.toml`):

```toml
[dependencies]
open = "5.0"  # For cross-platform URL launching
```

**Testing:**
- Ctrl/Cmd+click on `https://example.com` opens browser
- Works with http://, https://, git://, ssh://
- Error logged if URL malformed
- No crashes

---

### 3.4 Phase 4: Visual Styling - Hover Effects (2-3 hours)

**Objective:** Render hover state feedback (simplified approach).

**Key Decision:** Instead of rewriting cell-by-cell rendering, use simplified visual feedback:
- Display hovered URL in terminal footer
- Change cursor to pointer on hover
- Defer full underline rendering to future sprint

**File:** `src/terminal/pane.rs`

**Changes:**

1. **Enhance rendering** (modify `render_terminal_content()`):

```rust
/// Render the terminal content area with hover feedback
fn render_terminal_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let theme = cx.theme();

    if let Some(tab) = self.tabs.get(self.active_tab) {
        let terminal = tab.terminal.read(cx);
        let content = terminal.last_content();

        // Get hovered URL for display
        let hovered_url = content.last_hovered_word.as_ref()
            .map(|hw| hw.word.clone());

        // Simple text rendering (existing code)
        let mut lines: Vec<String> = Vec::new();
        // ... existing cell iteration code ...

        div()
            .flex_1()
            .w_full()
            .relative()
            .bg(theme.colors().terminal_background)
            .text_color(theme.colors().terminal_foreground)
            .font_family("Menlo")
            .text_sm()
            .p_2()
            .overflow_hidden()
            .children(lines.into_iter().map(|line| {
                div().child(if line.is_empty() { " ".to_string() } else { line })
            }))
            // ADD: Show hovered URL in footer
            .when(hovered_url.is_some(), |d| {
                d.child(
                    div()
                        .absolute()
                        .bottom_0()
                        .left_0()
                        .right_0()
                        .px_2()
                        .py_1()
                        .bg(theme.colors().element_background)
                        .border_t_1()
                        .border_color(theme.colors().border)
                        .text_xs()
                        .text_color(theme.colors().link_text_hover)
                        .child(format!("🔗 {}", hovered_url.unwrap_or_default()))
                )
            })
    } else {
        // ... loading state ...
    }
}
```

2. **Add cursor change on hover** (modify `render()`):

```rust
fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
    // Check if hovering over a URL
    let hovering_url = self.tabs.get(self.active_tab)
        .and_then(|tab| {
            tab.terminal.read(cx)
                .last_content()
                .last_hovered_word
                .as_ref()
                .map(|_| true)
        })
        .unwrap_or(false);

    div()
        .track_focus(&self.focus_handle)
        .flex()
        .flex_col()
        .size_full()
        .when(hovering_url, |d| d.cursor_pointer())  // ADD
        .on_key_down(cx.listener(Self::handle_key_down))
        .on_mouse_move(cx.listener(Self::handle_mouse_move))
        .on_mouse_down(cx.listener(Self::handle_mouse_down))
        .on_mouse_up(cx.listener(Self::handle_mouse_up))
        .child(self.render_terminal_content(cx))
        .child(self.render_tabs(cx))
}
```

**Future Enhancement (Sprint 2.4+):**
- Implement proper cell-by-cell rendering with styled spans
- Apply `link_style` with underline decoration
- Match Zed's full hyperlink visual appearance

**Testing:**
- Ctrl/Cmd+hover shows URL at bottom of terminal
- Cursor changes to pointer on hover
- URL disappears when Ctrl/Cmd released

---

## 4. Data Flow Diagram

```
┌──────────────────────────────────────────────────────────────────┐
│                        User Interaction                          │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ Ctrl/Cmd + Mouse Move over "https://example.com"
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  TerminalPane.handle_mouse_move()                                │
│  - Forward to Terminal.mouse_move()                              │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ MouseMoveEvent { modifiers.secondary: true }
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  Terminal.mouse_move() [Zed]                                     │
│  - Check modifiers.secondary() (Ctrl/Cmd)                        │
│  - Throttle: 5px spatial, 100ms temporal                         │
│  - Create InternalEvent::FindHyperlink(position, open=false)     │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ InternalEvent::FindHyperlink
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  Terminal.process_internal_event() [Zed]                         │
│  - Convert pixel position to grid point                          │
│  - Call terminal_hyperlinks::find_from_grid_point()              │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ grid_point, regex_searches
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  terminal_hyperlinks::find_from_grid_point() [Zed]               │
│  1. Check ANSI hyperlinks (OSC 8)                                │
│  2. Search URL_REGEX pattern                                     │
│  3. Search path_hyperlink_regexes                                │
│  4. Sanitize trailing punctuation                               │
│  Returns: Some((url, is_url, match_range))                      │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ ("https://example.com", true, 10..=29)
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  Terminal.process_hyperlink() [Zed]                              │
│  - Update last_content.last_hovered_word = HoveredWord {         │
│      word: "https://example.com",                                │
│      word_match: 10..=29,                                        │
│      id: unique_id                                               │
│    }                                                             │
│  - Emit Event::NewNavigationTarget(Some(Url(...)))              │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ Event::NewNavigationTarget
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  TerminalPane.handle_terminal_event()                            │
│  - Call handle_navigation_target()                               │
│  - Log: "Hovering URL: https://example.com"                     │
│  - cx.notify() triggers re-render                               │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ cx.notify()
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  TerminalPane.render()                                           │
│  - Read last_hovered_word from terminal content                  │
│  - Set cursor_pointer() when hovering URL                        │
│  - Display URL in bottom status bar                             │
└──────────────────────────────────────────────────────────────────┘

═══════════════════════════════════════════════════════════════════

User Action: Ctrl/Cmd + Click

┌──────────────────────────────────────────────────────────────────┐
│  TerminalPane.handle_mouse_down()                                │
│  - Forward to Terminal.mouse_down()                              │
└───────────────┬──────────────────────────────────────────────────┘
                │
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  Terminal.mouse_down() [Zed]                                     │
│  - Find hyperlink at click position                              │
│  - Store in mouse_down_hyperlink: Some((...))                    │
└──────────────────────────────────────────────────────────────────┘

User Action: Ctrl/Cmd + Release

┌──────────────────────────────────────────────────────────────────┐
│  TerminalPane.handle_mouse_up()                                  │
│  - Forward to Terminal.mouse_up()                                │
└───────────────┬──────────────────────────────────────────────────┘
                │
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  Terminal.mouse_up() [Zed]                                       │
│  - Find hyperlink at release position                            │
│  - Compare with stored mouse_down_hyperlink                      │
│  - If same: Create InternalEvent::ProcessHyperlink(..., true)    │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ InternalEvent::ProcessHyperlink(open=true)
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  Terminal.process_hyperlink() [Zed]                              │
│  - Emit Event::Open(MaybeNavigationTarget::Url(...))            │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ Event::Open
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  TerminalPane.handle_terminal_event()                            │
│  - Call handle_open_target()                                     │
│  - Extract URL string                                            │
│  - Call open::that(url)                                          │
└───────────────┬──────────────────────────────────────────────────┘
                │
                │ open::that("https://example.com")
                ↓
┌──────────────────────────────────────────────────────────────────┐
│  System Default Browser                                          │
│  - Browser launched with URL                                     │
└──────────────────────────────────────────────────────────────────┘
```

---

## 5. Build Sequence Checklist

### Phase 1: URL Regex Configuration (1 hour)
- [ ] Add `DEFAULT_PATH_REGEXES` constant to `pane.rs`
- [ ] Modify `spawn_terminal()` to pass regex patterns
- [ ] Test: Terminal spawns without errors
- [ ] Test: No change to existing behavior
- [ ] Commit: "feat: configure URL path regexes for terminal hyperlinks"

### Phase 2: Mouse Event Wiring (2-3 hours)
- [ ] Add `handle_mouse_move()` method
- [ ] Add `handle_mouse_down()` method
- [ ] Add `handle_mouse_up()` method
- [ ] Wire events in `render()` method
- [ ] Test: Mouse events logged via tracing
- [ ] Test: Ctrl/Cmd+hover triggers FindHyperlink (check logs)
- [ ] Test: No crashes or panics
- [ ] Commit: "feat: wire mouse events to terminal hyperlink detection"

### Phase 3: Event Handling - Open URLs (2-3 hours)
- [ ] Add `open` crate to `Cargo.toml`
- [ ] Extend `handle_terminal_event()` with `Open` case
- [ ] Extend `handle_terminal_event()` with `NewNavigationTarget` case
- [ ] Add `handle_open_target()` method
- [ ] Add `handle_navigation_target()` method
- [ ] Test: Ctrl/Cmd+click on `https://example.com` opens browser
- [ ] Test: Works with http://, https://, git://, ssh://
- [ ] Test: Error logged if URL malformed
- [ ] Test: No crashes
- [ ] Commit: "feat: implement URL opening in default browser"

### Phase 4: Visual Styling - Hover Effects (2-3 hours)
- [ ] Modify `render_terminal_content()` to show hovered URL
- [ ] Add cursor pointer style when hovering URL
- [ ] Test: Ctrl/Cmd+hover shows URL at bottom of terminal
- [ ] Test: Cursor changes to pointer on hover
- [ ] Test: URL disappears when Ctrl/Cmd released
- [ ] Commit: "feat: add visual feedback for URL hover state"

### Final Integration (1 hour)
- [ ] Run full test suite: `cargo test`
- [ ] Run clippy: `cargo clippy -- -D warnings`
- [ ] Manual testing: various URL types
- [ ] Update MASTER-PLAN.md: Sprint 2.3 complete
- [ ] Commit: "docs: mark Sprint 2.3 complete"

---

## 6. Testing Plan

### Manual Testing Checklist

**URL Types:**
- [ ] `https://example.com` - HTTPS URL
- [ ] `http://example.com` - HTTP URL
- [ ] `git://github.com/user/repo` - Git URL
- [ ] `ssh://user@host.com` - SSH URL
- [ ] `file:///path/to/file` - File URL
- [ ] `mailto:user@example.com` - Mailto URL
- [ ] `ftp://ftp.example.com` - FTP URL

**Edge Cases:**
- [ ] URL with trailing period: `https://example.com.` → opens `https://example.com`
- [ ] URL with trailing comma: `https://example.com,` → opens `https://example.com`
- [ ] URL in parentheses: `(https://example.com)` → opens `https://example.com`
- [ ] URL with query params: `https://example.com?foo=bar`
- [ ] URL with fragment: `https://example.com#section`
- [ ] URL with port: `http://localhost:8080`

**Interaction Tests:**
- [ ] Hover without Ctrl/Cmd - no effect
- [ ] Hover with Ctrl/Cmd - URL highlighted, cursor pointer
- [ ] Click without Ctrl/Cmd - normal terminal click
- [ ] Click with Ctrl/Cmd - browser opens
- [ ] Drag with Ctrl/Cmd held - no URL open (movement threshold)
- [ ] Release Ctrl/Cmd - highlighting disappears

---

## 7. Risk Assessment

| Risk | Severity | Likelihood | Mitigation |
|------|----------|------------|------------|
| Mouse events interfere with terminal selection | Medium | Medium | Zed's implementation handles this - only Ctrl/Cmd+click |
| URL regex causes performance issues | Low | Low | Throttled by Zed (5px, 100ms) - proven in production |
| False positive URL detection | Low | Medium | Use Zed's tested URL_REGEX - 447 test cases |
| Browser fails to open | Low | Medium | Log error, continue execution - graceful degradation |
| Complex rendering with underlines | High | Medium | Deferred to future sprint - use simple hover feedback |

---

## 8. References

### Zed Source Files (Reference)
- `zed/crates/terminal/src/terminal.rs`
  - Lines 1725-1977: Mouse event handling
  - Lines 1165-1228: Hyperlink detection and processing
  - Lines 790-794: HoveredWord struct
- `zed/crates/terminal/src/terminal_hyperlinks.rs`
  - Line 20: URL_REGEX pattern
  - Lines 69-155: find_from_grid_point() implementation

### TerminalG Source Files (Modify)
- `src/terminal/pane.rs`
  - Lines 75-90: TerminalBuilder configuration
  - Lines 122-151: Event handling
  - Lines 254-314: Terminal content rendering
  - Lines 325-336: Render method

### External Libraries
- **open crate:** https://docs.rs/open/5.0.0/open/
  - Cross-platform URL launching

---

**Document Status:** Ready for Implementation
**Next Steps:** Create feature branch, begin Phase 1 implementation
