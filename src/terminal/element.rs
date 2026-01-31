//! Custom terminal element for proper sizing
//!
//! This element calculates terminal dimensions from actual layout bounds
//! during the prepaint phase, ensuring the PTY receives correct size information.

use gpui::{
    point, px, App, Bounds, Element, ElementId, Entity, Font, FontFeatures, FontStyle,
    GlobalElementId, Hitbox, HitboxBehavior, Hsla, IntoElement, LayoutId, Pixels, Point,
    SharedString, Size, Style, TextAlign, TextRun, Window,
};
use settings::Settings;
use std::collections::BTreeMap;
use terminal::alacritty_terminal::index::Point as AlacPoint;
use terminal::terminal_settings::TerminalSettings;
use terminal::{Terminal, TerminalBounds, TerminalContent};
use theme::{ActiveTheme, ThemeSettings};

/// Helper struct for converting between Alacritty's cursor points and display cursor points.
/// Following Zed's terminal_element.rs pattern (lines 59-79)
#[derive(Debug, Clone, Copy)]
struct DisplayCursor {
    line: i32,
    col: usize,
}

impl DisplayCursor {
    /// Create a display cursor from an Alacritty cursor point and display offset.
    /// The display_offset accounts for scrollback, transforming Alacritty's coordinate
    /// system (where negative lines are scrollback) into screen coordinates (0, 1, 2...).
    fn from(cursor_point: AlacPoint, display_offset: usize) -> Self {
        Self {
            line: cursor_point.line.0 + display_offset as i32,
            col: cursor_point.column.0,
        }
    }

    fn line(&self) -> i32 {
        self.line
    }

    fn col(&self) -> usize {
        self.col
    }
}

/// Layout state computed during prepaint
pub struct TerminalLayoutState {
    #[allow(dead_code)] // For future mouse event handling
    hitbox: Hitbox,
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
    /// Lines indexed by screen position (0, 1, 2...), not by raw line.0 values
    pub lines: Vec<String>,
    /// Cursor display line (adjusted with display_offset)
    pub display_cursor_line: i32,
    /// Cursor column
    pub cursor_col: usize,
    /// Number of rows in the terminal viewport
    #[allow(dead_code)] // Used for debugging/future features
    pub viewport_rows: usize,
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

    /// Build lines from terminal content using enumerated screen positions.
    /// Following Zed's pattern from terminal_element.rs lines 1117-1124.
    ///
    /// This approach:
    /// 1. Groups cells by their line.0 value
    /// 2. Enumerates the groups to get screen positions (0, 1, 2...)
    /// 3. Builds a Vec indexed by screen position
    ///
    /// This works for both positive lines AND negative scrollback lines because
    /// we enumerate line groups rather than using raw line.0 values.
    fn build_lines_from_content(content: &TerminalContent) -> Vec<String> {
        if content.cells.is_empty() {
            return Vec::new();
        }

        // Group cells by line, creating a sorted map of line.0 -> cells
        let mut lines_map: BTreeMap<i32, Vec<char>> = BTreeMap::new();

        for cell in &content.cells {
            lines_map
                .entry(cell.point.line.0)
                .or_default()
                .push(cell.c);
        }

        // Convert to Vec<String> using enumerated positions (screen coordinates)
        // The BTreeMap automatically sorts by line.0, so enumeration gives us
        // screen positions: line 0 at index 0, line 1 at index 1, etc.
        lines_map
            .into_values()
            .map(|chars| chars.into_iter().collect::<String>())
            .collect()
    }

    /// Calculate cursor position and width, following Zed's shape_cursor pattern
    /// from terminal_element.rs lines 504-528.
    ///
    /// Returns Some((position, width)) if cursor is visible, None otherwise.
    fn shape_cursor(
        cursor: DisplayCursor,
        dimensions: &TerminalBounds,
    ) -> Option<(Point<Pixels>, Pixels)> {
        // Only render cursor if it's within the visible viewport
        if cursor.line() >= 0 && cursor.line() < dimensions.num_lines() as i32 {
            let cursor_position = point(
                (cursor.col() as f32 * dimensions.cell_width()).floor(),
                (cursor.line() as f32 * dimensions.line_height()).floor(),
            );

            // Use cell_width for cursor width
            let cursor_width = dimensions.cell_width().ceil();

            Some((cursor_position, cursor_width))
        } else {
            None
        }
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

        // Guard against narrow widths that cause alacritty to misbehave
        // See: https://github.com/zed-industries/zed/issues/2750
        let mut size = bounds.size;
        if size.width < cell_width * 2.0 {
            size.width = cell_width * 2.0;
        }
        let clamped_bounds = Bounds {
            origin: bounds.origin,
            size,
        };

        // Create terminal bounds from clamped layout bounds
        let dimensions = TerminalBounds::new(line_height, cell_width, clamped_bounds);

        // Set terminal size and sync to populate cells
        self.terminal.update(cx, |terminal, cx| {
            terminal.set_size(dimensions);
            terminal.sync(window, cx);
        });

        // Get content snapshot
        let content = self.terminal.read(cx).last_content();
        let lines = Self::build_lines_from_content(content);
        let viewport_rows = dimensions.num_lines();

        // Create display cursor following Zed's pattern (line 1137)
        let display_cursor = DisplayCursor::from(content.cursor.point, content.display_offset);

        let content_snapshot = TerminalContentSnapshot {
            lines,
            display_cursor_line: display_cursor.line(),
            cursor_col: display_cursor.col(),
            viewport_rows,
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
        // Lines are already indexed by screen position (0, 1, 2...) from build_lines_from_content
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

        // Paint cursor following Zed's pattern
        let display_cursor = DisplayCursor {
            line: layout.content.display_cursor_line,
            col: layout.content.cursor_col,
        };

        if let Some((cursor_position, cursor_width)) =
            Self::shape_cursor(display_cursor, &layout.dimensions)
        {
            let cursor_bounds = Bounds {
                origin: point(
                    bounds.origin.x + cursor_position.x,
                    bounds.origin.y + cursor_position.y,
                ),
                size: gpui::size(cursor_width, layout.line_height),
            };

            // Paint cursor as a filled rectangle
            window.paint_quad(gpui::fill(cursor_bounds, cursor_color));
        }
    }
}
