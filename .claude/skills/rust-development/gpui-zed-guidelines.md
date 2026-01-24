# GPUI/Zed Development Guidelines

This document captures project-specific guidelines for building GPUI-based features with Zed as the upstream reference.

## Dependency Policy

- Pin `gpui` to a Zed release tag for reproducible builds.
- Avoid `branch = "main"` for production dependencies.
- Document upgrades in changelog/release notes and test before merging.

## Concurrency and Threading

- Do not block the GPUI render thread.
- Run PTY and other I/O in background tasks.
- Use channels to move data into the UI thread and call `cx.notify()` on state changes.
- For long-running CPU tasks, use cooperative yield points (`yield_now().await`).

## State Management

- Store shared state in globals via `cx.set_global()` (settings, theme).
- Observe global state changes with `cx.observe_global()` and re-render on updates.
- Prefer `Model<T>` for view-local state and mutations.

## Rendering Performance

- Avoid per-cell element creation in hot paths when possible.
- Batch adjacent cells with identical styles.
- Consider dirty-region updates to reduce full-grid re-renders.

## Error Handling

- Application crates may use `anyhow`/`eyre` for application-level errors.
- Library crates should use structured error types with `Display` + `Error`.

## Testing and Quality Gates

- Run `cargo build`, `cargo clippy`, and `cargo test` before merging.
- Perform manual window behavior checks (open, resize, close, theme changes).

## Platform Notes

- macOS and Linux first; Windows PTY/ConPTY integration is a separate milestone.
- Keep Zed version targets explicit in docs and dependency manifests.
