# TerminalG Master Plan

**Version:** 1.0
**Last Updated:** 2025-01-24
**Current Phase:** Phase 1 (70% complete)

---

## 1. Overview

Development organized into 4 phases, each containing sprints that can be executed sequentially or in parallel where dependencies allow.

**Total Estimated Effort:** 90-130 hours
**Timeline:** 10-16 days full-time, or 2-3 weeks part-time

---

## 2. Phase Breakdown

**Aligned with priorities:**
1. App framework w/ empty windows
2. Fully working Zed terminal
3. Add file/folder browser
4. Add markdown viewer (preview)
5. Markdown viewer + editing (post-MVP)

| Phase | Name | Status | Estimated Hours | Sprints |
|-------|------|--------|-----------------|---------|
| 1 | Foundation & Workspace | 70% | 20-25 | 4 |
| 2 | Zed Terminal Integration | Not Started | 20-30 | 3 |
| 3 | File/Folder Browser | Not Started | 15-20 | 2 |
| 4 | Markdown Viewer (MVP) | Not Started | 15-20 | 2 |
| 5 | Markdown Editor (Post-MVP) | Not Started | 10-15 | 2 |

---

## 3. PHASE 1: Foundation & Workspace

**Goal:** Establish project structure, settings system, theme system, GPUI app with workspace tabs, and workspace configuration.

**Priority:** 1 - App framework w/ empty windows

**Status:** 70% complete (Settings ✓, Theme ✓, GPUI app + workspace pending)

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

#### Sprint 1.4: GPUI Bootstrap ⏳ IN PROGRESS
**Duration:** 3-5 hours
**Status:** Ready to start

**Detailed Checklist:** `docs/sprints/phase-1-sprint-4-gpui-bootstrap.md`

**High-Level Tasks:**
- [ ] Add GPUI dependency (tag = v0.220.3) to Cargo.toml
- [ ] Add smol and futures dependencies
- [ ] Update main.rs with GPUI app initialization
- [ ] Create empty window with title "TerminalG"
- [ ] Apply theme background color to window
- [ ] Test: Window opens
- [ ] Test: Window interactive (move, resize, close)
- [ ] Test: No crashes

**Supporting Doc:** `docs/architecture/gpui-integration.md`

#### Sprint 1.5: Workspace Tabs & Configuration
**Duration:** 4-6 hours
**Status:** Not started

**Need to plan sprint:** Detailed implementation checklist

**High-Level Tasks:**
- [ ] Create WorkspaceConfig struct (save/load workspace state)
- [ ] Implement workspace tab bar UI (top tabs)
- [ ] Implement workspace switching
- [ ] Create placeholder pane layout (3 empty panes)
- [ ] Add pane visibility controls (hide/show buttons)
- [ ] Auto-save workspace config on changes
- [ ] Default workspace: file browser + terminal visible
- [ ] Test: Workspace tabs switch
- [ ] Test: Workspace config persists

### Phase 1 Checkpoint

**Complete when:**
- [x] App-wide settings system working (load/save/reload)
- [x] Settings file created in correct location
- [x] Theme system working (dark/light themes)
- [x] `cargo check` passes
- [x] `cargo build` succeeds
- [ ] GPUI window opens with theme colors
- [ ] Workspace tabs functional (UI, switching)
- [ ] Workspace configuration system working (save/load)
- [ ] Three placeholder panes rendering
- [ ] Pane visibility controls work (hide/show)
- [ ] No crashes or errors

**Ready for:** Phase 2 (Zed Terminal Integration)

---

## 4. PHASE 2: Zed Terminal Integration

**Goal:** Copy Zed's terminal wholesale, integrate with TerminalG, add URL recognition.

**Priority:** 2 - Fully working Zed terminal (all features, all platforms)

**Status:** Not started

**Dependencies:** Phase 1 complete

**Estimated:** 20-30 hours

### Sprints

#### Sprint 2.1: Copy Zed Terminal
**Duration:** 6-8 hours
**Parallel:** No (foundation)

**Need to plan sprint:** Detailed implementation checklist

**High-Level Tasks:**
- [ ] Copy entire `crates/terminal/` from Zed (v0.220.3)
- [ ] Add to TerminalG as `src/terminal/`
- [ ] Add necessary dependencies (alacritty_terminal, libc, windows, etc.)
- [ ] Verify all platforms compile (macOS, Linux, Windows)
- [ ] Test: Module compiles without errors

**Key Point:** Minimal modifications - preserve Zed's structure for PR-ability

#### Sprint 2.2: Integrate with TerminalG
**Duration:** 8-12 hours
**Parallel:** No (depends on Sprint 2.1)

**Need to plan sprint:** Detailed implementation checklist

**High-Level Tasks:**
- [ ] Replace Zed settings references with TerminalG SettingsStore
- [ ] Replace Zed theme references with TerminalG Theme
- [ ] Create TerminalPane wrapper for WorkspaceView
- [ ] Integrate with workspace configuration
- [ ] Add terminal tab management (multiple terminals)
- [ ] Connect to pane visibility system
- [ ] Test: Terminal opens in workspace
- [ ] Test: Terminal functional (type, execute, see output)
- [ ] Test: All Zed features work (copy/paste, mouse, search, etc.)

**Key Point:** Keep Zed terminal logic intact, only adapt integration points

#### Sprint 2.3: URL Recognition & Clicking
**Duration:** 6-10 hours
**Parallel:** No (depends on Sprint 2.2)

**Need to plan sprint:** Detailed implementation checklist

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
- [ ] All Zed terminal features working (PTY, rendering, scrollback, copy/paste, mouse, search)
- [ ] Works on all platforms (macOS, Linux, Windows)
- [ ] Terminal tabs functional (multiple terminals per workspace)
- [ ] Terminal integrated with workspace config (persists state)
- [ ] URL recognition and clicking works
- [ ] Theme applied correctly
- [ ] Settings applied correctly
- [ ] No crashes or memory leaks

**Ready for:** Phase 3 (File Browser)

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

## 6. PHASE 4: Markdown Viewer (MVP)

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
- [ ] Add pulldown-cmark dependency
- [ ] Create `src/viewer/markdown.rs` - MarkdownViewer
- [ ] Parse markdown to AST
- [ ] Render to GPUI elements (headings, paragraphs, lists, code blocks)
- [ ] Implement scrolling
- [ ] Apply theme colors (text, background, code blocks)
- [ ] Test: Render markdown file correctly
- [ ] Test: All markdown elements display

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

### Phase 4 Checkpoint (MVP COMPLETE)

**Complete when:**
- [ ] Markdown viewer renders correctly
- [ ] Document tabs functional (multiple documents)
- [ ] File browser → document viewer integration works
- [ ] All three panes working together (file browser, terminal, document viewer)
- [ ] Workspace fully functional
- [ ] Link clicks work (URLs open in browser)
- [ ] Theme applied to all components
- [ ] Workspace configuration persists everything

**🎉 MVP COMPLETE - All core features functional**

---

## 7. PHASE 5: Markdown Editor (Post-MVP)

**Goal:** Add markdown editing capability with preview mode.

**Priority:** 5 - Markdown viewer + editing

**Status:** Not started (Post-MVP)

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

### Phase 5 Checkpoint

**Complete when:**
- [ ] Markdown editing mode functional
- [ ] Switch between preview/edit/split modes works
- [ ] Save changes back to file
- [ ] Syntax highlighting for markdown
- [ ] Split view with scroll sync
- [ ] All editor features work correctly

**🎉 PHASE 5 COMPLETE - Markdown editing fully functional**
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
├─ Sprint 1.4: GPUI Bootstrap ⏳
└─ Sprint 1.5: Workspace Tabs & Config
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

Phase 4 (Markdown Viewer - MVP)
├─ Sprint 4.1: Markdown Viewer Core
      ↓
└─ Sprint 4.2: Markdown Viewer Integration
      ↓ (MVP COMPLETE) 🎉

Phase 5 (Markdown Editor - Post-MVP)
├─ Sprint 5.1: Markdown Editor Core
      ↓
└─ Sprint 5.2: Editor/Preview Integration
```

**Parallel Opportunities:**
- None in current plan (all phases sequential)
- Within sprints: Some tasks can be parallelized during implementation

---

## 9. Current Status

### Completed Work (Phase 1)

**Session 1:** 2025-01-23 (~8-10 hours)
- ✅ Project setup
- ✅ Settings system implemented and tested
- ✅ Theme system implemented and tested
- ✅ Build verification passed

**Session 2:** 2025-01-24 (~2 hours)
- ✅ Documentation structure defined
- ✅ GPUI dependency strategy established
- ✅ Architecture documented
- ⏳ Ready for GPUI bootstrap implementation

**Session 3:** 2025-01-24 (~1 hour)
- ✅ Claude skill for Rust development guidelines created
- ✅ Microsoft's Pragmatic Rust Guidelines integrated (88KB, 2,437 lines)
- ✅ Skill configured for automatic activation on Rust code
- ✅ Git-flow branching model initialized (main/develop)
- ✅ Develop branch created and pushed to remote
- ✅ Git workflow documentation added (docs/GIT-WORKFLOW.md)
- ✅ Main branch protection enabled (PR required, no direct commits)
- ✅ Branch protection verified and documented
- ✅ All changes committed to develop branch

### In Progress

**Sprint 1.4:** GPUI Bootstrap
- Status: Ready to start
- Blockers: None
- Next step: Follow `docs/sprints/phase-1-sprint-4-gpui-bootstrap.md`

### Effort Summary

**Used:** 10-12 hours (Phase 1 foundation)
**Remaining:** 80-120 hours (Phases 1.4, 2, 3, 4)

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

**Note:** This project uses **git-flow** branching model. See `docs/GIT-WORKFLOW.md` for complete details.

### Daily Workflow

1. Pick next sprint or task from MASTER-PLAN.md
2. Read sprint document (if exists) in `docs/sprints/`
3. Create feature branch: `git flow feature start sprint-X-Y-name`
4. Implement and test locally
5. Verify: `cargo check`, `cargo build`, `cargo test`, `cargo clippy`, `cargo fmt`
6. Commit: `git commit -m "sprint X.Y: description"`
7. Mark task complete in MASTER-PLAN.md
8. Move to next task

### Sprint Completion

1. Verify all tasks checked
2. Run quality gates
3. Finish feature: `git flow feature finish sprint-X-Y-name`
4. Push develop: `git push origin develop`
5. Update MASTER-PLAN.md status on develop
6. Commit status update to develop

### Phase Completion

1. Verify all sprints complete on develop
2. Run phase checkpoint
3. Create release: `git flow release start phase-X-complete`
4. Update documentation if needed
5. Finish release: `git flow release finish phase-X-complete`
6. Push all: `git push origin main develop --tags`
7. Plan next phase

---

## 12. Timeline Estimates

### Aggressive Schedule (Full-Time)

- **Phase 1:** 1.5-2 days (12-16 hours) - 70% done, 6-10 hours remaining
- **Phase 2:** 2.5-4 days (20-30 hours)
- **Phase 3:** 2-3 days (15-20 hours)
- **Phase 4:** 2-3 days (15-20 hours) ← **MVP COMPLETE**
- **Phase 5:** 1.5-2 days (10-15 hours) - Post-MVP

**Total (MVP):** 8-12 days full-time (Phases 1-4)
**Total (with editing):** 9.5-14 days full-time (Phases 1-5)

### Realistic Schedule (Part-Time)

- **Phase 1:** 2-3 sessions - 70% done, 1-2 sessions remaining
- **Phase 2:** 3-4 sessions
- **Phase 3:** 2-3 sessions
- **Phase 4:** 2-3 sessions ← **MVP COMPLETE**
- **Phase 5:** 2-3 sessions - Post-MVP

**Total (MVP):** 9-13 sessions (~2-3 weeks)
**Total (with editing):** 11-16 sessions (~3-4 weeks)

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

### Phase 4 Success (Markdown Viewer - MVP)
- Markdown viewer renders correctly
- Document tabs functional (multiple documents)
- File browser → document viewer integration works
- All three panes working together
- Theme applied to all components
- **🎉 MVP COMPLETE**

### Phase 5 Success (Markdown Editor - Post-MVP)
- Markdown editing mode functional
- Switch between preview/edit/split modes
- Save changes back to file
- Syntax highlighting works
- Split view with scroll sync

### MVP Success (Phases 1-4)
- Three co-equal panes (file browser, terminal, document viewer)
- Workspace tabs switch full context
- Zed terminal fully functional + URL clicking
- File browser navigates projects
- Markdown renders inline
- No browser tabs needed
- Workspace configuration persists

---

## 14. Post-MVP Enhancements

**Included in Phase 5 (Markdown Editor):**
- ✓ Markdown editing with syntax highlighting
- ✓ Preview/Edit/Split modes

**Future enhancements (not in current plan):**
- MCP integration layer
- Test visualization (structured test results)
- Agent execution visualization
- Live settings reload / hot-reload
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
- **This Document:** `docs/MASTER-PLAN.md`
- **Git Workflow:** `docs/GIT-WORKFLOW.md`

### Supporting Documents

- **GPUI Integration:** `docs/architecture/gpui-integration.md`
- **Settings System:** `docs/architecture/settings-system.md`
- **Development Guide:** `docs/DEVELOPMENT-GUIDE.md`
- **Quick Start:** `docs/QUICK-START.md`

### Sprint Documents

- **Phase 1 Sprint 4:** `docs/sprints/phase-1-sprint-4-gpui-bootstrap.md`
- **Future sprints:** Create as needed

---

**Document Status:** ✅ Approved
**Next Update:** After each sprint completion or phase transition
