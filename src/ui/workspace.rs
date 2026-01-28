//! Workspace view component
//!
//! Implements workspace tab bar and three-pane layout with configurable visibility.

use crate::file_browser::{FileBrowserPane, FileBrowserPaneEvent};
use crate::terminal::{TerminalPane, TerminalPaneEvent};
use crate::ui::workspace_config::WorkspaceConfigStore;
use gpui::{
    div, prelude::*, px, relative, App, Bounds, ClickEvent, Element, ElementId, Entity,
    GlobalElementId, IntoElement, LayoutId, MouseButton, MouseDownEvent, MouseMoveEvent, Pixels,
    Render, Size, Style, Styled, Subscription, Task, WeakEntity, Window,
};
use std::time::Duration;
use theme::ActiveTheme;

/// Minimum width for a pane to prevent narrow/unusable layouts
const MIN_PANE_WIDTH: Pixels = px(150.0);
const DIVIDER_WIDTH: Pixels = px(6.0);

/// State tracking for drag-to-resize operations on pane dividers
#[derive(Clone, Copy)]
struct ResizeDragState {
    /// Index of the divider being dragged (0 = left divider, 1 = right divider)
    divider_index: usize,
    /// Initial X position of the mouse when drag started
    start_x: Pixels,
    /// Pane ratios at the start of the drag operation
    start_ratios: [f32; 3],
    /// Total available width for all panes at drag start
    total_width: Pixels,
}

/// Element that captures the bounds of its layout and reports width to the workspace view.
struct ContentBoundsReporter {
    target: WeakEntity<WorkspaceView>,
    id: ElementId,
}

impl ContentBoundsReporter {
    fn new(target: WeakEntity<WorkspaceView>, id: impl Into<ElementId>) -> Self {
        Self {
            target,
            id: id.into(),
        }
    }
}

impl IntoElement for ContentBoundsReporter {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for ContentBoundsReporter {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        Some(self.id.clone())
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            size: Size {
                width: gpui::relative(1.).into(),
                height: gpui::relative(1.).into(),
            },
            ..Default::default()
        };
        let layout_id = window.request_layout(style, None, cx);
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        if let Some(target) = self.target.upgrade() {
            let width = bounds.size.width;
            target.update(cx, |view, _cx| {
                view.content_width = Some(width);
            });
        }
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _layout: &mut Self::PrepaintState,
        _window: &mut Window,
        _cx: &mut App,
    ) {
    }
}

/// Main workspace view with tab bar and three-pane layout
pub struct WorkspaceView {
    /// Workspace configuration store
    config_store: WorkspaceConfigStore,

    /// File browser pane entity
    file_browser_pane: Entity<FileBrowserPane>,

    /// Subscription to file browser pane events
    _file_browser_subscription: Subscription,

    /// Terminal pane entity
    terminal_pane: Entity<TerminalPane>,

    /// Subscription to terminal pane events
    _terminal_subscription: Subscription,

    /// Debounce timer for auto-save (task handle)
    save_task: Option<Task<()>>,

    /// Active resize drag state (Some when user is dragging a divider)
    resize_drag_state: Option<ResizeDragState>,

    /// Last known content width for pane layout calculations
    content_width: Option<Pixels>,
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
        let terminal_pane =
            cx.new(|cx| TerminalPane::new(workspace_id.clone(), working_directory, cx));

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

        // Create file browser pane
        let file_browser_pane = cx.new(|cx| FileBrowserPane::new(workspace_id, cx));

        // Subscribe to file browser pane events
        let file_browser_subscription =
            cx.subscribe(&file_browser_pane, |this, _pane, event, cx| match event {
                FileBrowserPaneEvent::OpenFile(path) => {
                    tracing::info!("Open file requested: {:?}", path);
                    // TODO: Wire up to document viewer once implemented
                }
                FileBrowserPaneEvent::OpenInTerminal(path) => {
                    tracing::info!("Open in terminal requested: {:?}", path);
                    this.handle_open_in_terminal(path.clone(), cx);
                }
                FileBrowserPaneEvent::SelectionChanged(path) => {
                    tracing::debug!("File browser selection changed: {:?}", path);
                }
            });

        Self {
            config_store,
            file_browser_pane,
            _file_browser_subscription: file_browser_subscription,
            terminal_pane,
            _terminal_subscription: terminal_subscription,
            save_task: None,
            resize_drag_state: None,
            content_width: None,
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

        // Update file browser pane
        self.file_browser_pane.update(cx, |file_browser_pane, cx| {
            file_browser_pane.set_active_workspace(workspace_id.clone(), cx);
        });

        // Update terminal pane
        self.terminal_pane.update(cx, |terminal_pane, cx| {
            terminal_pane.set_active_workspace(workspace_id, working_directory, cx);
        });

        cx.notify();
        self.schedule_save(cx);
    }

    /// Handle `OpenInTerminal` event from file browser
    fn handle_open_in_terminal(&mut self, path: std::path::PathBuf, cx: &mut Context<Self>) {
        // Ensure the terminal pane is visible
        let ws = self.config_store.active_workspace_mut();
        if !ws.terminal_visible {
            ws.terminal_visible = true;
            self.schedule_save(cx);
        }

        // Spawn a new terminal tab with the directory as working directory
        let workspace_id = self.config_store.active_workspace().id.clone();
        let working_directory = if path.is_dir() {
            Some(path)
        } else {
            path.parent().map(std::path::Path::to_path_buf)
        };

        self.terminal_pane.update(cx, |terminal_pane, cx| {
            terminal_pane.spawn_terminal(workspace_id, working_directory, cx);
        });

        cx.notify();
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

    /// Render vertical divider between panes
    #[allow(clippy::needless_pass_by_ref_mut)] // cx.listener requires &mut Context
    fn render_divider(
        &self,
        divider_index: usize,
        total_width: Pixels,
        cx: &mut Context<Self>,
    ) -> impl IntoElement {
        let is_dragging = self
            .resize_drag_state
            .as_ref()
            .is_some_and(|s| s.divider_index == divider_index);

        let theme = cx.theme();

        div()
            .id(ElementId::NamedInteger(
                "pane-divider".into(),
                divider_index as u64,
            ))
            .w(px(6.0))
            .h_full()
            .cursor_col_resize()
            .bg(if is_dragging {
                theme.colors().border_focused
            } else {
                theme.colors().border
            })
            .on_mouse_down(
                MouseButton::Left,
                cx.listener(move |this, event: &MouseDownEvent, _window, cx| {
                    this.start_resize_drag(divider_index, event.position.x, total_width, cx);
                }),
            )
            .on_click(cx.listener(move |this, event: &ClickEvent, _window, cx| {
                // Double-click to reset ratios
                if let ClickEvent::Mouse(mouse_event) = event {
                    if mouse_event.up.click_count == 2 {
                        this.reset_pane_ratios(cx);
                    }
                }
            }))
    }

    /// Start drag-to-resize operation on a divider
    fn start_resize_drag(
        &mut self,
        divider_index: usize,
        start_x: Pixels,
        total_width: Pixels,
        cx: &mut Context<Self>,
    ) {
        let ws = self.config_store.active_workspace();
        let visible = [
            ws.file_browser_visible,
            ws.terminal_visible,
            ws.document_viewer_visible,
        ];
        let available_width = if total_width > px(0.0) {
            total_width
        } else {
            self.available_panes_width(visible)
        };
        if available_width <= px(0.0) {
            return;
        }
        self.resize_drag_state = Some(ResizeDragState {
            divider_index,
            start_x,
            start_ratios: ws.pane_ratios,
            total_width: available_width,
        });
        cx.notify();
    }

    /// Handle mouse movement during drag-to-resize
    fn handle_resize_drag(&mut self, position_x: Pixels, cx: &mut Context<Self>) {
        if let Some(ref drag_state) = self.resize_drag_state {
            let ws = self.config_store.active_workspace();
            let visible = [
                ws.file_browser_visible,
                ws.terminal_visible,
                ws.document_viewer_visible,
            ];
            let available_width = self.available_panes_width(visible);
            let total_width = if available_width > px(0.0) {
                available_width
            } else {
                drag_state.total_width
            };
            let delta = position_x - drag_state.start_x;
            let new_ratios = calculate_new_ratios(drag_state, delta, visible, total_width);

            let ws = self.config_store.active_workspace_mut();
            ws.pane_ratios = new_ratios;
            cx.notify();
        }
    }

    /// End drag-to-resize operation and persist changes
    fn end_resize_drag(&mut self, cx: &mut Context<Self>) {
        if self.resize_drag_state.take().is_some() {
            self.schedule_save(cx);
            cx.notify();
        }
    }

    /// Calculate new pane ratios based on drag delta, enforcing minimum widths
    #[allow(clippy::unused_self)] // Method for consistency with other instance methods
    /// Reset pane ratios to default [1.0, 2.0, 1.0]
    fn reset_pane_ratios(&mut self, cx: &mut Context<Self>) {
        let ws = self.config_store.active_workspace_mut();
        ws.pane_ratios = [1.0, 2.0, 1.0];
        self.schedule_save(cx);
        cx.notify();
    }

    /// Calculate normalized ratios for only visible panes
    #[allow(clippy::unused_self)] // Method for consistency with other instance methods
    fn normalized_visible_ratios(&self, ratios: &[f32; 3], visible: [bool; 3]) -> [f32; 3] {
        let visible_sum: f32 = ratios
            .iter()
            .enumerate()
            .filter(|(i, _)| visible[*i])
            .map(|(_, r)| r)
            .sum();

        if visible_sum == 0.0 {
            return [0.0, 0.0, 0.0];
        }

        [
            if visible[0] {
                ratios[0] / visible_sum
            } else {
                0.0
            },
            if visible[1] {
                ratios[1] / visible_sum
            } else {
                0.0
            },
            if visible[2] {
                ratios[2] / visible_sum
            } else {
                0.0
            },
        ]
    }

    fn available_panes_width(&self, visible: [bool; 3]) -> Pixels {
        let Some(content_width) = self.content_width else {
            return px(0.0);
        };
        let divider_count =
            usize::from(visible[0] && visible[1]) + usize::from(visible[1] && visible[2]);
        let divider_factor = match divider_count {
            0 => 0.0,
            1 => 1.0,
            _ => 2.0,
        };
        let divider_width = DIVIDER_WIDTH * divider_factor;
        let available = content_width - divider_width;
        if available > px(0.0) {
            available
        } else {
            px(0.0)
        }
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

    /// Render three-pane content area with dynamic flex basis and dividers
    fn render_content(&self, cx: &mut Context<Self>) -> impl IntoElement {
        let ws = self.config_store.active_workspace();
        let ratios = ws.pane_ratios;
        let visible = [
            ws.file_browser_visible,
            ws.terminal_visible,
            ws.document_viewer_visible,
        ];
        let available_width = self.available_panes_width(visible);

        // Calculate normalized ratios for visible panes only
        let visible_ratios = self.normalized_visible_ratios(&ratios, visible);

        div()
            .flex()
            .flex_1()
            .w_full()
            .relative()
            .child(
                div()
                    .absolute()
                    .size_full()
                    .child(ContentBoundsReporter::new(
                        cx.weak_entity(),
                        "workspace-content-bounds",
                    )),
            )
            // Global mouse event handlers for drag continuation
            .on_mouse_move(cx.listener(|this, event: &MouseMoveEvent, _window, cx| {
                this.handle_resize_drag(event.position.x, cx);
            }))
            .on_mouse_up(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.end_resize_drag(cx);
                }),
            )
            .on_mouse_up_out(
                MouseButton::Left,
                cx.listener(|this, _event, _window, cx| {
                    this.end_resize_drag(cx);
                }),
            )
            // File browser
            .when(visible[0], |d| {
                d.child(
                    div()
                        .flex_basis(relative(visible_ratios[0]))
                        .child(self.render_pane(PaneType::FileBrowser, cx)),
                )
            })
            // Divider 0 (between file browser and terminal)
            .when(visible[0] && visible[1], |d| {
                d.child(self.render_divider(0, available_width, cx))
            })
            // Terminal
            .when(visible[1], |d| {
                d.child(
                    div()
                        .flex_basis(relative(visible_ratios[1]))
                        .child(self.render_pane(PaneType::Terminal, cx)),
                )
            })
            // Divider 1 (between terminal and doc viewer)
            .when(visible[1] && visible[2], |d| {
                d.child(self.render_divider(1, available_width, cx))
            })
            // Document viewer
            .when(visible[2], |d| {
                d.child(
                    div()
                        .flex_basis(relative(visible_ratios[2]))
                        .child(self.render_pane(PaneType::DocumentViewer, cx)),
                )
            })
    }

    /// Render a single pane (file browser and terminal use real components, doc viewer is placeholder)
    fn render_pane(&self, pane_type: PaneType, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme();

        // File browser pane renders the actual FileBrowserPane component
        if pane_type == PaneType::FileBrowser {
            return div()
                .flex_1()
                .flex()
                .flex_col()
                .m_2()
                .bg(theme.colors().panel_background)
                .rounded_md()
                .overflow_hidden()
                .child(self.file_browser_pane.clone());
        }

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

        // Document viewer renders placeholder
        let label = "Document Viewer";

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

fn calculate_new_ratios(
    drag_state: &ResizeDragState,
    delta: Pixels,
    visible: [bool; 3],
    total_width: Pixels,
) -> [f32; 3] {
    let mut ratios = drag_state.start_ratios;
    let visible_count = visible.iter().filter(|v| **v).count();
    if visible_count < 2 || total_width <= px(0.0) {
        return ratios;
    }

    let visible_factor = match visible_count {
        0 => 0.0,
        1 => 1.0,
        2 => 2.0,
        _ => 3.0,
    };
    let min_total_width = MIN_PANE_WIDTH * visible_factor;
    if total_width < min_total_width {
        return ratios;
    }

    let left_idx = drag_state.divider_index;
    if left_idx >= 2 {
        return ratios;
    }
    let right_idx = left_idx + 1;
    if !visible[left_idx] || !visible[right_idx] {
        return ratios;
    }

    let hidden_ratio_sum: f32 = ratios
        .iter()
        .enumerate()
        .filter(|(i, _)| !visible[*i])
        .map(|(_, ratio)| *ratio)
        .sum();
    let visible_ratio_total = 1.0 - hidden_ratio_sum;
    if visible_ratio_total <= 0.0 {
        return ratios;
    }

    let mut widths = [px(0.0); 3];
    for (idx, width) in widths.iter_mut().enumerate() {
        if visible[idx] {
            *width = (ratios[idx] / visible_ratio_total) * total_width;
        }
    }

    widths[left_idx] += delta;
    widths[right_idx] -= delta;

    let mut left = widths[left_idx];
    let mut right = widths[right_idx];

    if left < MIN_PANE_WIDTH {
        let deficit = MIN_PANE_WIDTH - left;
        left = MIN_PANE_WIDTH;
        right -= deficit;
    }

    if right < MIN_PANE_WIDTH {
        let deficit = MIN_PANE_WIDTH - right;
        right = MIN_PANE_WIDTH;
        left -= deficit;
    }

    if left < MIN_PANE_WIDTH || right < MIN_PANE_WIDTH {
        return ratios;
    }

    widths[left_idx] = left;
    widths[right_idx] = right;

    for (idx, ratio) in ratios.iter_mut().enumerate() {
        if visible[idx] {
            *ratio = (widths[idx] / total_width) * visible_ratio_total;
        }
    }

    ratios
}

#[cfg(test)]
mod tests {
    use super::{calculate_new_ratios, ResizeDragState};
    use gpui::px;

    fn assert_approx(actual: f32, expected: f32) {
        assert!(
            (actual - expected).abs() < 1e-3,
            "actual={actual} expected={expected}"
        );
    }

    #[test]
    fn calculate_new_ratios_keeps_hidden_pane_ratios() {
        let drag_state = ResizeDragState {
            divider_index: 1,
            start_x: px(0.0),
            start_ratios: [0.2, 0.4, 0.4],
            total_width: px(400.0),
        };
        let visible = [false, true, true];
        let result = calculate_new_ratios(&drag_state, px(40.0), visible, px(400.0));

        assert_approx(result[0], 0.2);
        assert_approx(result[1], 0.48);
        assert_approx(result[2], 0.32);
    }

    #[test]
    fn calculate_new_ratios_returns_original_when_too_narrow() {
        let drag_state = ResizeDragState {
            divider_index: 0,
            start_x: px(0.0),
            start_ratios: [0.33, 0.33, 0.34],
            total_width: px(200.0),
        };
        let visible = [true, true, true];
        let result = calculate_new_ratios(&drag_state, px(20.0), visible, px(200.0));

        assert_approx(result[0], 0.33);
        assert_approx(result[1], 0.33);
        assert_approx(result[2], 0.34);
    }

    #[test]
    fn calculate_new_ratios_clamps_min_widths() {
        let drag_state = ResizeDragState {
            divider_index: 0,
            start_x: px(0.0),
            start_ratios: [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0],
            total_width: px(600.0),
        };
        let visible = [true, true, true];
        let result = calculate_new_ratios(&drag_state, px(80.0), visible, px(600.0));

        assert_approx(result[0], 250.0 / 600.0);
        assert_approx(result[1], 150.0 / 600.0);
        assert_approx(result[2], 200.0 / 600.0);
    }

    #[test]
    fn calculate_new_ratios_rejects_invalid_clamp() {
        let drag_state = ResizeDragState {
            divider_index: 0,
            start_x: px(0.0),
            start_ratios: [1.0 / 3.0, 1.0 / 3.0, 1.0 / 3.0],
            total_width: px(300.0),
        };
        let visible = [true, true, true];
        let result = calculate_new_ratios(&drag_state, px(100.0), visible, px(300.0));

        assert_approx(result[0], 1.0 / 3.0);
        assert_approx(result[1], 1.0 / 3.0);
        assert_approx(result[2], 1.0 / 3.0);
    }
}
