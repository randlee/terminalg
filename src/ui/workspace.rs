//! Workspace view component
//!
//! Implements workspace tab bar and three-pane layout with configurable visibility.

use crate::terminal::{TerminalPane, TerminalPaneEvent};
use crate::ui::workspace_config::WorkspaceConfigStore;
use gpui::{
    div, prelude::*, px, ElementId, Entity, IntoElement, Render, Styled, Subscription, Task, Window,
};
use std::time::Duration;
use theme::ActiveTheme;

/// Main workspace view with tab bar and three-pane layout
pub struct WorkspaceView {
    /// Workspace configuration store
    config_store: WorkspaceConfigStore,

    /// Terminal pane entity
    terminal_pane: Entity<TerminalPane>,

    /// Subscription to terminal pane events
    _terminal_subscription: Subscription,

    /// Debounce timer for auto-save (task handle)
    save_task: Option<Task<()>>,
}

/// Pane type identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaneType {
    FileBrowser,
    Terminal,
    DocumentViewer,
}

impl WorkspaceView {
    /// Create new workspace view, loading config from disk
    pub fn new(cx: &mut Context<Self>) -> Self {
        let config_store = WorkspaceConfigStore::new().unwrap_or_else(|e| {
            tracing::error!("Failed to load workspace config: {}, using defaults", e);
            // Fallback: create a temporary config store with defaults
            // This shouldn't happen in practice but prevents crashes
            WorkspaceConfigStore::new_with_path(
                std::env::temp_dir().join("terminalg-workspace-fallback.json"),
            )
            .expect("Failed to create fallback workspace config")
        });

        tracing::info!(
            "Workspace config loaded from {:?}",
            config_store.config_path()
        );

        // Set CWD to workspace root to align terminal/file tree
        let workspace_root = config_store.workspace_root();
        if let Err(e) = std::env::set_current_dir(workspace_root) {
            tracing::error!(
                "Failed to set current directory to workspace root {}: {}",
                workspace_root.display(),
                e
            );
        } else {
            tracing::info!(
                "Current directory set to workspace root: {}",
                workspace_root.display()
            );
        }

        // Create terminal pane with workspace root as working directory
        let working_directory = Some(config_store.workspace_root().to_path_buf());
        let workspace_id = config_store.active_workspace().id.clone();
        let terminal_pane = cx.new(|cx| TerminalPane::new(workspace_id, working_directory, cx));

        // Subscribe to terminal pane events
        let terminal_subscription = cx.subscribe(&terminal_pane, |_this, _pane, event, cx| {
            match event {
                TerminalPaneEvent::TitleChanged => {
                    tracing::debug!("Terminal title changed");
                    cx.notify();
                }
                TerminalPaneEvent::Close => {
                    tracing::info!("Terminal pane closed");
                    // Could handle workspace-level terminal close logic here
                }
            }
        });

        Self {
            config_store,
            terminal_pane,
            _terminal_subscription: terminal_subscription,
            save_task: None,
        }
    }

    /// Switch to workspace at index
    fn switch_workspace(&mut self, index: usize, cx: &mut Context<Self>) {
        tracing::info!("Switching to workspace {index}");
        self.config_store.switch_workspace(index);
        let workspace_root = self.config_store.workspace_root();
        if let Err(e) = std::env::set_current_dir(workspace_root) {
            tracing::error!(
                "Failed to set current directory to workspace root {}: {}",
                workspace_root.display(),
                e
            );
        }
        let workspace_id = self.config_store.active_workspace().id.clone();
        let working_directory = Some(workspace_root.to_path_buf());
        self.terminal_pane.update(cx, |terminal_pane, cx| {
            terminal_pane.set_active_workspace(workspace_id, working_directory, cx);
        });
        cx.notify();
        self.schedule_save(cx);
    }

    /// Toggle pane visibility
    fn toggle_pane(&mut self, pane_type: PaneType, cx: &mut Context<Self>) {
        let ws = self.config_store.active_workspace_mut();
        match pane_type {
            PaneType::FileBrowser => {
                ws.file_browser_visible = !ws.file_browser_visible;
                tracing::debug!("File browser visibility: {}", ws.file_browser_visible);
            }
            PaneType::Terminal => {
                ws.terminal_visible = !ws.terminal_visible;
                tracing::debug!("Terminal visibility: {}", ws.terminal_visible);
            }
            PaneType::DocumentViewer => {
                ws.document_viewer_visible = !ws.document_viewer_visible;
                tracing::debug!("Document viewer visibility: {}", ws.document_viewer_visible);
            }
        }
        cx.notify();
        self.schedule_save(cx);
    }

    /// Schedule debounced save (200ms)
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.spawn requires &mut Context
    fn schedule_save(&mut self, cx: &mut Context<Self>) {
        // Cancel any pending save
        self.save_task.take();

        // Clone config for async save
        let config_store = self.config_store.clone();

        // Schedule save after 200ms debounce
        self.save_task = Some(cx.spawn(async move |_, _| {
            smol::Timer::after(Duration::from_millis(200)).await;

            // Save to disk
            if let Err(e) = config_store.save() {
                tracing::error!("Failed to save workspace config: {e}");
            } else {
                tracing::debug!("Workspace config saved");
            }
        }));
    }

    /// Render workspace tab bar
    fn render_tab_bar(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();
        let workspaces = &self.config_store.config().workspaces;
        let active_idx = self.config_store.config().active_workspace_index;

        div()
            .h(px(40.0))
            .w_full()
            .flex()
            .items_center()
            .bg(theme.colors().tab_bar_background)
            .border_b_1()
            .border_color(theme.colors().border)
            .children(
                workspaces
                    .iter()
                    .enumerate()
                    .map(|(idx, ws)| self.render_tab(idx, &ws.name, idx == active_idx, cx)),
            )
    }

    /// Render a single tab
    #[allow(clippy::unused_self)] // Required for method chaining in render
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.listener requires &mut Context
    fn render_tab(
        &self,
        index: usize,
        name: &str,
        is_active: bool,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id(ElementId::NamedInteger(
                "workspace-tab".into(),
                index as u64,
            ))
            .px_4()
            .py_2()
            .mx_1()
            .rounded_md()
            .cursor_pointer()
            .when(is_active, |d| d.bg(theme.colors().tab_active_background))
            .when(!is_active, |d| d.bg(theme.colors().tab_inactive_background))
            .text_color(theme.colors().text)
            .child(name.to_string())
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.switch_workspace(index, cx);
            }))
    }

    /// Render three-pane content area
    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let ws = self.config_store.active_workspace();

        div()
            .flex()
            .flex_1()
            .w_full()
            .when(ws.file_browser_visible, |d| {
                d.child(self.render_pane(PaneType::FileBrowser, cx))
            })
            .when(ws.terminal_visible, |d| {
                d.child(self.render_pane(PaneType::Terminal, cx))
            })
            .when(ws.document_viewer_visible, |d| {
                d.child(self.render_pane(PaneType::DocumentViewer, cx))
            })
    }

    /// Render a single pane (terminal uses real component, others are placeholders)
    fn render_pane(&self, pane_type: PaneType, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // Terminal pane renders the actual TerminalPane component
        if pane_type == PaneType::Terminal {
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .m_2()
                .bg(theme.colors().panel_background)
                .rounded_md()
                .overflow_hidden()
                .child(
                    div()
                        .flex()
                        .justify_between()
                        .items_center()
                        .px_3()
                        .py_2()
                        .border_b_1()
                        .border_color(theme.colors().border)
                        .child(
                            div()
                                .text_lg()
                                .font_weight(gpui::FontWeight::SEMIBOLD)
                                .text_color(theme.colors().text)
                                .child("Terminal"),
                        )
                        .child(self.render_hide_button(pane_type, cx)),
                )
                .child(self.terminal_pane.clone());
        }

        // Other panes render placeholders
        let label = match pane_type {
            PaneType::FileBrowser => "File Browser",
            PaneType::Terminal => unreachable!(),
            PaneType::DocumentViewer => "Document Viewer",
        };

        div()
            .flex_1()
            .flex()
            .flex_col()
            .m_2()
            .p_3()
            .bg(theme.colors().panel_background)
            .rounded_md()
            .text_color(theme.colors().text)
            .child(
                div()
                    .flex()
                    .justify_between()
                    .items_center()
                    .mb_2()
                    .child(
                        div()
                            .text_lg()
                            .font_weight(gpui::FontWeight::SEMIBOLD)
                            .child(label),
                    )
                    .child(self.render_hide_button(pane_type, cx)),
            )
            .child(
                div()
                    .flex_1()
                    .flex()
                    .items_center()
                    .justify_center()
                    .child(format!("{label} Placeholder")),
            )
    }

    /// Render hide button for a pane
    #[allow(clippy::unused_self)] // Required for method chaining in render
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.listener requires &mut Context
    fn render_hide_button(&self, pane_type: PaneType, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .id(ElementId::NamedInteger(
                "hide-pane".into(),
                pane_type as u64,
            ))
            .px_3()
            .py_1()
            .cursor_pointer()
            .bg(theme.colors().element_background)
            .hover(|s| s.bg(theme.colors().element_hover))
            .rounded_sm()
            .text_sm()
            .child("Hide")
            .on_click(cx.listener(move |this, _, _window, cx| {
                this.toggle_pane(pane_type, cx);
            }))
    }
}

impl Render for WorkspaceView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(theme.colors().background)
            .child(self.render_tab_bar(cx))
            .child(self.render_content(cx))
    }
}
