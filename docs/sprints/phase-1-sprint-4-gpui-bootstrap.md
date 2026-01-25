# Phase 1.5: GPUI Bootstrap Checklist

**Status:** Ready to implement
**Estimated Time:** 3-5 hours
**Prerequisites:** Phase 1 complete (settings + theme system working)
**Target:** Minimal GPUI app with empty window showing theme colors

---

## Pre-Implementation Review

**Read these first:**
- [ ] Review `docs/architecture/gpui-integration.md` (Sections 1-3)
- [ ] Study Zed's `src/main.rs`: https://github.com/zed-industries/zed/blob/v0.220.3/src/main.rs
- [ ] Skim GPUI examples: https://github.com/zed-industries/zed/tree/v0.220.3/crates/gpui/examples

**Key concepts to understand:**
- GPUI application lifecycle (`App::new()`, `cx.open_window()`)
- View trait and rendering with `div()` builders
- Global state with `cx.set_global()`
- Event handling with `cx.listener()`

---

## CRITICAL LEARNINGS (2026-01-24)

### Dependency Resolution Issues

**Issue 1: core-graphics version conflict**
- GPUI pulls multiple versions of core-graphics (0.24, 0.25)
- Root cause: `core-text v21.1.0` (latest) requires `core-graphics v0.25`
- Zed v0.220.3 uses `core-text v21.0.0` which works with `core-graphics v0.24`
- **Solution:** Lock core-text version in Cargo.toml

```toml
[target.'cfg(target_os = "macos")'.dependencies]
core-text = "=21.0.0"
```

**Issue 2: Rust version requirement**
- GPUI v0.220.3 uses unstable features (`ceil_char_boundary`)
- Requires Rust 1.92 (not stable 1.88)
- **Solution:** Add `rust-toolchain.toml` to project root

```toml
[toolchain]
channel = "1.92"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

### System Dependencies (macOS)

**Metal Compiler Required:**
- GPUI compiles Metal shaders on macOS
- Requires full Xcode.app (Command Line Tools not sufficient)
- Fix: `sudo xcode-select --switch /Applications/Xcode.app/Contents/Developer`
- Verify: `xcrun --find metal` should return path

**cmake Required:**
- GPUI dependency wasmtime requires cmake
- Install: `brew install cmake`

### Correct GPUI API (v0.220.3)

**Wrong (what doesn't work):**
```rust
use gpui::{App, AppContext, ...};

App::new().run(|cx: &mut AppContext| {  // ❌ Wrong types
    cx.open_window(..., |cx| cx.new_view(...))  // ❌ Wrong API
})
```

**Correct (from Zed examples):**
```rust
use gpui::{Application, App, Window, Context, Render, ...};

Application::new().run(|cx: &mut App| {  // ✅ Application::new(), App type
    cx.open_window(
        WindowOptions { focus: true, ..Default::default() },
        |_, cx| cx.new(|_| MyView::new()),  // ✅ cx.new(), not cx.new_view()
    ).unwrap();
    cx.activate(true);
});

impl Render for MyView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // ✅ Takes Window and Context<Self>
        div().size_full().child("Hello")
    }
}
```

**Key differences:**
- `Application::new()` not `App::new()`
- Closure receives `&mut App` not `&mut AppContext`
- `cx.new(|_| view)` not `cx.new_view(|_| view)`
- `Render::render()` takes `&mut Window, &mut Context<Self>`

**Reference:** `/Users/randlee/Documents/github/zed/crates/gpui/examples/gradient.rs`

---

## Step 1: Update Cargo.toml

**File:** `Cargo.toml`

### 1.1 Add GPUI Dependency

```toml
[dependencies]
# UI Framework (Phase 1.5)
gpui = { git = "https://github.com/zed-industries/zed", tag = "v0.220.3" }
```

**Checklist:**
- [ ] Add GPUI git dependency with tag "v0.220.3"
- [ ] Keep existing dependencies (serde, dirs, anyhow, tracing, etc.)

### 1.2 Add Async Runtime

```toml
# Async Runtime (Phase 1.5)
smol = "2.0"
futures = "0.3"
```

**Checklist:**
- [ ] Add smol for async executor (matches Zed)
- [ ] Add futures for async utilities

### 1.3 Add macOS Dependency Lock

```toml
# macOS font dependencies - lock to Zed v0.220.3 compatible versions
[target.'cfg(target_os = "macos")'.dependencies]
core-text = "=21.0.0"
```

**Checklist:**
- [ ] Add core-text version lock for macOS
- [ ] This prevents core-graphics v0.24/v0.25 conflict

### 1.3 Verify Build

```bash
cargo clean  # Optional but recommended for GPUI first build
cargo check  # Should download and compile GPUI
```

**Checklist:**
- [ ] `cargo check` completes (may take 10-15 minutes first time)
- [ ] No errors (warnings OK for now)
- [ ] GPUI compiles successfully

**Expected output:**
```
Compiling gpui v0.1.0 (https://github.com/zed-industries/zed?tag=v0.220.3#...)
...
Finished `dev` profile [unoptimized + debuginfo] target(s) in 8m 34s
```

---

## Step 1.5: Create Rust Toolchain File

**File:** `rust-toolchain.toml` (project root)

Create this file to lock Rust version to 1.92:

```toml
[toolchain]
channel = "1.92"
profile = "minimal"
components = ["rustfmt", "clippy"]
```

**Checklist:**
- [ ] Create `rust-toolchain.toml` in project root
- [ ] Verify Rust 1.92 is installed: `rustup toolchain list`
- [ ] If not installed: `rustup toolchain install 1.92`
- [ ] Verify: `rustc --version` should show 1.92.x

---

## Step 2: Minimal GPUI App

**File:** `src/main.rs`

### 2.1 Update Imports

Add to top of `main.rs`:

```rust
use gpui::{
    div, prelude::*, px, rgb, size, App, Application, Bounds,
    Context, Render, Window, WindowBounds, WindowOptions,
};
```

**Checklist:**
- [ ] Import GPUI types (Application, App, Window, Context, Render)
- [ ] Import element builders (div, prelude, px, rgb, size)
- [ ] Keep existing imports (settings, theme, logging)

### 2.2 Create GPUI App

Replace or modify `fn main()`:

```rust
fn main() -> Result<()> {
    // Initialize logging (existing code)
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("info".parse()?))
        .init();

    tracing::info!("TerminalG starting...");

    // Load settings (existing code)
    let settings_store = SettingsStore::new()?;
    tracing::info!("Settings loaded from {:?}", settings_store.settings_path());

    // Load theme (existing code)
    let theme_name = &settings_store.settings().ui.theme;
    let theme = Theme::by_name(theme_name).unwrap_or_else(Theme::dark);
    tracing::info!("Loaded theme: {}", theme.name);

    // Initialize GPUI application
    Application::new().run(move |cx: &mut App| {
        // Store settings and theme in global state
        cx.set_global(settings_store);
        cx.set_global(theme);

        // Calculate centered window bounds (1200x800)
        let bounds = Bounds::centered(None, size(px(1200.0), px(800.0)), cx);

        // Open main window
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                titlebar: Some(gpui::TitlebarOptions {
                    title: Some("TerminalG".into()),
                    ..Default::default()
                }),
                focus: true,
                ..Default::default()
            },
            |_, cx| cx.new(|_| EmptyView),
        )
        .expect("Failed to open window");

        cx.activate(true);

        tracing::info!("TerminalG window opened successfully");
    });

    Ok(())
}
```

**Checklist:**
- [ ] Keep existing logging initialization
- [ ] Keep existing settings/theme loading
- [ ] Add `App::new().run()` call
- [ ] Store settings and theme in GPUI global state with `cx.set_global()`
- [ ] Open window with `cx.open_window()`
- [ ] Create a simple view (EmptyView for now)

### 2.3 Create EmptyView Struct

Add before `fn main()`:

```rust
/// Empty view for initial GPUI bootstrap
struct EmptyView;

impl Render for EmptyView {
    fn render(&mut self, _: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        // Get theme and settings from global state
        let theme = cx.global::<Theme>();
        let settings = cx.global::<SettingsStore>();

        // Convert our theme color to GPUI colors (RGB 0-255 -> 0xRRGGBB)
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
                    .child(format!("Theme: {}", settings.settings().ui.theme)),
            )
    }
}
```

**Checklist:**
- [ ] Create EmptyView struct
- [ ] Implement Render trait with correct signature: `(&mut Window, &mut Context<Self>)`
- [ ] Use `div()` builder for rendering
- [ ] Convert theme colors from RGB u8 to u32 hex format
- [ ] Apply theme background and foreground colors
- [ ] Display "TerminalG" text and theme name
- [ ] Use flexbox for centering

---

## Step 3: Build and Test

### 3.1 Build

```bash
cargo build
```

**Expected output:**
```
Compiling terminalg v0.1.0 (terminalg)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.34s
```

**Checklist:**
- [ ] Build succeeds without errors
- [ ] Warnings are acceptable (we'll fix later)
- [ ] Binary created: `target/debug/terminalg`

### 3.2 Run

```bash
cargo run
```

**Expected behavior:**
- [ ] Window opens with title "TerminalG" (or default)
- [ ] Window background uses theme color (from settings)
- [ ] Window is responsive (can move, resize)
- [ ] Text "TerminalG" appears in window
- [ ] No crashes or panics
- [ ] Logs show settings loaded

**Check logs:**
```
INFO terminalg: Loaded settings from: ~/.config/terminalg/settings.json
INFO terminalg: Loaded theme: dark
```

### 3.3 Test Window Behavior

**Manual tests:**
- [ ] Window opens
- [ ] Window can be moved
- [ ] Window can be resized
- [ ] Window can be closed (click X)
- [ ] Window closes cleanly (no crash)
- [ ] App exits cleanly (no zombie processes)

### 3.4 Test Theme Colors

**Test with different themes:**

1. Edit `~/.config/terminalg/settings.json`:
   ```json
   {
     "ui": { "theme": "light" }
   }
   ```
2. Run `cargo run`
3. Verify window background is light color

**Checklist:**
- [ ] Dark theme shows dark background
- [ ] Light theme shows light background
- [ ] Colors match theme definitions in `src/theme/mod.rs`

---

## Step 4: Add Window Title

### 4.1 Update WindowOptions

Modify the `cx.open_window()` call:

```rust
let window_options = WindowOptions {
    bounds: Some(gpui::Bounds {
        origin: Default::default(),
        size: gpui::Size {
            width: 1200.0,
            height: 800.0,
        },
    }),
    titlebar: Some(gpui::TitlebarOptions {
        title: Some("TerminalG".into()),
        ..Default::default()
    }),
    ..Default::default()
};

cx.open_window(window_options, |cx| {
    cx.new_view(|_cx| EmptyView)
});
```

**Checklist:**
- [ ] Set window size (1200x800 or your preference)
- [ ] Set window title to "TerminalG"
- [ ] Test: Window title appears correctly

---

## Step 5: Improve EmptyView (Optional)

Make the view more informative:

```rust
impl Render for EmptyView {
    fn render(&mut self, cx: &mut ViewContext<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();
        let settings = cx.global::<SettingsStore>();

        div()
            .flex()
            .flex_col()
            .items_center()
            .justify_center()
            .bg(theme.ui.background)
            .size_full()
            .child(
                div()
                    .text_2xl()
                    .text_color(theme.ui.foreground)
                    .child("TerminalG")
            )
            .child(
                div()
                    .text_sm()
                    .text_color(theme.ui.foreground)
                    .child(format!("Theme: {}", settings.ui.theme))
            )
    }
}
```

**Checklist:**
- [ ] Center content with flexbox
- [ ] Display app name prominently
- [ ] Display current theme name
- [ ] Use theme foreground color for text
- [ ] Test: Window shows centered text with theme info

---

## Step 6: Verify and Document

### 6.1 Final Verification

**Run all checks:**
```bash
cargo fmt        # Format code
cargo clippy     # Lint (warnings OK)
cargo build      # Build succeeds
cargo run        # Window opens correctly
```

**Checklist:**
- [ ] Code formatted with rustfmt
- [ ] Clippy warnings reviewed (fix critical ones)
- [ ] Build succeeds
- [ ] Window opens and works
- [ ] Theme colors applied correctly
- [ ] No crashes or panics

### 6.2 Commit Changes

```bash
git add Cargo.toml Cargo.lock src/main.rs
git commit -m "feat: Add GPUI bootstrap with empty window

- Add GPUI dependency (Zed v0.220.3)
- Initialize GPUI app with window
- Apply theme colors to window background
- Display app name and theme info
- Window opens at 1200x800 with title

Phase 1.5 complete: Basic GPUI integration working"
```

**Checklist:**
- [ ] Changes committed
- [ ] Commit message describes what was added
- [ ] Cargo.lock included in commit

### 6.3 Update Documentation

**Update `docs/IMPLEMENTATION-PLAN.md`:**
- [ ] Mark Phase 1.6 items as complete
- [ ] Add notes about any issues encountered
- [ ] Update "Next Session" section

**Update `README.md`:**
- [ ] Update project status (Phase 1.5 complete)
- [ ] Add note about GPUI version (v0.220.3)

---

## Troubleshooting

### Build Fails: "Could not find GPUI"

**Issue:** GPUI dependency not resolving

**Solutions:**
1. Check internet connection (git clone required)
2. Verify git is installed: `git --version`
3. Try: `cargo clean && cargo build`
4. Check git credentials for GitHub access

### Build Fails: Missing Platform Dependencies

**Issue:** Missing system libraries for GPUI

**macOS:**
```bash
xcode-select --install
```

**Linux:**
```bash
sudo apt-get install libssl-dev pkg-config  # Ubuntu/Debian
sudo dnf install openssl-devel pkgconfig    # Fedora
```

**Windows:**
- Install Visual Studio Build Tools
- Ensure Windows SDK installed

### Window Doesn't Open

**Issue:** App runs but no window appears

**Debug:**
1. Check logs: `RUST_LOG=debug cargo run`
2. Look for GPUI initialization errors
3. Check if window is off-screen (display configuration)
4. Try simpler WindowOptions (just `Default::default()`)

### Theme Colors Not Applied

**Issue:** Window background is default color, not theme color

**Debug:**
1. Verify settings loaded: Check log output
2. Verify theme in global state: Add debug print
3. Check if `cx.global::<Theme>()` works
4. Verify theme color values are correct (not transparent)

**Fix:**
- Ensure `cx.set_global(theme)` is called before `cx.open_window()`
- Check theme background color is opaque (alpha = 1.0)

### Slow Build Times

**Issue:** GPUI takes 10+ minutes to compile

**Expected:** First build is slow (8-15 minutes normal)

**Speed up future builds:**
1. Keep `target/` directory (don't run `cargo clean` unnecessarily)
2. Use incremental compilation (enabled by default in dev profile)
3. Consider using `sccache`: `cargo install sccache`
4. Use `cargo check` instead of `cargo build` during development

---

## Success Criteria

✅ **Phase 1.5 is complete when:**
- [ ] GPUI dependency added (v0.220.3)
- [ ] `cargo build` succeeds
- [ ] `cargo run` opens a window
- [ ] Window has "TerminalG" title
- [ ] Window background uses theme color
- [ ] Window is interactive (move, resize, close)
- [ ] No crashes or errors
- [ ] Code committed to git

---

## Next Steps: Phase 2 Preparation

**After Phase 1.5 complete:**

1. **Study Zed's Terminal Implementation**
   - Read: `crates/terminal/src/terminal.rs`
   - Understand PTY integration patterns
   - Note how they handle async PTY reads

2. **Plan Terminal Component**
   - Design TerminalPane struct
   - Plan integration with alacritty_terminal
   - Sketch rendering approach

3. **Update Implementation Plan**
   - Mark Phase 1.5 complete
   - Review Phase 2 checklist
   - Estimate Phase 2 timeline

**Don't start Phase 2 yet** - ensure Phase 1.5 is solid first!

---

## Reference Links

- **GPUI Architecture Doc**: `docs/GPUI-INTEGRATION-ARCHITECTURE.md`
- **Zed v0.220.3 Source**: https://github.com/zed-industries/zed/tree/v0.220.3
- **GPUI Examples**: https://github.com/zed-industries/zed/tree/v0.220.3/crates/gpui/examples
- **Zed's main.rs**: https://github.com/zed-industries/zed/blob/v0.220.3/src/main.rs
- **Implementation Plan**: `docs/IMPLEMENTATION-PLAN.md`

---

**Last Updated:** 2025-01-24
**Author:** ARCH-RTERM
**Status:** Ready for implementation
