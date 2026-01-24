# TerminalG

A GPU-accelerated terminal UI with integrated artifact viewing, built on Rust and GPUI.

---

## What is TerminalG?

TerminalG is a purpose-built terminal application that combines powerful terminal emulation with rich artifact visualization in a unified workspace. No more switching between terminal and browser tabs to view test results, images, or documentation.

**Key Features:**
- 🚀 **GPU-Accelerated** - Smooth, responsive UI powered by GPUI
- 📺 **Integrated Viewers** - View markdown, images (including TIFF), and test results inline
- 🗂️ **Workspace Tabs** - Switch entire contexts, not just files
- 🎨 **Themes** - Built-in dark/light themes with hot-reload
- ⚙️ **Configurable** - JSON-based settings for everything
- 🔗 **URL Recognition** - Click links directly in terminal output

---

## Quick Start

```bash
# Build the project
cargo build

# Run (creates settings file automatically)
cargo run

# View generated settings
cat ~/.config/terminalg/settings.json
```

See [**Quick Start Guide**](docs/QUICK-START.md) for detailed setup instructions.

---

## Project Status

**Current Phase:** Foundation (Phase 1 - 70% complete)

- ✅ Settings system
- ✅ Theme system
- ⏳ GPUI integration (in progress)
- ⬜ Terminal emulation (Phase 2)
- ⬜ UI layout & viewers (Phase 3)
- ⬜ Enhancements (Phase 4)

**Timeline:** MVP estimated 10-16 days of development

---

## Documentation

### For Users

- **[Quick Start](docs/QUICK-START.md)** - Build, run, and configure
- **[Requirements](docs/REQUIREMENTS.md)** - What we're building and why
- **[Development Guide](docs/DEVELOPMENT-GUIDE.md)** - Contributing and development workflow

### For Developers

- **[Architecture](docs/ARCHITECTURE.md)** - System design and component overview
- **[Master Plan](docs/MASTER-PLAN.md)** - Phases, sprints, and implementation roadmap
- **[GPUI Integration](docs/architecture/gpui-integration.md)** - GPUI integration strategy
- **[Settings System](docs/architecture/settings-system.md)** - Settings architecture

---

## Why TerminalG?

Existing tools force workflows into constrained patterns:

- **Zed** - Editor-centric, not terminal-first
- **Warp** - Closed-source, not extensible
- **Terminal + Browser** - Fragmented workflow with constant tab switching
- **HTML Test Reports** - Opens new browser tabs for every test run

**TerminalG solves this** by providing a unified workspace where terminal output, test results, images, and documentation all live side-by-side.

---

## Technology

Built with modern Rust tooling:

- **GPUI** - GPU-accelerated UI framework (from Zed)
- **alacritty_terminal** - Proven terminal emulation
- **smol** - Lightweight async runtime
- **serde** - Configuration management

---

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS | ✓ Primary | 10.15+ |
| Linux | ✓ Supported | glibc 2.31+, X11/Wayland |
| Windows | ⏳ Future | 10+, ConPTY support |

---

## Requirements

- **Rust** 1.70+ - Install from https://rustup.rs/
- **Build tools:**
  - macOS: Xcode Command Line Tools
  - Linux: GCC/Clang + pkg-config
  - Windows: Visual Studio Build Tools

---

## Configuration

Settings are stored in platform-specific directories:

- **macOS/Linux:** `~/.config/terminalg/settings.json`
- **Windows:** `%APPDATA%\terminalg\settings.json`

Example settings:

```json
{
  "terminal": {
    "font_size": 12,
    "font_family": "Menlo",
    "enable_url_recognition": true,
    "scrollback_lines": 10000
  },
  "ui": {
    "theme": "dark",
    "terminal_width_ratio": 0.5,
    "show_preview": true
  }
}
```

---

## Development

```bash
# Build
cargo build              # Debug
cargo build --release    # Optimized

# Test
cargo test               # Run tests
cargo clippy             # Lint check
cargo fmt                # Format code

# Run with logging
RUST_LOG=info cargo run
RUST_LOG=debug cargo run
```

See [**Development Guide**](docs/DEVELOPMENT-GUIDE.md) for complete workflow.

---

## Roadmap

### Phase 1: Foundation ⏳
- Settings & theme system
- Basic GPUI application
- Window with theme colors

### Phase 2: Terminal 📋
- PTY management
- Terminal emulation (alacritty_terminal)
- Scrollback buffer
- Theme integration

### Phase 3: UI & Viewers 📋
- Workspace tabs
- File browser pane
- Markdown viewer
- Image viewer (PNG, JPEG, GIF, SVG, TIFF)

### Phase 4: Enhancements 📋
- URL recognition & clicking
- Settings hot-reload
- Test visualization
- MCP integration layer

See [**Master Plan**](docs/MASTER-PLAN.md) for detailed roadmap.

---

## Contributing

This is an active development project. Contributions welcome!

1. Read [**Development Guide**](docs/DEVELOPMENT-GUIDE.md)
2. Create feature branch: `git checkout -b feature/description`
3. Make changes and test
4. Commit with clear messages
5. Push and create PR

---

## License

MIT OR Apache-2.0

---

## Resources

- [Zed Editor](https://zed.dev/) - Inspiration and GPUI framework
- [GPUI Documentation](https://github.com/zed-industries/zed/tree/main/crates/gpui)
- [Alacritty Terminal](https://github.com/alacritty/alacritty)
- [Rust Book](https://doc.rust-lang.org/book/)

---

## Contact

**Documentation:**
- [Requirements](docs/REQUIREMENTS.md) - Vision and goals
- [Architecture](docs/ARCHITECTURE.md) - Technical design
- [Master Plan](docs/MASTER-PLAN.md) - Development roadmap

**Questions or ideas?** See the documentation above or create an issue.
