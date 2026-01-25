# Phase 1.4: GPUI Bootstrap - Implementation Design

**Status:** Implemented
**Date:** 2026-01-24
**Sprint Duration:** 3-5 hours

---

## Architecture Decision: Minimal Bootstrap

**Chosen Approach:**
- Single view (`EmptyView`) displaying app name and theme info
- Direct global state management via `cx.set_global()`
- Simple color conversion: `theme::Color` (RGB u8) → `gpui::Rgb` (hex u32)
- Fixed 1200x800 centered window
- No complex event handling

**Rationale:**
- Lowest risk implementation
- Fast validation of GPUI integration
- Clear success criteria
- Solid foundation for Phase 1.5 (Workspace)

---

## Component Design

### EmptyView
```rust
struct EmptyView;

impl Render for EmptyView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let settings = cx.global::<SettingsStore>();

        // Convert theme colors to GPUI RGB format
        let bg_color = rgb(
            u32::from(theme.ui.background.r) << 16
                | u32::from(theme.ui.background.g) << 8
                | u32::from(theme.ui.background.b),
        );
        let fg_color = rgb(
            u32::from(theme.ui.foreground.r) << 16
                | u32::from(theme.ui.foreground.g) << 8
                | u32::from(theme.ui.foreground.b),
        );

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(bg_color)
            .size_full()
            .text_color(fg_color)
            .child(div().text_2xl().child("TerminalG"))
            .child(
                div()
                    .text_sm()
                    .child(format!("Theme: {}", settings.settings().ui.theme))
            )
    }
}
```

---

## Implementation Map

### Files Modified

**1. Cargo.toml**
- Added: `gpui = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }`
- Added: `smol = "2.0"`, `futures = "0.3"`

**2. src/main.rs**
- Added GPUI imports (`App`, `Bounds`, `Render`, etc.)
- Created `EmptyView` struct with `Render` trait
- Updated `main()` to initialize GPUI app
- Stored settings/theme in global context
- Configured window (1200x800, centered, titled)

---

## Data Flow

```
Application Startup:
  main()
    → Initialize logging
    → Load SettingsStore
    → Load Theme by name
    → App::new().run()
      → cx.set_global(settings)
      → cx.set_global(theme)
      → cx.open_window()
        → cx.new_view(EmptyView)

Render Flow:
  EmptyView::render()
    → cx.global::<Theme>()
    → cx.global::<SettingsStore>()
    → Build element tree with flexbox
    → Apply theme colors
```

---

## Key Integration Points

### Global State Pattern
- Settings and theme stored via `cx.set_global()`
- Retrieved in views via `cx.global::<T>()`
- Type-safe, matches Zed's pattern

### Color Conversion
- Theme uses RGB u8 (0-255)
- GPUI uses u32 hex (0xRRGGBB)
- Conversion: `(r << 16) | (g << 8) | b`

### Window Management
- `Bounds::centered()` for initial placement
- `WindowOptions` for size/title configuration
- `cx.new_view()` for view creation

---

## Success Criteria

- [x] GPUI dependency added (v0.220.3)
- [x] Code compiles successfully
- [ ] Window opens with "TerminalG" title
- [ ] Background uses theme color
- [ ] Window is interactive (move, resize, close)
- [ ] No crashes or errors
- [ ] Code formatted and linted

---

## Known Issues

### Dependency Conflicts
**Issue:** core-graphics version mismatch in font-kit
**Resolution:** Using cargo update to resolve dependencies

---

## Next Steps (Phase 1.5)

After Sprint 1.4 complete:
1. Add workspace tab system
2. Implement 3-pane layout
3. Add workspace configuration persistence
4. Prepare for terminal integration (Phase 2)
