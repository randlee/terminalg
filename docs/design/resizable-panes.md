# Resizable Panes Design Document

## Overview

This document describes the architecture for adding draggable vertical dividers between panes in TerminalG, enabling users to resize the file browser, terminal, and document viewer panes.

## Current State

- **Layout**: `src/ui/workspace.rs` - `render_content()` uses equal `flex_1()` for all panes
- **Config**: `src/ui/workspace_config.rs` - `pane_ratios: [f32; 3]` exists with defaults `[1.0, 2.0, 1.0]` but is **unused**
- **Persistence**: Debounced save pattern already implemented (200ms delay)

## Architecture Decision

**Approach: Global Drag State with Mouse Event Handlers**

Rationale:
1. GPUI provides robust mouse event system (`on_mouse_down`, `on_mouse_move`, `on_mouse_up`)
2. Global listeners enable drag continuation outside divider bounds
3. State stored in `WorkspaceView` for simplicity
4. Leverages existing persistence infrastructure

## Component Design

### 1. ResizeDragState

```rust
struct ResizeDragState {
    divider_index: usize,      // 0 or 1 (between which panes)
    start_x: Pixels,           // Initial mouse X position
    start_ratios: [f32; 3],    // Initial pane ratios
    total_width: Pixels,       // Available width for panes
}
```

### 2. Divider Component

- **Width**: 6px hitbox (discoverable but unobtrusive)
- **Visual**: 1px visible border line
- **Cursor**: `cursor_col_resize()` on hover
- **States**: Normal, hover (highlighted), dragging (highlighted)
- **Double-click**: Reset ratios to default

### 3. WorkspaceView Modifications

Add fields:
```rust
pub struct WorkspaceView {
    // ... existing fields ...
    resize_drag_state: Option<ResizeDragState>,
}

const MIN_PANE_WIDTH: Pixels = px(150.0);
```

## State Flow

```
Mouse Down on Divider
    ↓
Create ResizeDragState {
    divider_index,
    start_x: event.position.x,
    start_ratios: current ratios,
    total_width: container width
}
    ↓
Store in WorkspaceView
    ↓
cx.notify() → Re-render

Mouse Move (global listener)
    ↓
If drag_state.is_some():
    Calculate delta_x
    Calculate new ratios (with min-width constraints)
    Update workspace_state.pane_ratios
    cx.notify() → Re-render with new flex_basis values

Mouse Up (global listener)
    ↓
Clear drag_state
    ↓
schedule_save() → Persist after 200ms debounce
```

## Implementation Plan

### File: `src/ui/workspace.rs`

#### New Methods

```rust
fn render_divider(&self, divider_index: usize, cx: &mut Context<Self>) -> impl IntoElement {
    let is_dragging = self.resize_drag_state
        .as_ref()
        .map_or(false, |s| s.divider_index == divider_index);

    div()
        .w(px(6.0))
        .h_full()
        .cursor_col_resize()
        .bg(if is_dragging {
            cx.theme().colors().border_focused
        } else {
            cx.theme().colors().border
        })
        .on_mouse_down(MouseButton::Left, cx.listener(move |this, event, _, cx| {
            this.start_resize_drag(divider_index, event.position.x, cx);
        }))
        .on_double_click(cx.listener(|this, _, _, cx| {
            this.reset_pane_ratios(cx);
        }))
}

fn start_resize_drag(&mut self, divider_index: usize, start_x: Pixels, cx: &mut Context<Self>) {
    let ws = self.workspace_state.read(cx);
    self.resize_drag_state = Some(ResizeDragState {
        divider_index,
        start_x,
        start_ratios: ws.pane_ratios,
        total_width: /* get from layout */,
    });
    cx.notify();
}

fn handle_resize_drag(&mut self, position_x: Pixels, cx: &mut Context<Self>) {
    if let Some(ref drag_state) = self.resize_drag_state {
        let delta = position_x - drag_state.start_x;
        let new_ratios = self.calculate_new_ratios(drag_state, delta);

        self.workspace_state.update(cx, |ws, _| {
            ws.pane_ratios = new_ratios;
        });
        cx.notify();
    }
}

fn end_resize_drag(&mut self, cx: &mut Context<Self>) {
    if self.resize_drag_state.take().is_some() {
        self.schedule_save(cx);
        cx.notify();
    }
}

fn calculate_new_ratios(&self, drag_state: &ResizeDragState, delta: Pixels) -> [f32; 3] {
    // 1. Convert ratios to pixel widths
    // 2. Apply delta to adjacent panes (divider_index and divider_index + 1)
    // 3. Clamp to MIN_PANE_WIDTH
    // 4. Normalize to maintain total
    // 5. Return new ratios
}

fn reset_pane_ratios(&mut self, cx: &mut Context<Self>) {
    self.workspace_state.update(cx, |ws, _| {
        ws.pane_ratios = [1.0, 2.0, 1.0]; // Default
    });
    self.schedule_save(cx);
    cx.notify();
}
```

#### Modified render_content()

```rust
fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
    let ws = self.workspace_state.read(cx);
    let ratios = ws.pane_ratios;
    let visible = [ws.file_browser_visible, ws.terminal_visible, ws.document_viewer_visible];

    // Calculate normalized ratios for visible panes only
    let visible_ratios = self.normalized_visible_ratios(&ratios, &visible);

    div()
        .flex()
        .flex_1()
        .w_full()
        // File browser
        .when(visible[0], |d| {
            d.child(
                div()
                    .flex_basis(relative(visible_ratios[0]))
                    .child(self.render_pane(PaneType::FileBrowser, cx))
            )
        })
        // Divider 0 (between file browser and terminal)
        .when(visible[0] && visible[1], |d| {
            d.child(self.render_divider(0, cx))
        })
        // Terminal
        .when(visible[1], |d| {
            d.child(
                div()
                    .flex_basis(relative(visible_ratios[1]))
                    .child(self.render_pane(PaneType::Terminal, cx))
            )
        })
        // Divider 1 (between terminal and doc viewer)
        .when(visible[1] && visible[2], |d| {
            d.child(self.render_divider(1, cx))
        })
        // Document viewer
        .when(visible[2], |d| {
            d.child(
                div()
                    .flex_basis(relative(visible_ratios[2]))
                    .child(self.render_pane(PaneType::DocumentViewer, cx))
            )
        })
}
```

#### Global Event Listeners

Register in `new()` or use `on_mouse_move_out` / `on_mouse_up_out` on the container:

```rust
// Option 1: Global listeners in new()
cx.on_mouse_event(move |this: &mut Self, event: &MouseMoveEvent, _, cx| {
    this.handle_resize_drag(event.position.x, cx);
});

cx.on_mouse_event(move |this: &mut Self, _event: &MouseUpEvent, _, cx| {
    this.end_resize_drag(cx);
});

// Option 2: On the content container div
.on_mouse_move(cx.listener(|this, event, _, cx| {
    this.handle_resize_drag(event.position.x, cx);
}))
.on_mouse_up(MouseButton::Left, cx.listener(|this, _, _, cx| {
    this.end_resize_drag(cx);
}))
```

## Edge Cases

| Case | Handling |
|------|----------|
| Pane below min width | Clamp to MIN_PANE_WIDTH, redistribute excess |
| Pane visibility toggle | Recalculate visible ratios, dynamic divider placement |
| Window resize | Ratios are relative, auto-adapts |
| Multiple dividers | Only one drag at a time (single drag_state) |
| Drag outside window | Global listener continues tracking |

## Build Sequence

1. **Foundation** - Add structs, fields, method stubs
2. **Divider Rendering** - Implement `render_divider()` with styling
3. **Drag Handling** - Implement state management and global listeners
4. **Layout Integration** - Modify `render_content()` for flex_basis
5. **Constraints** - Min-width enforcement, double-click reset
6. **Polish** - Visual feedback, hover states, testing

## Files Changed

- `src/ui/workspace.rs` - Main implementation
- `src/ui/workspace_config.rs` - No changes needed (pane_ratios already exists)

## Performance

- Target: 60fps during drag
- GPUI flex layout is optimized for frequent updates
- Debounced persistence prevents disk thrashing

---

## ARCH-CODEX Review Findings

Architecture review identified three issues to address as part of this implementation.

### HIGH: Missing Minimum-Width Guard in TerminalElement

**Location**: `src/terminal/element.rs:184-190`

**Problem**: When computing `TerminalBounds`, there's no guard against extremely narrow widths. If a pane is resized very narrow (0-1 columns), `set_size` produces a degenerate grid that causes alacritty to misbehave with rendering glitches or crashes.

**Reference**: Zed implements this guard in `terminal_element.rs:987-992`:
```rust
// https://github.com/zed-industries/zed/issues/2750
// if the terminal is one column wide, rendering 🦀
// causes alacritty to misbehave.
if size.width < cell_width * 2.0 {
    size.width = cell_width * 2.0;
}
```

**Fix**: Add minimum width clamping before creating `TerminalBounds`:

```rust
// In element.rs prepaint(), before TerminalBounds::new:

// Guard against narrow widths that cause alacritty to misbehave
// See: https://github.com/zed-industries/zed/issues/2750
let mut size = bounds.size;
if size.width < cell_width * 2.0 {
    size.width = cell_width * 2.0;
}
let clamped_bounds = Bounds { origin: bounds.origin, size };

let dimensions = TerminalBounds::new(line_height, cell_width, clamped_bounds);
```

---

### MEDIUM: Alt+Key Modifier Handling in Key Input

**Location**: `src/terminal/pane.rs:325-343`

**Problem**: The `key_char` fallback sends raw bytes even when Alt/Meta modifiers are present. Terminal applications expect Alt+key to send ESC followed by the key (e.g., Alt+b should send `\x1b` + `b` for backward-word in bash/readline).

**Current Code**:
```rust
} else if let Some(key_char) = &event.keystroke.key_char {
    // For plain text input, send the character directly to the terminal
    terminal.input(key_char.as_bytes().to_vec());
    cx.stop_propagation();
}
```

**Fix**: Check for Alt/Meta modifiers and prepend ESC when present:

```rust
} else if let Some(key_char) = &event.keystroke.key_char {
    let has_alt = event.keystroke.modifiers.alt;
    let has_meta = option_as_meta && event.keystroke.modifiers.platform;

    if has_alt || has_meta {
        // Alt/Meta + key should send ESC followed by the key
        let mut bytes = vec![0x1b]; // ESC
        bytes.extend_from_slice(key_char.as_bytes());
        terminal.input(bytes);
    } else {
        // Plain text input
        terminal.input(key_char.as_bytes().to_vec());
    }
    cx.stop_propagation();
}
```

**Impact**: Shell shortcuts like Alt+b (back word), Alt+f (forward word), Alt+d (delete word) will work correctly.

---

### LOW: Unused Code After TerminalElement Switch

**Locations**:
- `src/terminal/pane.rs:536-549` - `update_terminal_snapshot()`
- `src/terminal/pane.rs` - `build_lines_from_content()` (top-level function)
- `src/terminal/tab.rs:14,81-88` - `rendered_lines` field and accessors

**Problem**: After switching to `TerminalElement`, which builds lines directly from `last_content()` in its prepaint phase, the old snapshot/caching code is unused but still runs on terminal events, causing unnecessary allocations.

**Fix**: Remove dead code:

1. Delete `update_terminal_snapshot()` method from `TerminalPane`
2. Delete top-level `build_lines_from_content()` function (keep the one in `TerminalElement`)
3. Remove `rendered_lines` field from `TerminalTab` struct
4. Remove `set_rendered_lines()` and `rendered_lines()` methods from `TerminalTab`
5. Remove any event handler calls to `update_terminal_snapshot()`

---

## Updated Build Sequence

1. **Foundation** - Add structs, fields, method stubs
2. **Divider Rendering** - Implement `render_divider()` with styling
3. **Drag Handling** - Implement state management and global listeners
4. **Layout Integration** - Modify `render_content()` for flex_basis
5. **Constraints** - Min-width enforcement, double-click reset
6. **Terminal Hardening** - Implement ARCH-CODEX fixes:
   - Add minimum-width guard in `element.rs`
   - Fix Alt+key handling in `pane.rs`
   - Remove dead snapshot code
7. **Polish** - Visual feedback, hover states, testing

## Updated Files Changed

- `src/ui/workspace.rs` - Main resizable panes implementation
- `src/ui/workspace_config.rs` - No changes needed (pane_ratios already exists)
- `src/terminal/element.rs` - Add minimum-width guard
- `src/terminal/pane.rs` - Fix Alt+key handling, remove dead code
- `src/terminal/tab.rs` - Remove unused `rendered_lines` field
