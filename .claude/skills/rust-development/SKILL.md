---
name: Rust Development
description: This skill enforces new code or code changes to conform to proper Rust guidelines and best practices
---

# Rust Development

This skill automatically enforces Rust coding standards and best practices when creating or modifying Rust code.

## Instructions

When asked about generating new code or making code changes to existing code, make
sure that edits are conformant to the [guidelines.txt](guidelines.txt) if the language to generate is Rust.

Key areas to enforce:
- Idiomatic Rust patterns and conventions
- Proper error handling (Result types, custom errors)
- Memory safety and lifetime management
- Async/await patterns when applicable
- FFI best practices for interop scenarios
- Documentation standards (rustdoc conventions)
- Testing patterns and practices

If the file is fully compliant with the guidelines, add a comment:
```rust
// Rust guideline compliant {date}
```
where {date} is the guideline date/version.

## When to Activate

This skill activates automatically when:
- Creating new Rust source files (.rs)
- Modifying existing Rust code
- Reviewing Rust code for compliance
- Answering questions about Rust best practices
- Working with Cargo projects

## Guidelines Source

The detailed guidelines are maintained in [guidelines.txt](guidelines.txt).
