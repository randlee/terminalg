# TerminalG Quick Start Guide

Get TerminalG building and running in 5 minutes.

## Prerequisites

### System Requirements
- **Rust 1.70+** - https://rustup.rs/
- **macOS 10.15+**, **Linux** (glibc 2.31+), or **Windows 10+**
- **Git**

### Platform-Specific Dependencies

**macOS:**
```bash
# Nothing additional required - everything is built-in
```

**Linux (Ubuntu/Debian):**
```bash
sudo apt-get install -y build-essential pkg-config libssl-dev
```

**Linux (Fedora/RHEL):**
```bash
sudo dnf install -y gcc pkg-config openssl-devel
```

**Windows:**
- Visual Studio Build Tools or Visual Studio Community
- Windows SDK (includes ConPTY headers)

## Setup

### 1. Clone the Repository
```bash
cd /Users/randlee/Documents/github
git clone <url> terminalg  # if not already cloned
cd terminalg
```

### 2. Verify Rust Setup
```bash
rustc --version  # Should be 1.70+
cargo --version
```

### 3. Build the Project
```bash
cargo build
```

This will:
- Download all dependencies
- Compile the project
- Create binary at `target/debug/terminalg`

### 4. Create Settings File
```bash
cargo run
```

This will:
- Initialize settings at `~/.config/terminalg/settings.json`
- Create default configuration
- Print debug output

### 5. View Settings
```bash
cat ~/.config/terminalg/settings.json
```

Expected output:
```json
{
  "terminal": {
    "font_size": 12,
    "font_family": "Menlo",
    "padding": { "horizontal": 8, "vertical": 8 },
    "enable_url_recognition": true,
    "scrollback_lines": 10000,
    "shell": null
  },
  "ui": {
    "theme": "dark",
    "terminal_width_ratio": 0.5,
    "show_preview": true,
    "autosave_interval": 0
  }
}
```

## Development Commands

```bash
# Build debug version
cargo build

# Build release (optimized, smaller binary)
cargo build --release

# Run with logging
RUST_LOG=debug cargo run

# Run tests
cargo test

# Check for issues without building
cargo check

# Format code
cargo fmt

# Lint code
cargo clippy

# Clean build artifacts
cargo clean
```

## Project Structure

```
terminalg/
├── src/
│   ├── main.rs                 # Entry point
│   ├── settings/               # Settings system
│   │   ├── mod.rs
│   │   ├── terminal.rs
│   │   └── ui.rs
│   ├── theme/                  # Theme definitions
│   │   └── mod.rs
│   ├── terminal/               # Terminal component (Phase 2)
│   │   └── mod.rs
│   ├── ui/                     # UI layout (Phase 3)
│   │   └── mod.rs
│   └── viewer/                 # Markdown/image viewers (Phase 3)
│       └── mod.rs
├── docs/
│   ├── QUICK-START.md         # This file
│   ├── IMPLEMENTATION-PLAN.md  # Detailed roadmap
│   ├── settings-architecture.md
│   └── CLAUDE.md
├── Cargo.toml                  # Dependencies
└── src/main.rs
```

## Common Issues

### Cargo build fails: "cannot find -lpty"
PTY library not found. Check platform-specific section above.

### Settings file not created
Make sure `~/.config/` directory exists:
```bash
mkdir -p ~/.config
```

### RUST_LOG not showing output
Set log level explicitly:
```bash
RUST_LOG=terminalg=debug cargo run
```

## Next Steps

1. Read `docs/IMPLEMENTATION-PLAN.md` for detailed roadmap
2. Check `src/main.rs` for TODOs
3. Start Phase 1 implementation
4. Update progress in `CLAUDE.md`

## Useful References

- **GPUI Examples**: https://github.com/zed-industries/zed/tree/main/crates/gpui/examples
- **Zed Terminal Source**: https://github.com/zed-industries/zed/tree/main/crates/terminal/src
- **Rust Book**: https://doc.rust-lang.org/book/
- **Cargo Guide**: https://doc.rust-lang.org/cargo/

## Getting Help

- Check logs: `RUST_LOG=debug cargo run`
- Read error messages carefully (Rust compiler is very helpful)
- Consult platform-specific setup section above
- Check existing issues/PRs in the repo
