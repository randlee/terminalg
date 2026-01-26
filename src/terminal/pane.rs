//! Terminal pane component
//!
//! Wraps Zed's `Terminal` to provide a renderable terminal pane for `TerminalG`.
//! This module handles terminal spawning, rendering, and input handling.

use collections::HashMap;
use gpui::{
    div, prelude::*, px, App, Context, Entity, EventEmitter, FocusHandle, Focusable, IntoElement,
    KeyDownEvent, Render, Styled, Task, Window,
};
use settings::Settings;
use std::path::PathBuf;
use terminal::{
    terminal_settings::TerminalSettings, Event as TerminalEvent, Terminal, TerminalBuilder,
};
use theme::ActiveTheme;
use util::shell::Shell;

use crate::terminal::tab::TerminalTab;

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
    /// Terminal tabs per workspace
    tabs_by_workspace: HashMap<String, Vec<TerminalTab>>,
    /// Active workspace ID
    active_workspace_id: String,
    /// Active tab index per workspace
    active_tab_by_workspace: HashMap<String, usize>,
    /// Working directory per workspace
    working_directory_by_workspace: HashMap<String, Option<PathBuf>>,
    /// Focus handle for keyboard input
    focus_handle: FocusHandle,
}

impl TerminalPane {
    /// Create a new terminal pane with an initial terminal
    pub fn new(
        workspace_id: String,
        working_directory: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) -> Self {
        let focus_handle = cx.focus_handle();
        let mut pane = Self {
            tabs_by_workspace: HashMap::default(),
            active_workspace_id: workspace_id,
            active_tab_by_workspace: HashMap::default(),
            working_directory_by_workspace: HashMap::default(),
            focus_handle,
        };

        // Spawn initial terminal
        let active_workspace_id = pane.active_workspace_id.clone();
        pane.working_directory_by_workspace
            .insert(active_workspace_id.clone(), working_directory.clone());
        pane.spawn_terminal(active_workspace_id, working_directory, cx);

        pane
    }

    /// Spawn a new terminal tab
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.spawn requires &mut Context
    pub fn spawn_terminal(
        &mut self,
        workspace_id: String,
        working_directory: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) {
        let settings = TerminalSettings::get_global(cx);
        let shell = Shell::System;
        let env: HashMap<String, String> = std::env::vars().collect();
        let cursor_shape = settings.cursor_shape;
        let alternate_scroll = settings.alternate_scroll;
        let max_scroll_history = settings.max_scroll_history_lines;

        // Get window ID for PTY
        let window_id = cx.entity_id().as_u64();

        // Clone working_directory and workspace_id for the async closure
        let workspace_key = workspace_id.clone();
        self.working_directory_by_workspace
            .insert(workspace_id, working_directory.clone());
        let working_dir = working_directory.clone();

        // Spawn terminal asynchronously
        let terminal_task: Task<anyhow::Result<TerminalBuilder>> = TerminalBuilder::new(
            working_directory,
            None, // No task state
            shell,
            env,
            cursor_shape,
            alternate_scroll,
            max_scroll_history,
            Vec::new(), // path_hyperlink_regexes
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

                        // Subscribe to terminal events
                        let subscription =
                            cx.subscribe(&terminal, |pane: &mut Self, terminal, event, cx| {
                                pane.handle_terminal_event(&terminal, event, cx);
                            });

                        let tab = TerminalTab::new(terminal.clone(), subscription, working_dir, cx);

                        let tabs = pane
                            .tabs_by_workspace
                            .entry(workspace_key.clone())
                            .or_insert_with(Vec::new);
                        tabs.push(tab);

                        let active_index = tabs.len().saturating_sub(1);
                        pane.active_tab_by_workspace
                            .insert(workspace_key.clone(), active_index);
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
    fn handle_terminal_event(
        &mut self,
        terminal: &Entity<Terminal>,
        event: &TerminalEvent,
        cx: &mut Context<Self>,
    ) {
        match event {
            TerminalEvent::TitleChanged | TerminalEvent::BreadcrumbsChanged => {
                cx.emit(TerminalPaneEvent::TitleChanged);
                cx.notify();
            }
            TerminalEvent::CloseTerminal => {
                self.close_terminal_by_id(terminal.entity_id(), cx);
            }
            TerminalEvent::Wakeup => {
                cx.notify();
            }
            TerminalEvent::Bell => {
                // Could play a sound or flash the window
                tracing::debug!("Terminal bell");
            }
            _ => {}
        }
    }

    /// Switch to a specific workspace (lazy-loads terminals on first switch)
    pub fn set_active_workspace(
        &mut self,
        workspace_id: String,
        working_directory: Option<PathBuf>,
        cx: &mut Context<Self>,
    ) {
        self.active_workspace_id.clone_from(&workspace_id);
        self.working_directory_by_workspace
            .insert(workspace_id.clone(), working_directory.clone());
        if !self.tabs_by_workspace.contains_key(&workspace_id) {
            self.tabs_by_workspace
                .insert(workspace_id.clone(), Vec::new());
        }
        if !self.active_tab_by_workspace.contains_key(&workspace_id) {
            self.active_tab_by_workspace.insert(workspace_id.clone(), 0);
        }
        let is_empty = self
            .tabs_by_workspace
            .get(&workspace_id)
            .is_some_and(Vec::is_empty);
        if is_empty {
            self.spawn_terminal(workspace_id, working_directory, cx);
        } else {
            cx.notify();
        }
    }

    /// Get the active terminal tab
    #[allow(dead_code)]
    pub fn active_tab(&self) -> Option<&TerminalTab> {
        self.tabs_by_workspace
            .get(&self.active_workspace_id)
            .and_then(|tabs| {
                let index = self
                    .active_tab_by_workspace
                    .get(&self.active_workspace_id)?;
                tabs.get(*index)
            })
    }

    /// Get the active terminal tab mutably
    pub fn active_tab_mut(&mut self) -> Option<&mut TerminalTab> {
        let active_index = *self
            .active_tab_by_workspace
            .get(&self.active_workspace_id)?;
        self.tabs_by_workspace
            .get_mut(&self.active_workspace_id)
            .and_then(|tabs| tabs.get_mut(active_index))
    }

    /// Switch to a specific tab
    pub fn switch_tab(&mut self, index: usize, cx: &mut Context<Self>) {
        let Some(tabs) = self.tabs_by_workspace.get(&self.active_workspace_id) else {
            return;
        };
        if index < tabs.len() {
            self.active_tab_by_workspace
                .insert(self.active_workspace_id.clone(), index);
            cx.notify();
        }
    }

    /// Close the active tab
    #[allow(dead_code)]
    pub fn close_active_tab(&mut self, cx: &mut Context<Self>) {
        let terminal_id = self.active_tab().map(|tab| tab.terminal.entity_id());
        if let Some(terminal_id) = terminal_id {
            self.close_terminal_by_id(terminal_id, cx);
        }
    }

    /// Handle key input
    fn handle_key_down(
        &mut self,
        event: &KeyDownEvent,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) {
        let option_as_meta = TerminalSettings::get_global(cx).option_as_meta;
        if let Some(tab) = self.active_tab_mut() {
            tab.terminal.update(cx, |terminal, cx| {
                let handled = terminal.try_keystroke(&event.keystroke, option_as_meta);
                if handled {
                    cx.stop_propagation();
                }
            });
            cx.notify();
        }
    }

    /// Render terminal tabs bar
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.listener requires &mut Context
    fn render_tabs(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let tabs: &[TerminalTab] = match self.tabs_by_workspace.get(&self.active_workspace_id) {
            Some(tabs) => tabs.as_slice(),
            None => &[],
        };
        let active_tab_index = *self
            .active_tab_by_workspace
            .get(&self.active_workspace_id)
            .unwrap_or(&0);

        div()
            .h(px(32.0))
            .w_full()
            .flex()
            .items_center()
            .bg(theme.colors().tab_bar_background)
            .border_t_1()
            .border_color(theme.colors().border)
            .children(tabs.iter().enumerate().map(|(idx, tab)| {
                let is_active = idx == active_tab_index;
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
                        let workspace_id = this.active_workspace_id.clone();
                        let working_directory = this
                            .working_directory_by_workspace
                            .get(&workspace_id)
                            .cloned()
                            .unwrap_or(None);
                        this.spawn_terminal(workspace_id, working_directory, cx);
                    })),
            )
    }

    /// Render the terminal content area
    #[allow(clippy::needless_pass_by_ref_mut)] // GPUI read requires context
    #[allow(clippy::option_if_let_else)] // if-let is more readable here
    fn render_terminal_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        let tabs: &[TerminalTab] = match self.tabs_by_workspace.get(&self.active_workspace_id) {
            Some(tabs) => tabs.as_slice(),
            None => &[],
        };
        let active_tab_index = *self
            .active_tab_by_workspace
            .get(&self.active_workspace_id)
            .unwrap_or(&0);

        if let Some(tab) = tabs.get(active_tab_index) {
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
            .child(self.render_terminal_content(cx))
            .child(self.render_tabs(cx))
    }
}

impl TerminalPane {
    fn close_terminal_by_id(&mut self, terminal_id: gpui::EntityId, cx: &mut Context<Self>) {
        let mut target: Option<(String, usize)> = None;
        for (workspace_id, tabs) in &self.tabs_by_workspace {
            if let Some(index) = tabs
                .iter()
                .position(|tab| tab.matches_terminal(terminal_id))
            {
                target = Some((workspace_id.clone(), index));
                break;
            }
        }

        if let Some((workspace_id, index)) = target {
            if let Some(tabs) = self.tabs_by_workspace.get_mut(&workspace_id) {
                tabs.remove(index);
                let active_index = self
                    .active_tab_by_workspace
                    .entry(workspace_id.clone())
                    .or_insert(0);
                if *active_index >= tabs.len() && !tabs.is_empty() {
                    *active_index = tabs.len() - 1;
                }
                if tabs.is_empty() && workspace_id == self.active_workspace_id {
                    cx.emit(TerminalPaneEvent::Close);
                }
                cx.notify();
            }
        }
    }
}
