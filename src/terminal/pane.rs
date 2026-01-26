//! Terminal pane component
//!
//! Wraps Zed's `Terminal` to provide a renderable terminal pane for `TerminalG`.
//! This module handles terminal spawning, rendering, and input handling.

use collections::HashMap;
use gpui::{
    div, prelude::*, px, App, Context, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, MouseButton, MouseDownEvent, MouseMoveEvent, MouseUpEvent, Render, Styled,
    Subscription, Task, Window,
};
use settings::Settings;
use std::path::PathBuf;
use terminal::{
    terminal_settings::TerminalSettings, Event as TerminalEvent, MaybeNavigationTarget,
    TerminalBuilder,
};
use theme::ActiveTheme;
use util::shell::Shell;

use crate::terminal::tab::TerminalTab;

/// Default regex patterns for detecting file paths in terminal output.
/// Note: These can be noisy. Consider gating behind a setting if false positives are an issue.
const DEFAULT_PATH_REGEXES: &[&str] = &[
    // File paths with optional line:col
    r"[a-zA-Z0-9._\-~/]+/[a-zA-Z0-9._\-~/]+(?::\d+)?(?::\d+)?",
    // Common source file extensions
    r"[\w\-/\.]+\.(?:rs|js|ts|py|go|java|c|cpp|h|md|txt)",
];

/// Events emitted by the terminal pane
#[derive(Clone, Debug)]
pub enum TerminalPaneEvent {
    /// Terminal title changed
    TitleChanged,
    /// Terminal closed
    Close,
}

/// Terminal pane wrapping Zed's Terminal
pub struct TerminalPane {
    /// Terminal tabs (each tab is a separate terminal session)
    tabs: Vec<TerminalTab>,
    /// Active tab index
    active_tab: usize,
    /// Focus handle for keyboard input
    focus_handle: FocusHandle,
    /// Subscriptions to terminal events
    subscriptions: Vec<Subscription>,
}

impl TerminalPane {
    /// Create a new terminal pane with an initial terminal
    pub fn new(working_directory: Option<PathBuf>, cx: &mut Context<Self>) -> Self {
        let focus_handle = cx.focus_handle();
        let pane = Self {
            tabs: Vec::new(),
            active_tab: 0,
            focus_handle,
            subscriptions: Vec::new(),
        };

        // Spawn initial terminal
        pane.spawn_terminal(working_directory, cx);

        pane
    }

    /// Spawn a new terminal tab
    #[allow(clippy::unused_self)] // Method semantically operates on this pane
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.spawn requires &mut Context
    pub fn spawn_terminal(&self, working_directory: Option<PathBuf>, cx: &mut Context<Self>) {
        let settings = TerminalSettings::get_global(cx);
        let shell = Shell::System;
        let env: HashMap<String, String> = std::env::vars().collect();
        let cursor_shape = settings.cursor_shape;
        let alternate_scroll = settings.alternate_scroll;
        let max_scroll_history = settings.max_scroll_history_lines;

        // Get window ID for PTY
        let window_id = cx.entity_id().as_u64();

        // Clone working_directory for the async closure
        let working_dir = working_directory.clone();

        // Prepare path hyperlink regex patterns
        let path_hyperlink_regexes: Vec<String> = DEFAULT_PATH_REGEXES
            .iter()
            .map(|s| (*s).to_string())
            .collect();

        // Spawn terminal asynchronously
        let terminal_task: Task<anyhow::Result<TerminalBuilder>> = TerminalBuilder::new(
            working_directory,
            None, // No task state
            shell,
            env,
            cursor_shape,
            alternate_scroll,
            max_scroll_history,
            path_hyperlink_regexes, // Use configured patterns
            500,        // path_hyperlink_timeout_ms
            false,      // is_remote_terminal
            window_id,
            None, // completion_tx
            cx,
            Vec::new(), // activation_script
        );

        // Handle terminal creation
        cx.spawn(async move |this, cx| {
            match terminal_task.await {
                Ok(builder) => {
                    let _ = this.update(cx, |pane, cx| {
                        // Create the terminal entity and subscribe to events
                        let terminal = cx.new(|cx| builder.subscribe(cx));

                        let tab = TerminalTab::new(terminal.clone(), working_dir, cx);

                        // Subscribe to terminal events
                        let subscription =
                            cx.subscribe(&terminal, |pane: &mut Self, _terminal, event, cx| {
                                pane.handle_terminal_event(event, cx);
                            });

                        pane.tabs.push(tab);
                        pane.active_tab = pane.tabs.len() - 1;
                        pane.subscriptions.push(subscription);
                        cx.notify();
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to spawn terminal: {}", e);
                }
            }
        })
        .detach();
    }

    /// Handle terminal events
    fn handle_terminal_event(&mut self, event: &TerminalEvent, cx: &mut Context<Self>) {
        match event {
            TerminalEvent::TitleChanged | TerminalEvent::BreadcrumbsChanged => {
                cx.emit(TerminalPaneEvent::TitleChanged);
                cx.notify();
            }
            TerminalEvent::CloseTerminal => {
                // Remove the active terminal tab
                if !self.tabs.is_empty() {
                    self.tabs.remove(self.active_tab);
                    if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                        self.active_tab = self.tabs.len() - 1;
                    }
                    if self.tabs.is_empty() {
                        cx.emit(TerminalPaneEvent::Close);
                    }
                    cx.notify();
                }
            }
            TerminalEvent::Wakeup => {
                cx.notify();
            }
            TerminalEvent::Bell => {
                // Could play a sound or flash the window
                tracing::debug!("Terminal bell");
            }
            // Handle URL open events
            TerminalEvent::Open(target) => {
                self.handle_open_target(target, cx);
            }
            // Handle hover state changes
            TerminalEvent::NewNavigationTarget(target) => {
                self.handle_navigation_target(target, cx);
            }
            _ => {}
        }
    }

    /// Handle opening a URL or path
    #[allow(clippy::needless_pass_by_ref_mut)] // Called from event handler context
    #[allow(clippy::unused_self)] // Method signature required by event handler pattern
    fn handle_open_target(&mut self, target: &MaybeNavigationTarget, _cx: &mut Context<Self>) {
        match target {
            MaybeNavigationTarget::Url(url) => {
                tracing::info!("Opening URL: {}", url);
                if let Err(e) = open::that(url) {
                    tracing::error!("Failed to open URL {}: {}", url, e);
                }
            }
            MaybeNavigationTarget::PathLike(path_target) => {
                // Future: implement path navigation
                tracing::info!(
                    "Path navigation requested: {:?}",
                    path_target.maybe_path
                );
            }
        }
    }

    /// Handle navigation target hover state changes
    #[allow(clippy::needless_pass_by_ref_mut)] // Called from event handler context
    #[allow(clippy::unused_self)] // Method signature required by event handler pattern
    #[allow(clippy::ref_option)] // API signature from Zed terminal crate
    fn handle_navigation_target(
        &mut self,
        target: &Option<MaybeNavigationTarget>,
        cx: &mut Context<Self>,
    ) {
        if let Some(MaybeNavigationTarget::Url(url)) = target {
            tracing::debug!("Hovering URL: {}", url);
        }
        // Ensure hover state clears immediately when target becomes None
        cx.notify();
    }

    /// Get the active terminal tab
    #[allow(dead_code)]
    pub fn active_tab(&self) -> Option<&TerminalTab> {
        self.tabs.get(self.active_tab)
    }

    /// Get the active terminal tab mutably
    pub fn active_tab_mut(&mut self) -> Option<&mut TerminalTab> {
        self.tabs.get_mut(self.active_tab)
    }

    /// Switch to a specific tab
    pub fn switch_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        if index < self.tabs.len() {
            self.active_tab = index;
            cx.notify();
        }
    }

    /// Close the active tab
    #[allow(dead_code)]
    pub fn close_active_tab(&mut self, cx: &mut Context<Self>) {
        if !self.tabs.is_empty() {
            self.tabs.remove(self.active_tab);
            if self.active_tab >= self.tabs.len() && !self.tabs.is_empty() {
                self.active_tab = self.tabs.len() - 1;
            }
            if self.tabs.is_empty() {
                cx.emit(TerminalPaneEvent::Close);
            }
            cx.notify();
        }
    }

    /// Handle key input
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab) = self.active_tab_mut() {
            // Convert keystroke to terminal input
            if let Some(input) = keystroke_to_input(&event.keystroke) {
                tab.terminal.update(cx, |terminal, _| {
                    terminal.input(input);
                });
                cx.notify();
            }
        }
    }

    /// Handle mouse move events - forwards to Zed terminal for hyperlink detection
    fn handle_mouse_move(
        &mut self,
        event: &MouseMoveEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab) = self.active_tab_mut() {
            tab.terminal.update(cx, |terminal, cx| {
                terminal.mouse_move(event, cx);
            });
            cx.notify();
        }
    }

    /// Handle mouse down events - forwards to Zed terminal
    fn handle_mouse_down(
        &mut self,
        event: &MouseDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab) = self.active_tab_mut() {
            tab.terminal.update(cx, |terminal, cx| {
                terminal.mouse_down(event, cx);
            });
            cx.notify();
        }
    }

    /// Handle mouse up events - forwards to Zed terminal for URL opening
    fn handle_mouse_up(
        &mut self,
        event: &MouseUpEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        if let Some(tab) = self.active_tab_mut() {
            tab.terminal.update(cx, |terminal, cx| {
                terminal.mouse_up(event, cx);
            });
            cx.notify();
        }
    }

    /// Render terminal tabs bar
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.listener requires &mut Context
    fn render_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .h(px(32.0))
            .w_full()
            .flex()
            .items_center()
            .bg(theme.colors().tab_bar_background)
            .border_t_1()
            .border_color(theme.colors().border)
            .children(self.tabs.iter().enumerate().map(|(idx, tab)| {
                let is_active = idx == self.active_tab;
                let title = tab.title();

                div()
                    .id(("terminal-tab", idx))
                    .px_3()
                    .py_1()
                    .mx_1()
                    .rounded_sm()
                    .cursor_pointer()
                    .when(is_active, |d| d.bg(theme.colors().tab_active_background))
                    .when(!is_active, |d| d.bg(theme.colors().tab_inactive_background))
                    .text_color(theme.colors().text)
                    .text_sm()
                    .child(title)
                    .on_click(cx.listener(move |this, _, _window, cx| {
                        this.switch_tab(idx, cx);
                    }))
            }))
            .child(
                // New tab button
                div()
                    .id("new-terminal-tab")
                    .px_2()
                    .py_1()
                    .cursor_pointer()
                    .text_color(theme.colors().text_muted)
                    .hover(|s| s.text_color(theme.colors().text))
                    .child("+")
                    .on_click(cx.listener(|this, _, _window, cx| {
                        this.spawn_terminal(None, cx);
                    })),
            )
    }

    /// Render the terminal content area
    #[allow(clippy::needless_pass_by_ref_mut)] // GPUI read requires context
    #[allow(clippy::option_if_let_else)] // if-let is more readable here
    fn render_terminal_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        if let Some(tab) = self.tabs.get(self.active_tab) {
            let terminal = tab.terminal.read(cx);
            let content = terminal.last_content();

            // Simple text rendering of terminal content
            let mut lines: Vec<String> = Vec::new();
            let mut current_line = String::new();
            let mut current_row = 0i32;

            for cell in &content.cells {
                if cell.point.line.0 != current_row {
                    if !current_line.is_empty() || current_row < cell.point.line.0 {
                        lines.push(std::mem::take(&mut current_line));
                    }
                    // Fill empty lines
                    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                    while (lines.len() as i32) < cell.point.line.0 {
                        lines.push(String::new());
                    }
                    current_row = cell.point.line.0;
                }
                current_line.push(cell.c);
            }
            if !current_line.is_empty() {
                lines.push(current_line);
            }

            div()
                .flex_1()
                .w_full()
                .bg(theme.colors().terminal_background)
                .text_color(theme.colors().terminal_foreground)
                .font_family("Menlo")
                .text_sm()
                .p_2()
                .overflow_hidden()
                .children(lines.into_iter().map(|line| {
                    div().child(if line.is_empty() {
                        " ".to_string()
                    } else {
                        line
                    })
                }))
        } else {
            div()
                .flex_1()
                .w_full()
                .flex()
                .items_center()
                .justify_center()
                .bg(theme.colors().terminal_background)
                .text_color(theme.colors().text_muted)
                .child("Starting terminal...")
        }
    }
}

impl EventEmitter<TerminalPaneEvent> for TerminalPane {}

impl Focusable for TerminalPane {
    fn focus_handle(&self, _cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for TerminalPane {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .track_focus(&self.focus_handle)
            .flex()
            .flex_col()
            .size_full()
            .on_key_down(cx.listener(Self::handle_key_down))
            .on_mouse_move(cx.listener(Self::handle_mouse_move))
            .on_mouse_down(MouseButton::Left, cx.listener(Self::handle_mouse_down))
            .on_mouse_up(MouseButton::Left, cx.listener(Self::handle_mouse_up))
            .child(self.render_terminal_content(cx))
            .child(self.render_tabs(cx))
    }
}

/// Convert a GPUI keystroke to terminal input bytes
fn keystroke_to_input(keystroke: &gpui::Keystroke) -> Option<Vec<u8>> {
    use terminal::mappings::keys::to_esc_str;

    // Try to convert using Zed's key mapping
    if let Some(esc_str) = to_esc_str(
        keystroke,
        &terminal::alacritty_terminal::term::TermMode::empty(),
        false,
    ) {
        return Some(esc_str.as_bytes().to_vec());
    }

    // Fallback for printable characters
    if keystroke.key.len() == 1 && !keystroke.modifiers.control && !keystroke.modifiers.alt {
        return Some(keystroke.key.as_bytes().to_vec());
    }

    None
}
