# TerminalG - AI Agent Context

**Version:** 1.0
**Last Updated:** 2025-01-24

Notes for this repo:

- For Rust development tasks, use the local skill at `.claude/skills/rust-development/SKILL.md`.
- Agents defined in `.claude/agents/*` can be used inline as guidance or reference because Codex does not support sub-agents.

---

## Project Overview

GPU-accelerated terminal application with integrated artifact viewing, built on Rust and GPUI. Designed for interactive development workflows with rich visualization of test execution and agent behavior.

---

## Technology Stack

- **UI Framework:** GPUI (Zed v0.220.3, git dependency)
- **Terminal Emulation:** alacritty_terminal
- **Async Runtime:** smol
- **Language:** Rust 1.70+

---

## Primary Documentation

- **Requirements:** `docs/REQUIREMENTS.md`
- **Architecture:** `docs/ARCHITECTURE.md`
- **Master Plan:** `docs/MASTER-PLAN.md`

---

## Supporting Documentation

- **GPUI Integration:** `docs/architecture/gpui-integration.md`
- **Settings System:** `docs/architecture/settings-system.md`
- **Development Guide:** `docs/DEVELOPMENT-GUIDE.md`
- **Quick Start:** `docs/QUICK-START.md`

---

## Key Principles

1. Leverage Zed's proven patterns
2. GPU-accelerated performance
3. Workspace-level context switching
4. Integrated viewing (no browser tabs)
5. Keyboard-driven workflow

## Rules
1. DO NOT switch main working folder from develop branch
2. All development work to be done on worktrees created with sc-git-worktree skill.  Worktrees are located at ../terminalg-worktrees/
3. commit/push/pr when work is complete
4. DO NOT complete pr unless specifically requested by the user.  User will complete pr AFTER code review and ci passing 100%.