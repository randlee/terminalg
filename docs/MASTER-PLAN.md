# TerminalG Master Plan

**Version:** 1.2
**Last Updated:** 2026-01-25
**Current Phase:** Phase 2 (Sprint 2.2 complete - PR #14 pending)
**Dependency Strategy:** See `docs/architecture/zed-reuse-strategy.md`
**License:** GPL-3.0-or-later (required by Zed crate dependencies)

---

## 1. Overview

Development organized into 5 phases, each containing sprints that can be executed sequentially or in parallel where dependencies allow.

**Total Estimated Effort:** 80-110 hours
**Timeline:** 10-16 days full-time, or 2-3 weeks part-time

---

## 2. Phase Breakdown

**Aligned with priorities:**
1. App framework w/ empty windows
2. Fully working Zed terminal
3. Add file/folder browser
4. Add markdown viewer (preview)
5. Markdown viewer + editing (MVP)

| Phase | Name | Status | Estimated Hours | Sprints |
|-------|------|--------|-----------------|---------|
| 1 | Foundation & Workspace | ✅ Complete | 20-25 | 5 |
| 2 | Zed Terminal Integration | 66% (Sprint 2.2 complete) | 20-30 | 3 |
| 3 | File/Folder Browser | Not Started | 15-20 | 2 |
| 4 | Markdown Viewer | Not Started | 15-20 | 2 |
| 5 | Markdown Editor (MVP) | Not Started | 10-15 | 2 |

---

## 3. PHASE 1: Foundation & Workspace

**Goal:** Establish project structure, settings system, theme system, GPUI app with workspace tabs, and workspace configuration.

**Priority:** 1 - App framework w/ empty windows

**Status:** 95% complete (Settings ✓, Theme ✓, GPUI Bootstrap ✓, Workspace Tabs PR pending)

**Dependencies:** None

### Sprints

#### Sprint 1.1: Project Setup ✅ COMPLETE
**Duration:** 2-3 hours
**Status:** Complete

- [x] Initialize Cargo project
- [x] Create directory structure (src/, docs/)
- [x] Create .gitignore
- [x] Initialize git repo
- [x] Create README.md
- [x] Add base dependencies to Cargo.toml

#### Sprint 1.2: Settings System ✅ COMPLETE
**Duration:** 4-6 hours
**Status:** Complete

- [x] Create `src/settings/mod.rs` - SettingsStore
- [x] Create `src/settings/terminal.rs` - TerminalSettings
- [x] Create `src/settings/ui.rs` - UiSettings
- [x] Implement load/save/reload methods
- [x] Platform-specific config paths (macOS/Linux/Windows)
- [x] JSON serialization/deserialization
- [x] Default values
- [x] Test: Settings created on first run
- [x] Test: Settings loaded from disk
- [x] Test: Settings file location correct

**Supporting Doc:** `docs/architecture/settings-system.md`

#### Sprint 1.3: Theme System ✅ COMPLETE
**Duration:** 3-4 hours
**Status:** Complete

- [x] Create `src/theme/mod.rs` - Color, Theme structs
- [x] Define ANSI colors (16 standard)
- [x] Define UI colors (foreground, background, accent, etc.)
- [x] Define status colors (success, error, warning, info)
- [x] Implement dark theme (default)
- [x] Implement light theme
- [x] Theme::by_name() lookup with fallback
- [x] Test: Load themes without errors
- [x] Test: Theme colors applied

#### Sprint 1.4: GPUI Bootstrap ✅ COMPLETE
**Duration:** 3-5 hours
**Status:** Complete

**Detailed Checklist:** `docs/sprints/phase-1-sprint-4-gpui-bootstrap.md`

**High-Level Tasks:**
- [x] Add GPUI dependency (tag = v0.220.3) to Cargo.toml
- [x] Add smol and futures dependencies
- [x] Update main.rs with GPUI app initialization
- [x] Create empty window with title "TerminalG"
- [x] Apply theme background color to window
- [x] Test: Window opens
- [x] Test: Window interactive (move, resize, close)
- [x] Test: No crashes

**Supporting Doc:** `docs/architecture/gpui-integration.md`
**QA Report:** `docs/sprints/phase-1-sprint-4-qa.md`

#### Sprint 1.5: Workspace Tabs & Configuration ✅ COMPLETE (PR #12)
**Duration:** 4-6 hours
**Status:** Complete - PR pending merge

**Design Doc:** `docs/sprints/phase-1-sprint-5-design.md`

**High-Level Tasks:**
- [x] Create WorkspaceConfig struct (save/load workspace state)
- [x] Implement workspace tab bar UI (top tabs)
- [x] Implement workspace switching
- [x] Create placeholder pane layout (3 panes: File Browser, Terminal, Document Viewer)
- [x] Add pane visibility controls (hide/show buttons)
- [x] Auto-save workspace config on changes (200ms debounce)
- [x] Default workspace: file browser + terminal visible
- [x] Config validation (bounds checking, empty workspace handling)
- [x] Git repo optional (falls back to current directory)
- [x] Test: Workspace tabs switch
- [x] Test: Workspace config persists
- [x] Test: 54 tests passing (11 new workspace_config tests)

**PR:** https://github.com/randlee/terminalg/pull/12
**Branch:** `feature/sprint-1-5-workspace-tabs`

### Phase 1 Checkpoint

**Complete when:**
- [x] App-wide settings system working (load/save/reload)
- [x] Settings file created in correct location
- [x] Theme system working (dark/light themes)
- [x] `cargo check` passes
- [x] `cargo build` succeeds
- [x] GPUI window opens with theme colors
- [x] Workspace tabs functional (UI, switching) - PR #12
- [x] Workspace configuration system working (save/load) - PR #12
- [x] Three placeholder panes rendering - PR #12
- [x] Pane visibility controls work (hide/show) - PR #12
- [ ] No crashes or errors - pending manual testing after PR merge

**Ready for:** Phase 2 (Zed Terminal Integration) - after PR #12 merge

---

## 4. PHASE 2: Zed Terminal Integration

**Goal:** Integrate Zed's terminal crate via git dependency, migrate to Zed's settings/theme, add URL recognition.

**Priority:** 2 - Fully working Zed terminal (all features, all platforms)

**Status:** 66% complete (Sprint 2.1 ✅, Sprint 2.2 ✅, Sprint 2.3 pending)

**Dependencies:** Phase 1 complete ✅

**Estimated:** 20-30 hours

**Key Decision:** Use Zed crates as git dependencies (NOT copy/vendor). See `docs/architecture/zed-reuse-strategy.md`.

**Design Doc:** `docs/sprints/phase-2-sprint-2-design.md`

**PR:** https://github.com/randlee/terminalg/pull/14

**Branch:** `feature/sprint-2-terminal-integration`

### Sprints

#### Sprint 2.1: Add Zed Dependencies & Migrate Settings/Theme ✅ COMPLETE
**Duration:** 6-8 hours
**Status:** Complete

**Completed Tasks:**
- [x] Add Zed crates to Cargo.toml as git dependencies:
  - `terminal` (GPL-3.0) - core terminal emulation
  - `settings` (GPL-3.0) - required by terminal
  - `theme` (GPL-3.0) - required by terminal
  - `ui` (GPL-3.0) - UI components
  - `util` - shell utilities
  - `collections` - HashMap collections
- [x] Create `src/settings_adapter.rs` - bridge to Zed settings
- [x] Create `src/theme_adapter.rs` - theme initialization
- [x] Update `src/main.rs` with Zed system initialization
- [x] Update WorkspaceView to use `cx.theme()` colors
- [x] All 60 tests passing
- [x] App launches with Zed theme colors

**Files Created:**
- `src/settings_adapter.rs` (~50 lines)
- `src/theme_adapter.rs` (~70 lines)

#### Sprint 2.2: Terminal Pane Integration ✅ COMPLETE
**Duration:** 8-12 hours
**Status:** Complete

**Completed Tasks:**
- [x] Create `src/terminal/pane.rs` - TerminalPane wrapping Zed Terminal
- [x] Create `src/terminal/tab.rs` - TerminalTab state management
- [x] Update `src/terminal/mod.rs` - module exports
- [x] Integrate TerminalPane into WorkspaceView
- [x] Terminal spawns with workspace root as working directory
- [x] Basic terminal content rendering from TerminalContent cells
- [x] Keystroke-to-terminal input conversion
- [x] Multiple terminal tabs with tab bar UI
- [x] Terminal event handling (title changes, close, wakeup, bell)
- [x] All 60 tests passing, clippy clean

**Files Created:**
- `src/terminal/pane.rs` (~360 lines)
- `src/terminal/tab.rs` (~70 lines)

**Files Modified:**
- `src/terminal/mod.rs`
- `src/ui/workspace.rs`

#### Sprint 2.3: URL Recognition & Clicking
**Duration:** 6-10 hours
**Parallel:** No (depends on Sprint 2.2)
**Status:** Not Started

**High-Level Tasks:**
- [ ] Add URL regex detection to terminal output
- [ ] Store URL → screen region mapping
- [ ] Implement click detection on URLs
- [ ] Open URL in default browser
- [ ] Style URLs (color, underline on hover)
- [ ] Test: URLs detected correctly
- [ ] Test: Clicking URLs opens browser
- [ ] Test: Works across scrolling

**Key Point:** Design for potential PR back to Zed - keep implementation clean and modular

### Phase 2 Checkpoint

**Complete when:**
- [x] PTY spawning and lifecycle managed by Zed ✅
- [x] Terminal renders content ✅
- [x] Keyboard input works ✅
- [x] Terminal tabs functional ✅
- [x] Theme applied correctly ✅
- [x] Settings applied correctly ✅
- [ ] GPU-accelerated rendering (basic text rendering complete, GPU optimization future)
- [ ] URL recognition and clicking (Sprint 2.3)
- [ ] Copy/paste, mouse, search (future enhancement)

**Ready for:** Phase 3 (File Browser) after Sprint 2.3 or can proceed in parallel

---

## 5. PHASE 3: File/Folder Browser

**Goal:** Implement file/folder tree browser pane with navigation and file selection.

**Priority:** 3 - Add file/folder browser

**Status:** Not started

**Dependencies:** Phase 1-2 complete

**Estimated:** 15-20 hours

### Sprints

#### Sprint 3.1: File Browser Core
**Duration:** 8-10 hours
**Parallel:** No

**Need to plan sprint:** Detailed implementation checklist

**High-Level Tasks:**
- [ ] Create `src/ui/file_browser.rs` - FileBrowserPane
- [ ] Implement file tree data structure
- [ ] Display file/folder tree (rooted at workspace folder)
- [ ] Implement expand/collapse directories
- [ ] Implement file selection
- [ ] Apply theme colors
- [ ] Test: Browse directories
- [ ] Test: Expand/collapse works
- [ ] Test: File selection works

#### Sprint 3.2: File Browser Integration
**Duration:** 6-8 hours
**Parallel:** No (depends on Sprint 3.1)

**Need to plan sprint:** Detailed implementation checklist

**High-Level Tasks:**
- [ ] Integrate with workspace configuration (persist state)
- [ ] Add file icons/indicators
- [ ] Implement refresh functionality
- [ ] Add keyboard navigation (arrow keys)
- [ ] Connect to document viewer (select file → open)
- [ ] Test: File browser → document viewer integration
- [ ] Test: State persists across restarts

### Phase 3 Checkpoint

**Complete when:**
- [ ] File/folder browser displays tree correctly
- [ ] Expand/collapse functional
- [ ] File selection works
- [ ] Integrated with workspace config
- [ ] Keyboard navigation works
- [ ] Opens files in document viewer
- [ ] Theme applied correctly

**Ready for:** Phase 4 (Markdown Viewer)

---

## 6. PHASE 4: Markdown Viewer

**Goal:** Implement markdown preview/rendering in document viewer pane.

**Priority:** 4 - Add markdown viewer (preview)

**Status:** Not started

**Dependencies:** Phase 1-3 complete

**Estimated:** 15-20 hours

### Sprints

#### Sprint 4.1: Markdown Viewer Core
**Duration:** 8-10 hours
**Parallel:** No

**Need to plan sprint:** Detailed implementation checklist

**High-Level Tasks:**
- [ ] Add Zed's `markdown` crate as git dependency (wraps pulldown-cmark)
- [ ] Create custom `ArtifactPane` view for document viewing
- [ ] Integrate Zed's markdown renderer with ArtifactPane
- [ ] Implement scrolling
- [ ] Apply theme colors (uses Zed's theme system from Phase 2)
- [ ] Test: Render markdown file correctly
- [ ] Test: All markdown elements display

**Key Point:** Use Zed's markdown crate for rendering, write custom ArtifactPane view

#### Sprint 4.2: Markdown Viewer Integration
**Duration:** 6-8 hours
**Parallel:** No (depends on Sprint 4.1)

**Need to plan sprint:** Detailed implementation checklist

**High-Level Tasks:**
- [ ] Create `src/viewer/pane.rs` - DocumentViewerPane
- [ ] Implement document tabs (bottom tabs)
- [ ] Add file type detection (.md → markdown viewer)
- [ ] Integrate with file browser (select .md → open in viewer)
- [ ] Handle link clicks (open URLs, navigate to other .md files)
- [ ] Add image viewer support (basic, for Phase 4 completion)
- [ ] Test: Multiple markdown docs in tabs
- [ ] Test: File browser → markdown viewer works
- [ ] Test: Link navigation works

### Phase 4 Checkpoint

**Complete when:**
- [ ] Markdown viewer renders correctly
- [ ] Document tabs functional (multiple documents)
- [ ] File browser → document viewer integration works
- [ ] All three panes working together (file browser, terminal, document viewer)
- [ ] Workspace fully functional
- [ ] Link clicks work (URLs open in browser)
- [ ] Theme applied to all components
- [ ] Workspace configuration persists everything

**Phase 4 COMPLETE - Markdown viewer functional**

---

## 7. PHASE 5: Markdown Editor (MVP)

**Goal:** Add markdown editing capability with preview mode.

**Priority:** 5 - Markdown viewer + editing

**Status:** Not started

**Dependencies:** Phase 1-4 complete

**Estimated:** 10-15 hours

### Sprints

#### Sprint 5.1: Markdown Editor Core
**Duration:** 5-7 hours
**Parallel:** No

**Need to plan sprint:** Detailed implementation checklist

**High-Level Tasks:**
- [ ] Create `src/viewer/editor.rs` - MarkdownEditor
- [ ] Implement basic text editing (input, cursor, selection)
- [ ] Add syntax highlighting for markdown
- [ ] Implement save functionality
- [ ] Add undo/redo support
- [ ] Test: Edit markdown file
- [ ] Test: Save changes
- [ ] Test: Syntax highlighting works

#### Sprint 5.2: Editor/Preview Integration
**Duration:** 4-6 hours
**Parallel:** No (depends on Sprint 5.1)

**Need to plan sprint:** Detailed implementation checklist

**High-Level Tasks:**
- [ ] Add mode switcher (preview/edit/split)
- [ ] Implement split view (edit + preview side-by-side)
- [ ] Sync scroll position between edit and preview
- [ ] Add keyboard shortcuts (Cmd+E for edit, Cmd+P for preview)
- [ ] Integrate with document tabs
- [ ] Test: Switch between modes
- [ ] Test: Split view works
- [ ] Test: Scroll sync works

### Phase 5 Checkpoint (MVP COMPLETE)

**Complete when:**
- [ ] Markdown editing mode functional
- [ ] Switch between preview/edit/split modes works
- [ ] Save changes back to file
- [ ] Syntax highlighting for markdown
- [ ] Split view with scroll sync
- [ ] All editor features work correctly

**🎉 MVP COMPLETE - Markdown editing fully functional**
- [ ] Add keyboard shortcut for theme cycling
- [ ] Test: URLs clickable
- [ ] Test: Settings hot-reload works

#### Sprint 4.2: MCP Layer (Placeholder)
**Duration:** 3-5 hours
**Parallel:** Can run parallel to Sprint 4.3

---

## 8. Dependency Graph

```
Phase 1 (Foundation & Workspace)
├─ Sprint 1.1: Project Setup ✅
├─ Sprint 1.2: Settings System ✅
├─ Sprint 1.3: Theme System ✅
├─ Sprint 1.4: GPUI Bootstrap ✅
└─ Sprint 1.5: Workspace Tabs & Config ✅ (PR #12 pending merge)
      ↓ (all complete)

Phase 2 (Zed Terminal)
├─ Sprint 2.1: Copy Zed Terminal
      ↓
├─ Sprint 2.2: Integrate with TerminalG
      ↓
└─ Sprint 2.3: URL Recognition & Clicking
      ↓ (all complete)

Phase 3 (File Browser)
├─ Sprint 3.1: File Browser Core
      ↓
└─ Sprint 3.2: File Browser Integration
      ↓ (all complete)

Phase 4 (Markdown Viewer)
├─ Sprint 4.1: Markdown Viewer Core
      ↓
└─ Sprint 4.2: Markdown Viewer Integration
      ↓

Phase 5 (Markdown Editor - MVP)
├─ Sprint 5.1: Markdown Editor Core
      ↓
└─ Sprint 5.2: Editor/Preview Integration
```

**Parallel Opportunities:**
- None in current plan (all phases sequential)
- Within sprints: Some tasks can be parallelized during implementation

---

## 9. Current Status

### Completed Work

#### Phase 1: Foundation & Workspace ✅ COMPLETE

**Session 1:** 2025-01-23 (~8-10 hours)
- ✅ Project setup
- ✅ Settings system implemented and tested
- ✅ Theme system implemented and tested
- ✅ Build verification passed

**Session 2:** 2025-01-24 (~2 hours)
- ✅ Documentation structure defined
- ✅ GPUI dependency strategy established
- ✅ Architecture documented

**Session 3:** 2025-01-24 (~1 hour)
- ✅ Claude skill for Rust development guidelines created
- ✅ Git-flow branching model initialized
- ✅ Branch protection enabled

**Session 4:** 2026-01-25 (~4 hours)
- ✅ Sprint 1.5: Workspace Tabs & Configuration
- ✅ PR #12 merged

#### Phase 2: Zed Terminal Integration (In Progress)

**Session 5:** 2026-01-25 (~6 hours)

**Sprint 2.1:** Zed Dependencies & Settings/Theme Migration ✅
- Added Zed crates as git dependencies (terminal, settings, theme, ui, util, collections)
- Created `src/settings_adapter.rs` - Zed settings initialization
- Created `src/theme_adapter.rs` - Zed theme initialization
- Updated `src/main.rs` with Zed system init sequence
- WorkspaceView now uses `cx.theme()` for colors
- 60/60 tests passing

**Sprint 2.2:** Terminal Pane Integration ✅
- Created `src/terminal/pane.rs` (~360 lines) - TerminalPane wrapping Zed Terminal
- Created `src/terminal/tab.rs` (~70 lines) - TerminalTab state management
- Integrated TerminalPane into WorkspaceView
- Terminal spawns with PTY, renders content, accepts keyboard input
- Multiple terminal tabs with tab bar UI
- 60/60 tests passing, clippy clean
- PR #14: https://github.com/randlee/terminalg/pull/14

### In Progress

**Phase 2:** Sprint 2.2 complete, PR #14 pending CI/review
- Sprint 2.3 (URL Recognition) not yet started
- Can proceed to Phase 3 in parallel if desired

### Effort Summary

**Used:** ~28-32 hours (Phase 1 complete, Phase 2 66% complete)
**Remaining:** 45-65 hours (Sprint 2.3 + Phases 3, 4, 5)

---

## 9. Sprint Checklist Format

Each sprint document (in `docs/sprints/`) follows this structure:

```markdown
# Phase X Sprint Y: <Name>

**Duration:** X-Y hours
**Dependencies:** <phase/sprint>
**Parallel:** Yes/No
**Status:** Not Started / In Progress / Complete

## Objectives
- Goal 1
- Goal 2

## Prerequisites
- Requirement 1
- Requirement 2

## Tasks
- [ ] Task with acceptance criteria
- [ ] Task with testing requirements

## Testing
- [ ] Test scenario 1
- [ ] Test scenario 2

## Completion Criteria
- [ ] Criterion 1
- [ ] Criterion 2

## References
- Supporting doc 1
- Supporting doc 2
```

---

## 10. Risk Management

### High-Risk Areas

**GPUI API Changes (Phase 1-4)**
- **Risk:** Breaking changes between Zed versions
- **Mitigation:** Tag pinning, deliberate upgrades, abstraction layer
- **Owner:** ARCH-RTERM

**PTY Thread Synchronization (Phase 2)**
- **Risk:** Deadlocks, UI freezes, crashes
- **Mitigation:** Use Zed's proven patterns, extensive testing
- **Owner:** Phase 2 developer

**Cross-Platform PTY (Phase 2)**
- **Risk:** Windows ConPTY significantly different from Unix
- **Mitigation:** Start macOS/Linux, defer Windows to Phase 3+
- **Owner:** Phase 2 developer

### Medium-Risk Areas

**Performance at High Throughput (Phase 2)**
- **Risk:** Sluggish UI, dropped frames
- **Mitigation:** Profile early, implement throttling if needed
- **Owner:** Phase 2-3 developer

**TIFF Image Support (Phase 3)**
- **Risk:** GPUI may not support TIFF natively
- **Mitigation:** Use image crate for decoding, may need custom component
- **Owner:** Phase 3 developer

---

## 11. Quality Gates

### Per-Sprint Gates

**Before marking sprint complete:**
- [ ] All tasks checked off
- [ ] All tests passing
- [ ] Code formatted (`cargo fmt`)
- [ ] Lints passing (`cargo clippy`)
- [ ] Documentation updated
- [ ] Changes committed to git

### Per-Phase Gates

**Before starting next phase:**
- [ ] All sprints in phase complete
- [ ] Phase checkpoint criteria met
- [ ] No compiler warnings (or documented exceptions)
- [ ] Performance baseline acceptable
- [ ] Git tag created (e.g., `phase-1-complete`)

---

## 11. Development Workflow

**Note:** This project uses **git-flow** branching model and agent-driven development. See `docs/GIT-WORKFLOW.md` for complete details.

### Agent-Driven Sprint Workflow

This project uses 5 specialized Rust agents to ensure quality and consistency:

**Available Agents:**
- **rust-architect** - Creates detailed architecture designs and implementation blueprints
- **rust-code-explorer** - Analyzes existing codebase to understand patterns and conventions
- **rust-developer** - Implements code changes following guidelines
- **rust-code-reviewer** - Reviews code for guideline compliance (confidence ≥80%)
- **rust-qa-agent** - Validates tests pass and coverage is adequate

**Agent Registry:** `.claude/agents/registry.yaml`
**Agent Specs:** `.claude/agents/*.md`

### Sprint Start Workflow

**Before writing any code:**

1. **Analyze Context** (if needed)
   - Invoke `rust-code-explorer` to understand relevant existing features
   - Document findings for architecture phase

2. **Design Architecture** (required)
   - Invoke `rust-architect` to create detailed design/implementation blueprint
   - Architect reads guidelines, analyzes codebase patterns, makes decisions
   - Output saved as: `docs/sprints/phase-X-sprint-Y-design.md`
   - Blueprint includes: component design, file changes, data flow, build sequence

3. **Create Feature Branch**
   - `git flow feature start sprint-X-Y-name`
   - Begin implementation following architecture blueprint

### During Sprint (Development)

1. Pick next task from sprint checklist or architecture blueprint
2. Implement using `rust-developer` agent or manual development
3. Follow architecture design and Rust guidelines
4. Write tests for all new code (required)
5. Verify locally: `cargo check`, `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`
6. Commit frequently: `git commit -m "sprint X.Y: description"`

### Sprint Completion (Quality Gates)

**All gates must pass before sprint can be marked complete:**

#### Gate 1: Parallel Review & QA ⚠️ BLOCKING

Run these two agents **IN PARALLEL** - both must pass:

**A. Code Review (`rust-code-reviewer`)**
- Reviews all changes from sprint (git diff)
- Checks compliance with Rust guidelines
- Reports only high-confidence issues (≥80% confidence)
- **BLOCKING:** All issues must be fixed or triaged
  - Fix immediately: Most issues
  - Create follow-up sprint: Serious architectural issues
- Output: `docs/sprints/phase-X-sprint-Y-review.md`

**B. QA Validation (`rust-qa-agent`)**
- Runs complete test suite: `cargo test` (debug + release)
- Generates coverage report using `cargo-llvm-cov`
- Verifies coverage is adequate (guideline: 80%, quality > metrics)
- Checks test quality (no empty tests, no ignored tests)
- **BLOCKING:** 100% tests must pass, test quality acceptable
- Output: `docs/sprints/phase-X-sprint-Y-qa.md`

**Critical Rules:**
- ❌ Cannot disable tests without explicit user permission
- ❌ Cannot modify tests to pass without explicit user permission
- ❌ Cannot proceed if any tests fail (100% must pass)
- ⚠️ Coverage guideline: 80% (quality matters more than hitting exact numbers)
- ✅ Review issues must be fixed or triaged (serious → follow-up sprint)
- ✅ QA must confirm adequate test coverage for code criticality

#### Gate 2: Final Verification

1. All review issues resolved or triaged
2. All tests passing (100%)
3. Coverage adequate (threshold met)
4. Code formatted (`cargo fmt`)
5. No clippy warnings (`cargo clippy -- -D warnings`)

#### Gate 3: Merge & Update

1. Finish feature: `git flow feature finish sprint-X-Y-name`
2. Push develop: `git push origin develop`
3. Update MASTER-PLAN.md status on develop (mark sprint complete)
4. Commit sprint completion: `git commit -m "docs: mark sprint X.Y complete"`
5. Push: `git push origin develop`

### Phase Completion

1. Verify all sprints complete on develop
2. Run phase checkpoint
3. Create release: `git flow release start 0.X.0` (bump version per phase)
4. Update Cargo.toml version to `0.X.0`
5. Update documentation if needed
6. Commit: `git commit -am "chore: bump version to 0.X.0 for Phase X release"`
7. Push release branch and create PR to main
8. After PR merged: tag release on main (e.g., `v0.X.0`, `phase-X-complete`)
9. Merge main back to develop
10. Plan next phase

**Version Strategy:** 0.1.0 → 0.2.0 → 0.3.0 → 0.4.0 → 0.5.0 → 1.0.0 (when validated as usable)

---

## 12. Agent Specifications

This project uses 5 specialized Rust agents for quality-driven development:

### rust-architect (Design Phase)
**Purpose:** Creates detailed architecture designs and implementation blueprints
**When:** Before each sprint, after understanding requirements
**Reads:** Rust guidelines, GPUI guidelines, existing codebase patterns
**Outputs:** Complete implementation blueprint with:
- Pattern analysis from existing code
- Architecture decisions with rationale
- Component design (files, responsibilities, interfaces)
- Implementation map (specific files to create/modify)
- Data flow diagrams
- Build sequence (phased checklist)

**Location:** `.claude/agents/rust-architect.md`

### rust-code-explorer (Analysis Phase)
**Purpose:** Deeply analyzes existing features to understand patterns
**When:** Before architecture phase, when learning existing code
**Reads:** Rust guidelines, GPUI guidelines
**Outputs:** Feature analysis with:
- Entry points and core files
- Code flow tracing
- Architecture layers and patterns
- Dependencies and integrations
- Key insights for new development

**Location:** `.claude/agents/rust-code-explorer.md`

### rust-developer (Implementation Phase)
**Purpose:** Implements code changes following guidelines and architecture
**When:** During sprint, following architecture blueprint
**Reads:** Rust guidelines, GPUI guidelines, architecture design
**Outputs:** Code implementation with:
- Changes summary
- Files modified/created
- Assumptions made
- Follow-up suggestions

**Location:** `.claude/agents/rust-developer.md`

### rust-code-reviewer (Review Phase)
**Purpose:** Reviews code for guideline compliance and quality issues
**When:** Sprint completion (parallel with QA)
**Reads:** Rust guidelines, GPUI guidelines
**Reviews:** Git diff from sprint changes
**Outputs:** Review report with:
- High-confidence issues only (≥80% confidence)
- Severity: Critical vs Important
- Specific file:line references
- Guideline violations
- Concrete fix suggestions

**Location:** `.claude/agents/rust-code-reviewer.md`

### rust-qa-agent (Quality Assurance Phase)
**Purpose:** Validates tests pass and coverage is adequate
**When:** Sprint completion (parallel with code review)
**Does NOT read:** Guidelines (focuses on testing, not design)
**Outputs:** QA report with:
- Test results (100% must pass)
- Coverage analysis (80% guideline)
- Test quality assessment
- Performance metrics
- Sprint gate: PASS or FAIL with blocking issues

**Location:** `.claude/agents/rust-qa-agent.md`

### Agent Registry
All agents registered in: `.claude/agents/registry.yaml`

---

## 13. Timeline Estimates

### Aggressive Schedule (Full-Time)

- **Phase 1:** 1.5-2 days (12-16 hours) - 60% done, 8-12 hours remaining → **v0.1.0**
- **Phase 2:** 2.5-4 days (20-30 hours) → **v0.2.0**
- **Phase 3:** 2-3 days (15-20 hours) → **v0.3.0**
- **Phase 4:** 2-3 days (15-20 hours) → **v0.4.0**
- **Phase 5:** 1.5-2 days (10-15 hours) → **v0.5.0** ← MVP features complete
- **v1.0.0:** When validated as usable (TBD)

**Total (MVP features):** 9.5-14 days full-time (Phases 1-5) → v0.5.0

### Realistic Schedule (Part-Time)

- **Phase 1:** 2-3 sessions - 60% done, 2 sessions remaining → **v0.1.0**
- **Phase 2:** 3-4 sessions → **v0.2.0**
- **Phase 3:** 2-3 sessions → **v0.3.0**
- **Phase 4:** 2-3 sessions → **v0.4.0**
- **Phase 5:** 2-3 sessions → **v0.5.0** ← MVP features complete
- **v1.0.0:** When validated as usable (TBD)

**Total (MVP features):** 11-16 sessions (~3-4 weeks) → v0.5.0

---

## 13. Success Metrics

### Phase 1 Success (Foundation & Workspace)
- App-wide settings + workspace config systems functional
- Theme system functional
- GPUI window opens with workspace tabs
- Three-pane placeholder layout renders
- Pane visibility controls work
- Build time < 10 minutes (clean build with GPUI)

### Phase 2 Success (Zed Terminal)
- Zed terminal fully integrated (all features, all platforms)
- Commands execute and display output
- URL recognition and clicking works
- Terminal tabs functional (multiple terminals)
- Copy/paste, mouse, search all working
- Performance: 60 FPS rendering

### Phase 3 Success (File Browser)
- File/folder browser displays tree correctly
- Expand/collapse functional
- File selection opens in document viewer
- Integrated with workspace config
- Keyboard navigation works

### Phase 4 Success (Markdown Viewer)
- Markdown viewer renders correctly
- Document tabs functional (multiple documents)
- File browser → document viewer integration works
- All three panes working together
- Theme applied to all components
- Markdown viewer fully functional

### Phase 5 Success (Markdown Editor - MVP)
- Markdown editing mode functional
- Switch between preview/edit/split modes
- Save changes back to file
- Syntax highlighting works
- Split view with scroll sync

### MVP Success (Phases 1-5)
- Three co-equal panes (file browser, terminal, document viewer)
- Workspace tabs switch full context
- Zed terminal fully functional + URL clicking
- File browser navigates projects
- Markdown renders inline
- No browser tabs needed
- Workspace configuration persists

---

## 14. Post-MVP Enhancements

**Future enhancements (not in current plan):**
- MCP integration layer
- Image viewer (currently basic support only)
- Advanced image features (zoom, pan)
- Syntax highlighting in markdown code blocks (beyond basic)
- Custom theme creation/editing UI
- Plugin system for custom viewers
- Advanced layouts (more complex panes, arbitrary splits)
- Command history and search
- Session recording/replay
- Synaptic Canvas agent integration
- Git integration UI
- Remote session support
- Collaboration features

---

## 15. References

### Primary Documents

- **Requirements:** `docs/REQUIREMENTS.md`
- **Architecture:** `docs/ARCHITECTURE.md`
- **Zed Reuse Strategy:** `docs/architecture/zed-reuse-strategy.md` - Source of truth for Zed crate dependencies
- **This Document:** `docs/MASTER-PLAN.md`
- **Git Workflow:** `docs/GIT-WORKFLOW.md`

### Supporting Documents

- **GPUI Integration:** `docs/architecture/gpui-integration.md`
- **Settings System:** `docs/architecture/settings-system.md` (Phase 1 only - migrates to Zed in Phase 2)
- **Development Guide:** `docs/DEVELOPMENT-GUIDE.md`
- **Quick Start:** `docs/QUICK-START.md`

### Sprint Documents

- **Phase 1 Sprint 4:** `docs/sprints/phase-1-sprint-4-gpui-bootstrap.md`
- **Phase 1 Sprint 5:** `docs/sprints/phase-1-sprint-5-design.md`
- **Future sprints:** Create as needed

---

**Document Status:** ✅ Approved
**Next Update:** After each sprint completion or phase transition
