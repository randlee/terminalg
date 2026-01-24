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

    #[allow(dead_code)] // Part of public API, will be used when custom themes are supported
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_color_new() {
        let color = Color::new(255, 128, 64);

        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 255); // Default alpha
    }

    #[test]
    fn test_color_with_alpha() {
        let color = Color::with_alpha(255, 128, 64, 128);

        assert_eq!(color.r, 255);
        assert_eq!(color.g, 128);
        assert_eq!(color.b, 64);
        assert_eq!(color.a, 128);
    }

    #[test]
    fn test_color_serialization() {
        let color = Color::new(100, 150, 200);

        let json = serde_json::to_string(&color).expect("Failed to serialize Color");

        let deserialized: Color = serde_json::from_str(&json).expect("Failed to deserialize Color");

        assert_eq!(deserialized.r, color.r);
        assert_eq!(deserialized.g, color.g);
        assert_eq!(deserialized.b, color.b);
        assert_eq!(deserialized.a, color.a);
    }

    #[test]
    fn test_theme_by_name_returns_dark() {
        let theme = Theme::by_name("dark");

        assert!(theme.is_some());
        let theme = theme.unwrap();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_theme_by_name_returns_light() {
        let theme = Theme::by_name("light");

        assert!(theme.is_some());
        let theme = theme.unwrap();
        assert_eq!(theme.name, "light");
    }

    #[test]
    fn test_theme_by_name_returns_none_for_unknown() {
        let theme = Theme::by_name("unknown_theme");
        assert!(theme.is_none());

        let theme = Theme::by_name("nord");
        assert!(theme.is_none());

        let theme = Theme::by_name("");
        assert!(theme.is_none());
    }

    #[test]
    fn test_theme_dark_has_correct_name() {
        let theme = Theme::dark();
        assert_eq!(theme.name, "dark");
    }

    #[test]
    fn test_theme_light_has_correct_name() {
        let theme = Theme::light();
        assert_eq!(theme.name, "light");
    }

    #[test]
    fn test_theme_dark_has_16_ansi_colors() {
        let theme = Theme::dark();
        assert_eq!(theme.ansi_colors.len(), 16);
    }

    #[test]
    fn test_theme_light_has_16_ansi_colors() {
        let theme = Theme::light();
        assert_eq!(theme.ansi_colors.len(), 16);
    }

    #[test]
    fn test_theme_dark_colors_are_valid() {
        let theme = Theme::dark();

        // Check that all colors have valid alpha values
        assert_eq!(theme.foreground.a, 255);
        assert_eq!(theme.background.a, 255);
        assert_eq!(theme.accent.a, 255);
        assert_eq!(theme.success.a, 255);
        assert_eq!(theme.error.a, 255);
        assert_eq!(theme.warning.a, 255);
        assert_eq!(theme.info.a, 255);
        assert_eq!(theme.text_muted.a, 255);
        assert_eq!(theme.text_hover.a, 255);
        assert_eq!(theme.text_selected.a, 255);

        // Check all ANSI colors have alpha
        for color in &theme.ansi_colors {
            assert_eq!(color.a, 255);
        }
    }

    #[test]
    fn test_theme_light_colors_are_valid() {
        let theme = Theme::light();

        // Check that all colors have valid alpha values
        assert_eq!(theme.foreground.a, 255);
        assert_eq!(theme.background.a, 255);
        assert_eq!(theme.accent.a, 255);
        assert_eq!(theme.success.a, 255);
        assert_eq!(theme.error.a, 255);
        assert_eq!(theme.warning.a, 255);
        assert_eq!(theme.info.a, 255);
        assert_eq!(theme.text_muted.a, 255);
        assert_eq!(theme.text_hover.a, 255);
        assert_eq!(theme.text_selected.a, 255);

        // Check all ANSI colors have alpha
        for color in &theme.ansi_colors {
            assert_eq!(color.a, 255);
        }
    }

    #[test]
    fn test_theme_dark_background_is_dark() {
        let theme = Theme::dark();

        // Dark theme should have a dark background
        // RGB values should be low
        assert!(theme.background.r < 100);
        assert!(theme.background.g < 100);
        assert!(theme.background.b < 100);
    }

    #[test]
    fn test_theme_light_background_is_light() {
        let theme = Theme::light();

        // Light theme should have a light background
        // RGB values should be high
        assert!(theme.background.r > 200);
        assert!(theme.background.g > 200);
        assert!(theme.background.b > 200);
    }

    #[test]
    fn test_theme_serialization() {
        let theme = Theme::dark();

        let json = serde_json::to_string(&theme).expect("Failed to serialize Theme");

        let deserialized: Theme = serde_json::from_str(&json).expect("Failed to deserialize Theme");

        assert_eq!(deserialized.name, theme.name);
        assert_eq!(deserialized.foreground.r, theme.foreground.r);
        assert_eq!(deserialized.background.g, theme.background.g);
        assert_eq!(deserialized.ansi_colors.len(), theme.ansi_colors.len());
    }

    #[test]
    fn test_theme_clone() {
        let theme = Theme::dark();
        let cloned = theme.clone();

        assert_eq!(cloned.name, theme.name);
        assert_eq!(cloned.foreground.r, theme.foreground.r);
        assert_eq!(cloned.background.g, theme.background.g);
    }

    #[test]
    fn test_color_copy() {
        let color1 = Color::new(100, 150, 200);
        let color2 = color1; // This should copy, not move

        // Both should be usable
        assert_eq!(color1.r, 100);
        assert_eq!(color2.r, 100);
    }

    #[test]
    fn test_theme_dark_and_light_are_different() {
        let dark = Theme::dark();
        let light = Theme::light();

        // Names should be different
        assert_ne!(dark.name, light.name);

        // Backgrounds should be different
        assert_ne!(dark.background.r, light.background.r);
        assert_ne!(dark.foreground.r, light.foreground.r);
    }
}
