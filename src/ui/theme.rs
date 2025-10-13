//! Theme system for ECS Voyager UI.
//!
//! This module provides a flexible theming system with support for dark, light, and custom themes.
//! Colors can be configured in the config file and are used consistently throughout the UI.

use ratatui::style::Color;
use serde::{Deserialize, Serialize};

/// Available theme presets
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ThemePreset {
    #[default]
    Dark,
    Light,
    Custom,
}

/// Complete color theme definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Theme {
    /// Theme preset name
    #[serde(default)]
    pub preset: ThemePreset,

    /// Color configuration
    #[serde(default)]
    pub colors: ThemeColors,
}

/// Individual color definitions for the theme
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeColors {
    // Primary UI colors
    /// Primary accent color (used for highlights, selected items)
    #[serde(default = "default_primary")]
    pub primary: String,

    /// Secondary accent color
    #[serde(default = "default_secondary")]
    pub secondary: String,

    /// Background color
    #[serde(default = "default_background")]
    pub background: String,

    /// Foreground/text color
    #[serde(default = "default_foreground")]
    pub foreground: String,

    // Status colors
    /// Success/positive messages
    #[serde(default = "default_success")]
    pub success: String,

    /// Warning messages
    #[serde(default = "default_warning")]
    pub warning: String,

    /// Error messages
    #[serde(default = "default_error")]
    pub error: String,

    /// Info messages
    #[serde(default = "default_info")]
    pub info: String,

    // Semantic colors
    /// Border color
    #[serde(default = "default_border")]
    pub border: String,

    /// Muted/disabled text
    #[serde(default = "default_muted")]
    pub muted: String,

    /// Highlighted text (selected background)
    #[serde(default = "default_highlight_bg")]
    pub highlight_bg: String,

    /// Highlighted text foreground
    #[serde(default = "default_highlight_fg")]
    pub highlight_fg: String,
}

// Default color functions for dark theme
fn default_primary() -> String {
    "cyan".to_string()
}
fn default_secondary() -> String {
    "magenta".to_string()
}
fn default_background() -> String {
    "black".to_string()
}
fn default_foreground() -> String {
    "white".to_string()
}
fn default_success() -> String {
    "green".to_string()
}
fn default_warning() -> String {
    "yellow".to_string()
}
fn default_error() -> String {
    "red".to_string()
}
fn default_info() -> String {
    "blue".to_string()
}
fn default_border() -> String {
    "white".to_string()
}
fn default_muted() -> String {
    "darkgray".to_string()
}
fn default_highlight_bg() -> String {
    "cyan".to_string()
}
fn default_highlight_fg() -> String {
    "black".to_string()
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self::dark()
    }
}

impl ThemeColors {
    /// Creates a dark theme color palette
    pub fn dark() -> Self {
        Self {
            primary: "cyan".to_string(),
            secondary: "magenta".to_string(),
            background: "black".to_string(),
            foreground: "white".to_string(),
            success: "green".to_string(),
            warning: "yellow".to_string(),
            error: "red".to_string(),
            info: "blue".to_string(),
            border: "white".to_string(),
            muted: "darkgray".to_string(),
            highlight_bg: "cyan".to_string(),
            highlight_fg: "black".to_string(),
        }
    }

    /// Creates a light theme color palette
    pub fn light() -> Self {
        Self {
            primary: "blue".to_string(),
            secondary: "magenta".to_string(),
            background: "white".to_string(),
            foreground: "black".to_string(),
            success: "green".to_string(),
            warning: "yellow".to_string(),
            error: "red".to_string(),
            info: "blue".to_string(),
            border: "black".to_string(),
            muted: "gray".to_string(),
            highlight_bg: "blue".to_string(),
            highlight_fg: "white".to_string(),
        }
    }

    /// Parses a color string into a ratatui Color (for future use)
    ///
    /// Supports named colors (red, green, blue, etc.) and hex colors (#RRGGBB)
    #[allow(dead_code)]
    pub fn parse_color(color_str: &str) -> Color {
        match color_str.to_lowercase().as_str() {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "gray" | "grey" => Color::Gray,
            "darkgray" | "darkgrey" => Color::DarkGray,
            "lightred" => Color::LightRed,
            "lightgreen" => Color::LightGreen,
            "lightyellow" => Color::LightYellow,
            "lightblue" => Color::LightBlue,
            "lightmagenta" => Color::LightMagenta,
            "lightcyan" => Color::LightCyan,
            "white" => Color::White,
            // Hex color support
            s if s.starts_with('#') && s.len() == 7 => {
                if let Ok(r) = u8::from_str_radix(&s[1..3], 16) {
                    if let Ok(g) = u8::from_str_radix(&s[3..5], 16) {
                        if let Ok(b) = u8::from_str_radix(&s[5..7], 16) {
                            return Color::Rgb(r, g, b);
                        }
                    }
                }
                Color::White // Fallback
            }
            _ => Color::White, // Default fallback
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            preset: ThemePreset::Dark,
            colors: ThemeColors::dark(),
        }
    }
}

impl Theme {
    /// Creates a new theme from a preset
    pub fn from_preset(preset: ThemePreset) -> Self {
        let colors = match preset {
            ThemePreset::Dark => ThemeColors::dark(),
            ThemePreset::Light => ThemeColors::light(),
            ThemePreset::Custom => ThemeColors::dark(), // Custom uses dark as base
        };

        Self { preset, colors }
    }

    /// Gets the primary color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn primary(&self) -> Color {
        ThemeColors::parse_color(&self.colors.primary)
    }

    /// Gets the secondary color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn secondary(&self) -> Color {
        ThemeColors::parse_color(&self.colors.secondary)
    }

    /// Gets the background color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn background(&self) -> Color {
        ThemeColors::parse_color(&self.colors.background)
    }

    /// Gets the foreground color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn foreground(&self) -> Color {
        ThemeColors::parse_color(&self.colors.foreground)
    }

    /// Gets the success color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn success(&self) -> Color {
        ThemeColors::parse_color(&self.colors.success)
    }

    /// Gets the warning color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn warning(&self) -> Color {
        ThemeColors::parse_color(&self.colors.warning)
    }

    /// Gets the error color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn error(&self) -> Color {
        ThemeColors::parse_color(&self.colors.error)
    }

    /// Gets the info color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn info(&self) -> Color {
        ThemeColors::parse_color(&self.colors.info)
    }

    /// Gets the border color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn border(&self) -> Color {
        ThemeColors::parse_color(&self.colors.border)
    }

    /// Gets the muted color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn muted(&self) -> Color {
        ThemeColors::parse_color(&self.colors.muted)
    }

    /// Gets the highlight background color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn highlight_bg(&self) -> Color {
        ThemeColors::parse_color(&self.colors.highlight_bg)
    }

    /// Gets the highlight foreground color as a ratatui Color (for future use)
    #[allow(dead_code)]
    pub fn highlight_fg(&self) -> Color {
        ThemeColors::parse_color(&self.colors.highlight_fg)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_default_theme_is_dark() {
        let theme = Theme::default();
        assert_eq!(theme.preset, ThemePreset::Dark);
    }

    #[test]
    fn test_dark_theme_colors() {
        let theme = Theme::from_preset(ThemePreset::Dark);
        assert_eq!(theme.primary(), Color::Cyan);
        assert_eq!(theme.success(), Color::Green);
        assert_eq!(theme.error(), Color::Red);
        assert_eq!(theme.warning(), Color::Yellow);
    }

    #[test]
    fn test_light_theme_colors() {
        let theme = Theme::from_preset(ThemePreset::Light);
        assert_eq!(theme.primary(), Color::Blue);
        assert_eq!(theme.background(), Color::White);
        assert_eq!(theme.foreground(), Color::Black);
    }

    #[test]
    fn test_parse_named_colors() {
        assert_eq!(ThemeColors::parse_color("red"), Color::Red);
        assert_eq!(ThemeColors::parse_color("green"), Color::Green);
        assert_eq!(ThemeColors::parse_color("blue"), Color::Blue);
        assert_eq!(ThemeColors::parse_color("cyan"), Color::Cyan);
        assert_eq!(ThemeColors::parse_color("yellow"), Color::Yellow);
        assert_eq!(ThemeColors::parse_color("magenta"), Color::Magenta);
    }

    #[test]
    fn test_parse_case_insensitive() {
        assert_eq!(ThemeColors::parse_color("RED"), Color::Red);
        assert_eq!(ThemeColors::parse_color("GrEeN"), Color::Green);
        assert_eq!(ThemeColors::parse_color("CYAN"), Color::Cyan);
    }

    #[test]
    fn test_parse_hex_colors() {
        assert_eq!(ThemeColors::parse_color("#FF0000"), Color::Rgb(255, 0, 0));
        assert_eq!(ThemeColors::parse_color("#00FF00"), Color::Rgb(0, 255, 0));
        assert_eq!(ThemeColors::parse_color("#0000FF"), Color::Rgb(0, 0, 255));
        assert_eq!(
            ThemeColors::parse_color("#FFFFFF"),
            Color::Rgb(255, 255, 255)
        );
        assert_eq!(ThemeColors::parse_color("#000000"), Color::Rgb(0, 0, 0));
    }

    #[test]
    fn test_parse_invalid_color_returns_white() {
        assert_eq!(ThemeColors::parse_color("invalid"), Color::White);
        assert_eq!(ThemeColors::parse_color(""), Color::White);
        assert_eq!(ThemeColors::parse_color("#FFF"), Color::White); // Wrong length
    }

    #[test]
    fn test_theme_preset_serialization() {
        let dark = ThemePreset::Dark;
        let light = ThemePreset::Light;
        let custom = ThemePreset::Custom;

        assert_eq!(serde_json::to_string(&dark).unwrap(), r#""dark""#);
        assert_eq!(serde_json::to_string(&light).unwrap(), r#""light""#);
        assert_eq!(serde_json::to_string(&custom).unwrap(), r#""custom""#);
    }

    #[test]
    fn test_theme_colors_default() {
        let colors = ThemeColors::default();
        assert_eq!(colors.primary, "cyan");
        assert_eq!(colors.error, "red");
        assert_eq!(colors.success, "green");
    }

    #[test]
    fn test_custom_theme_uses_dark_as_base() {
        let theme = Theme::from_preset(ThemePreset::Custom);
        assert_eq!(theme.preset, ThemePreset::Custom);
        assert_eq!(theme.colors.primary, "cyan"); // Same as dark
    }
}
