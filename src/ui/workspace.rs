//! Workspace view component
//!
//! Implements workspace tab bar and three-pane layout with configurable visibility.

use crate::theme::Theme;
use crate::ui::workspace_config::WorkspaceConfigStore;
use gpui::{div, prelude::*, px, rgb, ElementId, IntoElement, Render, Styled, Task, Window};
use std::time::Duration;

/// Main workspace view with tab bar and three-pane layout
pub struct WorkspaceView {
    /// Workspace configuration store
    config_store: WorkspaceConfigStore,

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
    pub fn new(_cx: &mut Context<Self>) -> Self {
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

        Self {
            config_store,
            save_task: None,
        }
    }

    /// Switch to workspace at index
    fn switch_workspace(&mut self, index: usize, cx: &mut Context<Self>) {
        tracing::info!("Switching to workspace {index}");
        self.config_store.switch_workspace(index);
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
        let theme = cx.global::<Theme>();
        let workspaces = &self.config_store.config().workspaces;
        let active_idx = self.config_store.config().active_workspace_index;

        // Tab bar background color (slightly lighter than main bg)
        let tab_bar_bg = rgb(u32::from(theme.background.r.saturating_add(10)) << 16
            | u32::from(theme.background.g.saturating_add(10)) << 8
            | u32::from(theme.background.b.saturating_add(10)));

        // Border color
        let border_color = rgb(u32::from(theme.text_muted.r) << 16
            | u32::from(theme.text_muted.g) << 8
            | u32::from(theme.text_muted.b));

        div()
            .h(px(40.0))
            .w_full()
            .flex()
            .items_center()
            .bg(tab_bar_bg)
            .border_b_1()
            .border_color(border_color)
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
        let theme = cx.global::<Theme>();

        // Active tab colors (accent color)
        let active_bg = rgb(u32::from(theme.accent.r) << 16
            | u32::from(theme.accent.g) << 8
            | u32::from(theme.accent.b));

        // Inactive tab colors (slightly lighter than bg)
        let inactive_bg = rgb(u32::from(theme.background.r.saturating_add(20)) << 16
            | u32::from(theme.background.g.saturating_add(20)) << 8
            | u32::from(theme.background.b.saturating_add(20)));

        // Text color
        let text_color = rgb(u32::from(theme.foreground.r) << 16
            | u32::from(theme.foreground.g) << 8
            | u32::from(theme.foreground.b));

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
            .when(is_active, |d| d.bg(active_bg))
            .when(!is_active, |d| d.bg(inactive_bg))
            .text_color(text_color)
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

    /// Render a single placeholder pane
    #[allow(clippy::unreadable_literal)] // Color hex codes are more readable without separators
    fn render_pane(&self, pane_type: PaneType, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.global::<Theme>();

        let (label, pane_bg) = match pane_type {
            PaneType::FileBrowser => ("File Browser", rgb(0x2D4A6E)),
            PaneType::Terminal => ("Terminal", rgb(0x3A3A3A)),
            PaneType::DocumentViewer => ("Document Viewer", rgb(0x4A2D6E)),
        };

        // Text color
        let text_color = rgb(u32::from(theme.foreground.r) << 16
            | u32::from(theme.foreground.g) << 8
            | u32::from(theme.foreground.b));

        div()
            .flex_1()
            .flex()
            .flex_col()
            .m_2()
            .p_3()
            .bg(pane_bg)
            .rounded_md()
            .text_color(text_color)
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
    #[allow(clippy::unreadable_literal)] // Color hex codes are more readable without separators
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.listener requires &mut Context
    fn render_hide_button(&self, pane_type: PaneType, cx: &mut Context<Self>) -> impl IntoElement {
        let button_bg = rgb(0x555555);
        let button_hover_bg = rgb(0x666666);

        div()
            .id(ElementId::NamedInteger(
                "hide-pane".into(),
                pane_type as u64,
            ))
            .px_3()
            .py_1()
            .cursor_pointer()
            .bg(button_bg)
            .hover(|s| s.bg(button_hover_bg))
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
        let theme = cx.global::<Theme>();

        // Background color
        let bg_color = rgb(u32::from(theme.background.r) << 16
            | u32::from(theme.background.g) << 8
            | u32::from(theme.background.b));

        div()
            .flex()
            .flex_col()
            .size_full()
            .bg(bg_color)
            .child(self.render_tab_bar(cx))
            .child(self.render_content(cx))
    }
}
