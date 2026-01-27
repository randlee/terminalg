//! Custom terminal element for proper sizing
//!
//! This element calculates terminal dimensions from actual layout bounds
//! during the prepaint phase, ensuring the PTY receives correct size information.

use gpui::{
    px, App, Bounds, Element, ElementId, Entity, Font, FontFeatures, FontStyle, GlobalElementId,
    Hitbox, HitboxBehavior, Hsla, IntoElement, LayoutId, Pixels, Point, SharedString, Size, Style,
    TextAlign, TextRun, Window,
};
use settings::Settings;
use terminal::terminal_settings::TerminalSettings;
use terminal::{Terminal, TerminalBounds, TerminalContent};
use theme::{ActiveTheme, ThemeSettings};

/// Layout state computed during prepaint
pub struct TerminalLayoutState {
    #[allow(dead_code)] // For future mouse event handling
    hitbox: Hitbox,
    #[allow(dead_code)] // For debugging/future use
    dimensions: TerminalBounds,
    content: TerminalContentSnapshot,
    background_color: Hsla,
    foreground_color: Hsla,
    line_height: Pixels,
    cell_width: Pixels,
    font: Font,
    font_size: Pixels,
}

/// Snapshot of terminal content for rendering
pub struct TerminalContentSnapshot {
    pub lines: Vec<String>,
    pub cursor_line: i32,
    pub cursor_col: usize,
}

/// Custom element that properly sizes the terminal based on layout bounds
pub struct TerminalElement {
    terminal: Entity<Terminal>,
    id: ElementId,
}

impl TerminalElement {
    pub fn new(terminal: Entity<Terminal>, id: impl Into<ElementId>) -> Self {
        Self {
            terminal,
            id: id.into(),
        }
    }

    fn build_lines_from_content(content: &TerminalContent) -> Vec<String> {
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

        lines
    }
}

impl IntoElement for TerminalElement {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for TerminalElement {
    type RequestLayoutState = ();
    type PrepaintState = TerminalLayoutState;

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
        // Request full available space
        let style = Style {
            flex_grow: 1.0,
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
        window: &mut Window,
        cx: &mut App,
    ) -> Self::PrepaintState {
        // Get theme colors BEFORE mutable borrow of cx
        let background_color = cx.theme().colors().terminal_background;
        let foreground_color = cx.theme().colors().terminal_foreground;

        let settings = ThemeSettings::get_global(cx);
        let terminal_settings = TerminalSettings::get_global(cx);

        // Get terminal font family - use terminal setting if set, otherwise use Courier
        // (a reliable monospace font) as fallback since buffer_font might be proportional
        let font_family: SharedString = terminal_settings.font_family.as_ref().map_or_else(
            || SharedString::from("Courier"),
            |font_family| font_family.0.clone().into(),
        );

        // Get font fallbacks
        let font_fallbacks = terminal_settings
            .font_fallbacks
            .as_ref()
            .or(settings.buffer_font.fallbacks.as_ref())
            .cloned();

        // Disable ligatures for terminal (standard practice)
        let font_features = terminal_settings
            .font_features
            .clone()
            .unwrap_or_else(FontFeatures::disable_ligatures);

        let font_weight = terminal_settings.font_weight.unwrap_or_default();

        // Build the terminal font
        let font = Font {
            family: font_family,
            features: font_features,
            fallbacks: font_fallbacks,
            weight: font_weight,
            style: FontStyle::Normal,
        };

        // Calculate font size - use terminal setting if set, otherwise buffer font size
        let rem_size = window.rem_size();
        let buffer_font_size = settings.buffer_font_size(cx);
        let font_size = terminal_settings.font_size.unwrap_or(buffer_font_size);

        // Get line height from terminal settings
        let line_height_setting = terminal_settings.line_height.value();
        let line_height = f32::from(font_size) * line_height_setting.to_pixels(rem_size);

        // Calculate cell width using the font's advance for 'm'
        let text_system = window.text_system();
        let font_id = text_system.resolve_font(&font);
        let cell_width = text_system
            .advance(font_id, font_size, 'm')
            .map(|advance| advance.width)
            .unwrap_or(px(8.4)); // Fallback

        // Create terminal bounds from actual layout bounds
        let dimensions = TerminalBounds::new(line_height, cell_width, bounds);

        // Set terminal size and sync to populate cells
        self.terminal.update(cx, |terminal, cx| {
            terminal.set_size(dimensions);
            terminal.sync(window, cx);
        });

        // Get content snapshot
        let content = self.terminal.read(cx).last_content();
        let lines = Self::build_lines_from_content(content);
        let content_snapshot = TerminalContentSnapshot {
            lines,
            cursor_line: content.cursor.point.line.0,
            cursor_col: content.cursor.point.column.0,
        };

        // Register hitbox for mouse events
        let hitbox = window.insert_hitbox(bounds, HitboxBehavior::Normal);

        TerminalLayoutState {
            hitbox,
            dimensions,
            content: content_snapshot,
            background_color,
            foreground_color,
            line_height,
            cell_width,
            font,
            font_size,
        }
    }

    fn paint(
        &mut self,
        _global_id: Option<&GlobalElementId>,
        _inspector_id: Option<&gpui::InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        layout: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let cursor_color = cx.theme().players().local().cursor;

        // Paint background
        window.paint_quad(gpui::fill(bounds, layout.background_color));

        // Paint each line of terminal content using the terminal font
        for (line_idx, line_text) in layout.content.lines.iter().enumerate() {
            if line_text.is_empty() {
                continue;
            }

            let y = bounds.origin.y + layout.line_height * line_idx;
            if y > bounds.origin.y + bounds.size.height {
                break; // Don't render lines outside viewport
            }

            let position = Point::new(bounds.origin.x, y);

            // Shape the line using window's text_system with the terminal font
            // Use force_width to ensure each glyph is positioned at glyph_index * cell_width
            let shaped_line = window.text_system().shape_line(
                SharedString::from(line_text.clone()),
                layout.font_size,
                &[TextRun {
                    len: line_text.len(),
                    font: layout.font.clone(),
                    color: layout.foreground_color,
                    background_color: None,
                    underline: None,
                    strikethrough: None,
                }],
                Some(layout.cell_width),
            );
            shaped_line
                .paint(
                    position,
                    layout.line_height,
                    TextAlign::Left,
                    None,
                    window,
                    cx,
                )
                .ok();
        }

        // Paint cursor (simple block cursor)
        #[allow(clippy::cast_sign_loss)] // cursor_line is always non-negative when visible
        let cursor_y =
            bounds.origin.y + layout.line_height * layout.content.cursor_line.max(0) as usize;
        let cursor_x = bounds.origin.x + layout.cell_width * layout.content.cursor_col;

        if cursor_y >= bounds.origin.y && cursor_y < bounds.origin.y + bounds.size.height {
            let cursor_bounds = Bounds {
                origin: Point::new(cursor_x, cursor_y),
                size: Size {
                    width: layout.cell_width,
                    height: layout.line_height,
                },
            };
            window.paint_quad(gpui::fill(cursor_bounds, cursor_color));
        }
    }
}
