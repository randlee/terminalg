# TerminalG Requirements

**Version:** 1.0
**Last Updated:** 2025-01-24

---

## 1. Vision

Build a custom GPU-accelerated terminal UI application for interactive development workflows with rich artifact visualization, structured test execution, and live output rendering.

---

## 2. Core Problems

Existing tools force workflows into constrained patterns:

- **Zed** - Editor-centric, not terminal-first
- **Warp** - Closed-source, not extensible
- **Terminal + Browser** - Fragmented workflow (new tabs for every test run)
- **HTML Test Reports** - Requires new browser tabs, breaks flow

---

## 3. Solution Overview

**TerminalG**: A purpose-built terminal application with:

- **Three co-equal panes:** File/folder browser, Terminal (Zed-based), Document viewer
- **Workspace-level tabs** at top (switch entire context/layout)
- **Multiple terminals/documents** per workspace (tabs at bottom of panes)
- **Hide/show any pane** with single button click
- **Integrated viewers** - Markdown, images (no browser needed)
- **URL recognition and clicking** in terminal output
- **Workspace configuration** - Separate config file per workspace
- **Real-time test execution** visualization

---

## 4. Functional Requirements

### 4.1 MVP Features (Phase 1-3)

**Terminal Capabilities**

*Use Zed's terminal wholesale - ALL features included:*
- [ ] Full terminal emulation (PTY-based, all platforms)
- [ ] ANSI color support (full 256-color + truecolor)
- [ ] Scrollback buffer (configurable size)
- [ ] Copy/paste support
- [ ] Mouse support (click, scroll, selection)
- [ ] Terminal resizing
- [ ] Shell integration
- [ ] Working directory tracking
- [ ] Title tracking
- [ ] Search in terminal output
- [ ] Clear screen
- [ ] Ligature support
- [ ] **NEW: URL recognition in output**
- [ ] **NEW: Clickable URLs**

*Note: Zed terminal copied wholesale with minimal modifications. URL handling added on top, designed for potential PR back to Zed.*

**Layout & UI**
- [ ] Workspace tabs at top (switch entire context)
- [ ] Three co-equal panes: File browser, Terminal, Document viewer
- [ ] Each pane independently hideable/showable (single button)
- [ ] Terminal pane: Multiple terminals with tabs at bottom
- [ ] Document viewer pane: Multiple documents with tabs at bottom
- [ ] Resizable pane dividers (drag to adjust)
- [ ] Default visibility (first workspace): File browser + Terminal (rooted at same folder)
- [ ] Subsequent visibility: Restore from workspace config
- [ ] Status bar

**Viewers**
- [ ] Markdown viewer - **PRIORITY** (preview/render mode)
- [ ] Image viewer (PNG, JPEG, GIF, TIFF)
- [ ] SVG viewer (required; defer if unsupported by chosen image stack)
- [ ] Automatic viewer selection by file type
- [ ] Zoom/pan for images

*Note: Markdown editing to follow shortly after MVP (Phase 5).*

**Configuration**

*App-wide settings:*
- [ ] JSON-based settings file (platform-specific config dir)
- [ ] Settings paths: macOS `~/Library/Application Support/terminalg/settings.json`, Linux `~/.config/terminalg/settings.json`, Windows `%APPDATA%\\terminalg\\settings.json`
- [ ] Versioned settings schema with migrations on load
- [ ] Theme system (dark/light built-in)
- [ ] Terminal settings (font, padding, scrollback)
- [ ] UI settings (default layout ratios, theme selection)
- [ ] Auto-save on change

*Workspace configuration:*
- [ ] Separate config file per workspace
- [ ] Auto-save workspace state (open terminals, documents, pane sizes, visibility)
- [ ] Near-term: Store in platform config dir (macOS `~/Library/Application Support/terminalg/workspaces/<name>.json`, Linux `~/.config/terminalg/workspaces/<name>.json`, Windows `%APPDATA%\\terminalg\\workspaces\\<name>.json`)
- [ ] Workspace-local: Support `.terminalg/<workspace>.json` in project repos
- [ ] First-time workspace: Default to file browser + terminal (same root folder)
- [ ] App settings track all available workspaces and their config paths

### 4.2 Enhanced Features (Phase 4)

**Live Reload**
- [ ] Settings hot-reload (file watcher)
- [ ] Theme switching without restart
- [ ] Keyboard shortcut to cycle themes

**Test Visualization**
- [ ] Parse test output (pytest, etc.)
- [ ] Display test results in structured format
- [ ] Show timing and status
- [ ] Expandable test details

**Agent Visualization**
- [ ] Display hook execution sequence
- [ ] Show prompts/responses timeline
- [ ] Subagent call hierarchy
- [ ] Timing information per step

**MCP Integration**
- [ ] MCP layer for structured data
- [ ] Terminal output → MCP parsing
- [ ] Viewer receives parsed data

### 4.3 Future Enhancements (Post-MVP)

- Syntax highlighting in markdown code blocks
- Custom theme creation UI
- Plugin system for custom viewers
- Advanced layouts (split-screen, complex panes)
- Command history and search
- Session recording/replay
- Synaptic Canvas agent integration

---

## 5. Non-Functional Requirements

### 5.1 Performance

**Target Metrics:**
- Window launch: < 2 seconds
- Render frame time: < 16ms (60 FPS)
- Memory usage (idle): < 100 MB
- Binary size (release): < 50 MB
- Handle 100k lines of terminal output smoothly

### 5.2 Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| macOS | ✓ Supported | 10.15+ |
| Linux | ✓ Supported | glibc 2.31+, X11/Wayland |
| Windows | ✓ Supported | 10+, ConPTY support |

**Phase 1 validation on all platforms** (bootstrap app window). Full terminal support via Zed integration arrives in Phase 2.

**Primary development/testing:** macOS
**CI/Testing:** All platforms

### 5.3 Reliability

- No crashes or memory leaks
- Graceful handling of corrupted files
- Proper error messages for user issues
- Clean shutdown (no zombie processes)

### 5.4 Maintainability

- Clear code structure with separation of concerns
- Comprehensive documentation
- Minimal dependencies (use Rust ecosystem)
- Follow Zed's patterns where applicable

---

## 6. Technical Constraints

### 6.1 Technology Stack

**Required:**
- Rust (1.70+)
- GPUI (Zed's UI framework, git dependency)
- alacritty_terminal (terminal emulation)
- smol (async runtime)

**Rationale:** Leverage proven components from Zed ecosystem

### 6.2 Dependencies

- GPUI not published to crates.io (use git dependency)
- Pin to stable Zed releases (tag-based versioning)
- Minimize external dependencies
- Use platform-specific crates where needed (PTY management)

### 6.3 Build Requirements

- Rust toolchain installed
- Platform-specific build tools:
  - macOS: Xcode Command Line Tools
  - Linux: GCC/Clang + pkg-config
  - Windows: Visual Studio Build Tools

---

## 7. Success Criteria

### 7.1 Phase 1 Complete When (Foundation):
- Settings system working (app-wide + workspace config)
- Theme system working (dark/light themes)
- Basic GPUI app launches with window
- Workspace tabs functional (UI only, empty panes)
- Theme colors applied to UI
- No crashes or errors

### 7.2 Phase 2 Complete When (Zed Terminal):
- Zed terminal integrated wholesale (all features)
- Terminal renders text output correctly (all platforms)
- User can type commands and see execution
- Shell prompt appears
- Scrollback works
- Copy/paste, mouse support functional
- Search in terminal output works
- Multiple terminals with tabs at bottom
- Terminal settings applied correctly
- URL recognition and clicking works

### 7.3 Phase 3 Complete When (File Browser):
- File/folder browser pane functional
- Tree view with expand/collapse
- File selection
- Navigation works
- Integrated with workspace config
- Pane visibility controls work (hide/show)
- Layout persists across restarts

### 7.4 Phase 4 Complete When (Markdown Viewer - MVP):
- Markdown viewer renders correctly
- Multiple documents with tabs at bottom
- File browser → document viewer integration
- Automatic file type detection
- All three panes working together
- Workspace fully functional

### 7.5 Phase 5 Complete When (Markdown Editor - Post-MVP):
- Markdown editing mode functional
- Switch between preview and edit modes
- Save changes back to file
- Syntax support for markdown

### 7.6 MVP Success (Phases 1-4):
- Three co-equal panes (file browser, terminal, document viewer)
- Workspace tabs switch full context
- Zed terminal fully functional (all features + URL clicking)
- File browser navigates projects
- Markdown renders inline
- No browser tabs needed for artifacts
- Workspace configuration persists

---

## 8. Out of Scope (Explicitly Not Doing)

- Full-featured text editor (basic markdown editing only)
- Git integration UI (use terminal git commands)
- Package manager integration
- Cloud sync/collaboration
- Mobile support
- Web version
- Plugin marketplace (plugin system is future consideration)
- IDE features (debugging, refactoring, code completion)

---

## 9. User Stories

### US-1: Developer Running Tests
**As a** developer running pytest
**I want** test results to appear in a structured viewer
**So that** I don't have to open HTML reports in browser tabs

### US-2: Developer Viewing Artifacts
**As a** developer reviewing generated images
**I want** images to appear inline in the terminal app
**So that** I can see results without switching to Finder/Preview

### US-3: Developer Switching Contexts
**As a** developer working on multiple projects
**I want** workspace tabs to switch my entire environment
**So that** I maintain separate contexts per project

### US-4: Developer Customizing Appearance
**As a** developer with theme preferences
**I want** to switch between dark/light themes
**So that** I can work comfortably in different lighting conditions

### US-5: Developer Clicking URLs
**As a** developer reading terminal output with URLs
**I want** to click URLs directly in the terminal
**So that** I can open documentation without copy-paste

### US-6: Developer Managing Screen Space
**As a** developer with limited screen space
**I want** to hide/show panes with a single click
**So that** I can focus on the task at hand (terminal, files, or documents)

### US-7: Developer Working with Multiple Terminals
**As a** developer running multiple services
**I want** multiple terminal tabs in the same workspace
**So that** I can monitor logs from different processes simultaneously

---

## 10. Assumptions

- Users are familiar with terminal usage
- Users have Rust development environment
- Users work primarily on macOS/Linux (initially)
- Users prefer keyboard-driven workflows
- Users value performance and responsiveness
- Users are comfortable with JSON configuration files

---

## 11. Dependencies on External Projects

### 11.1 Zed/GPUI
- **Dependency:** GPUI UI framework
- **Version:** Tag-pinned to Zed stable releases
- **Risk:** API changes between versions
- **Mitigation:** Pin to tested release, upgrade deliberately

### 11.2 alacritty_terminal
- **Dependency:** Terminal emulation library
- **Version:** crates.io stable release
- **Risk:** Breaking changes
- **Mitigation:** Use stable API surface, test thoroughly

---

## 12. References

- **Architecture:** `docs/ARCHITECTURE.md`
- **Master Plan:** `docs/MASTER-PLAN.md`
- **GPUI Integration:** `docs/architecture/gpui-integration.md`
- **Settings System:** `docs/architecture/settings-system.md`

---

**Document Status:** ✅ Approved
**Next Review:** After Phase 1 complete
