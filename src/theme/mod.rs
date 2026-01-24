//! Theme system for terminal and UI styling

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b, a: 255 }
    }

    pub const fn with_alpha(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    pub name: String,

    // Terminal colors (ANSI palette)
    pub ansi_colors: [Color; 16],

    // UI colors
    pub foreground: Color,
    pub background: Color,
    pub accent: Color,

    // Status colors
    pub success: Color,
    pub error: Color,
    pub warning: Color,
    pub info: Color,

    // Text colors
    pub text_muted: Color,
    pub text_hover: Color,
    pub text_selected: Color,
}

impl Theme {
    /// Get built-in theme by name
    pub fn by_name(name: &str) -> Option<Self> {
        match name {
            "dark" => Some(Self::dark()),
            "light" => Some(Self::light()),
            _ => None,
        }
    }

    /// Dark theme (default)
    pub fn dark() -> Self {
        Self {
            name: "dark".to_string(),
            ansi_colors: [
                Color::new(0, 0, 0),
                Color::new(205, 49, 49),
                Color::new(19, 161, 14),
                Color::new(229, 229, 16),
                Color::new(36, 114, 200),
                Color::new(188, 63, 60),
                Color::new(17, 168, 205),
                Color::new(204, 204, 204),
                Color::new(118, 118, 118),
                Color::new(241, 76, 76),
                Color::new(35, 209, 139),
                Color::new(245, 245, 67),
                Color::new(59, 142, 234),
                Color::new(214, 112, 214),
                Color::new(41, 184, 219),
                Color::new(229, 229, 229),
            ],
            foreground: Color::new(229, 229, 229),
            background: Color::new(26, 26, 26),
            accent: Color::new(59, 142, 234),
            success: Color::new(35, 209, 139),
            error: Color::new(241, 76, 76),
            warning: Color::new(245, 245, 67),
            info: Color::new(59, 142, 234),
            text_muted: Color::new(118, 118, 118),
            text_hover: Color::new(150, 150, 150),
            text_selected: Color::new(59, 142, 234),
        }
    }

    /// Light theme
    pub fn light() -> Self {
        Self {
            name: "light".to_string(),
            ansi_colors: [
                Color::new(255, 255, 255),
                Color::new(200, 40, 41),
                Color::new(19, 161, 14),
                Color::new(181, 137, 0),
                Color::new(25, 55, 109),
                Color::new(136, 23, 152),
                Color::new(17, 168, 205),
                Color::new(51, 51, 51),
                Color::new(102, 102, 102),
                Color::new(220, 50, 47),
                Color::new(133, 153, 0),
                Color::new(181, 137, 0),
                Color::new(38, 139, 210),
                Color::new(108, 113, 196),
                Color::new(42, 161, 152),
                Color::new(0, 0, 0),
            ],
            foreground: Color::new(51, 51, 51),
            background: Color::new(253, 246, 227),
            accent: Color::new(38, 139, 210),
            success: Color::new(133, 153, 0),
            error: Color::new(220, 50, 47),
            warning: Color::new(181, 137, 0),
            info: Color::new(38, 139, 210),
            text_muted: Color::new(147, 161, 161),
            text_hover: Color::new(101, 123, 113),
            text_selected: Color::new(38, 139, 210),
        }
    }
}
