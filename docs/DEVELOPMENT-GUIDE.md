# TerminalG Development Guide

Conventions and workflow for contributing to TerminalG.

## Code Style

### Rust Conventions
- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` before committing (automatic formatting)
- Pass `cargo clippy` checks (linting)
- Add comments for non-obvious logic

### Module Organization
```rust
// 1. Module and crate imports
mod submodule;
use std::path::Path;
use crate::other::module;

// 2. Type definitions
pub struct MyStruct { ... }
pub enum MyEnum { ... }

// 3. Trait implementations
impl MyStruct { ... }
impl Trait for MyStruct { ... }

// 4. Tests (if in same file)
#[cfg(test)]
mod tests { ... }
```

### Documentation
Every public item should have a doc comment:

```rust
/// Brief description.
///
/// Longer explanation if needed.
///
/// # Examples
///
/// ```
/// // Example code
/// ```
///
/// # Errors
///
/// Returns error if...
pub fn my_function() -> Result<()> {
    // implementation
}
```

### Error Handling
- Use `anyhow::Result<T>` for functions that can fail
- Use `.context("description")` to add context to errors
- Avoid `.unwrap()` and `.expect()` in library code

```rust
use anyhow::{Context, Result};

pub fn load_file(path: &Path) -> Result<String> {
    std::fs::read_to_string(path)
        .context(format!("Failed to read file: {:?}", path))
}
```

## Git Workflow

### Branch Naming
- **Feature**: `feature/description` (e.g., `feature/url-recognition`)
- **Fix**: `fix/issue-name` (e.g., `fix/terminal-crash`)
- **Docs**: `docs/something` (e.g., `docs/api-reference`)
- **Refactor**: `refactor/what` (e.g., `refactor/settings-system`)

### Commits
- Use imperative mood: "Add feature" not "Added feature"
- Keep commits focused and atomic
- Reference issues if applicable: "Fix #123"

```bash
# Bad
git commit -m "stuff"
git commit -m "Fixed some bugs and added new features"

# Good
git commit -m "Add URL recognition in terminal output"
git commit -m "Fix terminal rendering crash on long lines"
```

### Pull Request Process
1. Create feature branch: `git checkout -b feature/description`
2. Make changes and commit regularly
3. Push: `git push origin feature/description`
4. Create PR with description of changes
5. Ensure all checks pass (build, tests, lint)
6. Merge to main

## Build & Test Workflow

### Before Committing
```bash
# Check for compilation errors
cargo check

# Run tests
cargo test

# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings
```

### Build Targets
```bash
# Debug build (default, fast compile, slow runtime)
cargo build

# Release build (slow compile, fast runtime)
cargo build --release

# Minimal build (check only, no binary)
cargo check
```

## Implementation Workflow

### Starting a Phase
1. Read the phase section in `IMPLEMENTATION-PLAN.md`
2. Create feature branch: `git checkout -b feature/phase-X-description`
3. Create module stubs if needed
4. Write tests first (TDD approach)
5. Implement functionality
6. Ensure tests pass: `cargo test`
7. Check lints: `cargo clippy`
8. Update CLAUDE.md progress
9. Create commit(s) for reviewability

### During Implementation
1. Run `cargo check` frequently (catches errors early)
2. Keep functions small and focused
3. Add tests as you go, not at the end
4. Log important operations: `tracing::info!("message")`

### Checkpoints
At each phase checkpoint:
- [ ] All tests passing: `cargo test`
- [ ] No compiler warnings: `cargo check`
- [ ] Clippy clean: `cargo clippy`
- [ ] All phase items completed
- [ ] Update CLAUDE.md with status
- [ ] Git commit with phase tag: `git commit -m "Complete Phase X"`
- [ ] Git tag: `git tag phase-X-complete`

## Logging

Use the `tracing` crate for debug output:

```rust
use tracing::{info, debug, warn, error};

info!("Important operation completed");
debug!("Detailed debug information: {:?}", value);
warn!("Warning about potential issue");
error!("Error occurred: {}", err);
```

Set log level when running:
```bash
# Info and higher
cargo run

# Debug and higher
RUST_LOG=debug cargo run

# Specific module
RUST_LOG=terminalg::terminal=debug cargo run

# Multiple modules
RUST_LOG=terminalg::settings=debug,terminalg::terminal=info cargo run
```

## Platform-Specific Code

Use conditional compilation for platform-specific code:

```rust
#[cfg(target_os = "macos")]
fn macos_specific() {
    // macOS code
}

#[cfg(target_os = "linux")]
fn linux_specific() {
    // Linux code
}

#[cfg(target_os = "windows")]
fn windows_specific() {
    // Windows code
}

#[cfg(unix)]
fn unix_code() {
    // Unix (macOS + Linux)
}

#[cfg(not(windows))]
fn non_windows() {
    // Not Windows
}
```

Test on all platforms before merge:
```bash
# macOS
cargo test

# Linux (if available)
docker run --rm -v $(pwd):/work -w /work rust:latest cargo test

# Windows (if available)
cargo test
```

## Keyboard Shortcuts (When Implemented)

**Workspace Navigation**
- `Ctrl+1` - Focus terminal pane
- `Ctrl+2` - Focus viewer pane
- `Ctrl+Tab` - Next workspace tab
- `Ctrl+Shift+Tab` - Previous workspace tab
- `Ctrl+N` - New workspace
- `Ctrl+W` - Close workspace

**Terminal**
- `Ctrl+C` - Send interrupt
- `Ctrl+L` - Clear screen
- `Cmd+K` (macOS) / `Ctrl+L` - Clear scrollback
- `Cmd+F` (macOS) / `Ctrl+F` - Search output

**Viewers**
- `+` / `-` - Zoom in/out
- `Ctrl+0` - Reset zoom
- `Space` - Fit to window

**General**
- `Ctrl+,` - Open settings
- `Ctrl+Shift+T` - Cycle themes
- `Ctrl+Shift+L` - Toggle light/dark theme

## Testing

### Writing Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_description() {
        // Arrange
        let input = "test";

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result, "expected");
    }
}
```

Run tests:
```bash
# Run all tests
cargo test

# Run specific test
cargo test test_name

# Run with output
cargo test -- --nocapture

# Run only unit tests
cargo test --lib

# Run integration tests
cargo test --test '*'
```

## Performance

Use profiling for performance-critical sections:

```bash
# Flamegraph (requires cargo-flamegraph)
cargo install flamegraph
cargo flamegraph --release -- <args>

# Valgrind (Linux)
valgrind ./target/release/terminalg
```

## Security

- Use `cargo audit` to check for vulnerable dependencies
- Validate user input at boundaries
- Use types to enforce correctness (leverage Rust's type system)
- Avoid `unsafe` code unless absolutely necessary

## Common Tasks

### Add a new dependency
```bash
cargo add crate_name
# or
cargo add crate_name@version
# or manually edit Cargo.toml and run cargo build
```

### Update dependencies
```bash
cargo update
```

### Generate documentation
```bash
cargo doc --open
```

### Bench code
```bash
cargo bench  # if benchmarks exist
```

## Troubleshooting

### Compiler error about lifetimes
Read the error message carefully - Rust's compiler is excellent at explaining lifetime issues.

### Test failures
- Run with `-- --nocapture` to see println output
- Run single test: `cargo test test_name -- --nocapture`
- Use debugger: `rust-gdb target/debug/terminalg`

### Slow compilation
- Use `cargo check` instead of `cargo build` during development
- Use incremental compilation: `CARGO_INCREMENTAL=1`
- Consider splitting into more crates if it gets very large

### Platform-specific build failures
- Test on each platform before committing
- Use CI/CD to catch platform issues
- Check Cargo features for platform dependencies
