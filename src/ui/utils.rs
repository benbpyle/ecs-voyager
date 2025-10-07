//! UI utility functions for text processing and layout management.
//!
//! This module provides helper functions for truncating text, word wrapping,
//! terminal size validation, and responsive layout calculations.

use ratatui::layout::{Constraint, Direction, Layout, Rect};

/// Minimum terminal dimensions
pub const MIN_TERMINAL_WIDTH: u16 = 80;
pub const MIN_TERMINAL_HEIGHT: u16 = 24;

/// Validates that the terminal meets minimum size requirements
///
/// # Arguments
/// * `width` - Current terminal width
/// * `height` - Current terminal height
///
/// # Returns
/// Returns `Ok(())` if terminal is large enough, or an error message if too small
pub fn validate_terminal_size(width: u16, height: u16) -> Result<(), String> {
    if width < MIN_TERMINAL_WIDTH || height < MIN_TERMINAL_HEIGHT {
        Err(format!(
            "Terminal too small! Minimum size: {MIN_TERMINAL_WIDTH}x{MIN_TERMINAL_HEIGHT}, Current: {width}x{height}"
        ))
    } else {
        Ok(())
    }
}

/// Truncates text to fit within a maximum width, adding ellipsis if needed
///
/// # Arguments
/// * `text` - The text to truncate
/// * `max_width` - Maximum width in characters
///
/// # Returns
/// Truncated string with "..." appended if truncation occurred
///
/// # Examples
/// ```
/// use ecs_voyager::ui::utils::truncate_text;
///
/// assert_eq!(truncate_text("Hello, World!", 10), "Hello, ...");
/// assert_eq!(truncate_text("Short", 10), "Short");
/// ```
#[allow(dead_code)]
pub fn truncate_text(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        text.to_string()
    } else if max_width <= 3 {
        "...".to_string()
    } else {
        let truncated = &text[..max_width.saturating_sub(3)];
        format!("{truncated}...")
    }
}

/// Truncates text in the middle, preserving start and end
///
/// Useful for long identifiers like ARNs where both start and end are important
///
/// # Arguments
/// * `text` - The text to truncate
/// * `max_width` - Maximum width in characters
///
/// # Examples
/// ```
/// use ecs_voyager::ui::utils::truncate_middle;
///
/// assert_eq!(truncate_middle("arn:aws:ecs:us-east-1:123456:task/abc123", 20), "arn:aws:e...sk/abc123");
/// ```
#[allow(dead_code)]
pub fn truncate_middle(text: &str, max_width: usize) -> String {
    if text.len() <= max_width {
        text.to_string()
    } else if max_width <= 5 {
        "...".to_string()
    } else {
        let ellipsis = "...";
        let remaining = max_width.saturating_sub(ellipsis.len());
        let start_len = remaining / 2;
        let end_len = remaining.saturating_sub(start_len);

        let start = &text[..start_len];
        let end = &text[text.len().saturating_sub(end_len)..];

        format!("{start}{ellipsis}{end}")
    }
}

/// Wraps text to fit within a given width, breaking on word boundaries
///
/// # Arguments
/// * `text` - The text to wrap
/// * `width` - Maximum line width
///
/// # Returns
/// Vector of wrapped lines
#[allow(dead_code)]
pub fn wrap_text(text: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return vec![];
    }

    let mut lines = Vec::new();
    let mut current_line = String::new();
    let mut current_len = 0;

    for word in text.split_whitespace() {
        let word_len = word.len();

        // If word itself is longer than width, break it
        if word_len > width {
            if !current_line.is_empty() {
                lines.push(current_line.trim().to_string());
                current_line.clear();
                current_len = 0;
            }

            // Break long word into chunks
            let mut remaining = word;
            while remaining.len() > width {
                lines.push(remaining[..width].to_string());
                remaining = &remaining[width..];
            }
            if !remaining.is_empty() {
                current_line = remaining.to_string();
                current_len = remaining.len();
            }
            continue;
        }

        // Check if adding this word would exceed width
        let space_needed = if current_line.is_empty() { 0 } else { 1 }; // Space before word
        if current_len + space_needed + word_len > width {
            // Start new line
            if !current_line.is_empty() {
                lines.push(current_line.trim().to_string());
                current_line.clear();
                current_len = 0;
            }
        }

        // Add word to current line
        if !current_line.is_empty() {
            current_line.push(' ');
            current_len += 1;
        }
        current_line.push_str(word);
        current_len += word_len;
    }

    // Don't forget the last line
    if !current_line.is_empty() {
        lines.push(current_line.trim().to_string());
    }

    // Return at least one line (even if empty)
    if lines.is_empty() {
        lines.push(String::new());
    }

    lines
}

/// Adds line numbers to text lines
///
/// # Arguments
/// * `lines` - The lines to number
/// * `start_number` - Starting line number (1-based)
///
/// # Returns
/// Vector of lines with line numbers prepended
#[allow(dead_code)]
pub fn add_line_numbers(lines: &[String], start_number: usize) -> Vec<String> {
    let max_line_num = start_number + lines.len().saturating_sub(1);
    let width = format!("{max_line_num}").len();

    lines
        .iter()
        .enumerate()
        .map(|(i, line)| {
            let line_num = start_number + i;
            format!("{line_num:>width$} | {line}")
        })
        .collect()
}

/// Creates a centered area within a parent area
///
/// # Arguments
/// * `parent` - The parent area
/// * `width` - Desired width (or parent width if larger)
/// * `height` - Desired height (or parent height if larger)
///
/// # Returns
/// A centered Rect within the parent
#[allow(dead_code)]
pub fn centered_rect(parent: Rect, width: u16, height: u16) -> Rect {
    let width = width.min(parent.width);
    let height = height.min(parent.height);

    Rect {
        x: parent.x + (parent.width.saturating_sub(width)) / 2,
        y: parent.y + (parent.height.saturating_sub(height)) / 2,
        width,
        height,
    }
}

/// Creates a split-pane layout (two equal vertical panes)
///
/// # Arguments
/// * `area` - The area to split
///
/// # Returns
/// Tuple of (left_pane, right_pane) Rect areas
#[allow(dead_code)]
pub fn split_pane_layout(area: Rect) -> (Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(area);

    (chunks[0], chunks[1])
}

/// Creates a three-column layout with configurable widths
///
/// # Arguments
/// * `area` - The area to split
/// * `left_percent` - Width percentage for left column
/// * `right_percent` - Width percentage for right column
///
/// # Returns
/// Tuple of (left, center, right) Rect areas
#[allow(dead_code)]
pub fn three_column_layout(
    area: Rect,
    left_percent: u16,
    right_percent: u16,
) -> (Rect, Rect, Rect) {
    let left = left_percent.min(100);
    let right = right_percent.min(100);
    let center = 100u16.saturating_sub(left).saturating_sub(right);

    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(left),
            Constraint::Percentage(center),
            Constraint::Percentage(right),
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2])
}

/// Calculates responsive column widths based on terminal width
///
/// Adjusts layout for narrow terminals by reducing or hiding columns
///
/// # Arguments
/// * `terminal_width` - Current terminal width
/// * `full_widths` - Column widths for full-size display
///
/// # Returns
/// Adjusted column widths
#[allow(dead_code)]
pub fn responsive_column_widths(terminal_width: u16, full_widths: &[u16]) -> Vec<u16> {
    if terminal_width >= 120 {
        // Full size
        full_widths.to_vec()
    } else if terminal_width >= 100 {
        // Slightly compressed
        full_widths.iter().map(|w| (w * 90) / 100).collect()
    } else {
        // Highly compressed - drop last column if possible
        if full_widths.len() > 3 {
            full_widths[..full_widths.len() - 1].to_vec()
        } else {
            full_widths.iter().map(|w| (w * 80) / 100).collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_validate_terminal_size_valid() {
        assert!(validate_terminal_size(80, 24).is_ok());
        assert!(validate_terminal_size(120, 40).is_ok());
        assert!(validate_terminal_size(200, 60).is_ok());
    }

    #[test]
    fn test_validate_terminal_size_too_small() {
        assert!(validate_terminal_size(79, 24).is_err());
        assert!(validate_terminal_size(80, 23).is_err());
        assert!(validate_terminal_size(50, 20).is_err());
    }

    #[test]
    fn test_validate_terminal_size_error_message() {
        let err = validate_terminal_size(60, 20).unwrap_err();
        assert!(err.contains("Terminal too small"));
        assert!(err.contains("80x24"));
        assert!(err.contains("60x20"));
    }

    #[test]
    fn test_truncate_text_no_truncation() {
        assert_eq!(truncate_text("Short", 10), "Short");
        assert_eq!(truncate_text("Exact", 5), "Exact");
    }

    #[test]
    fn test_truncate_text_with_truncation() {
        assert_eq!(truncate_text("Hello, World!", 10), "Hello, ...");
        assert_eq!(truncate_text("Very long text here", 10), "Very lo...");
    }

    #[test]
    fn test_truncate_text_very_short_width() {
        assert_eq!(truncate_text("Hello", 3), "...");
        assert_eq!(truncate_text("Hello", 2), "...");
        assert_eq!(truncate_text("Hello", 1), "...");
    }

    #[test]
    fn test_truncate_text_empty() {
        assert_eq!(truncate_text("", 10), "");
    }

    #[test]
    fn test_truncate_middle_no_truncation() {
        assert_eq!(truncate_middle("Short", 10), "Short");
    }

    #[test]
    fn test_truncate_middle_with_truncation() {
        assert_eq!(
            truncate_middle("arn:aws:ecs:us-east-1:123456:task/abc123", 20),
            "arn:aws:...sk/abc123"
        );
    }

    #[test]
    fn test_truncate_middle_very_short() {
        assert_eq!(truncate_middle("Very long text", 5), "...");
    }

    #[test]
    fn test_wrap_text_no_wrapping() {
        let result = wrap_text("Short text", 20);
        assert_eq!(result, vec!["Short text"]);
    }

    #[test]
    fn test_wrap_text_basic_wrapping() {
        let result = wrap_text("This is a long line that needs wrapping", 10);
        assert_eq!(result.len(), 4);
        assert!(result[0].len() <= 10);
        assert!(result.iter().all(|line| line.len() <= 10));
    }

    #[test]
    fn test_wrap_text_long_word() {
        let result = wrap_text("Verylongwordthatcannotfit normal words", 10);
        // The long word should be broken up
        assert!(result.len() >= 3);
    }

    #[test]
    fn test_wrap_text_preserves_content() {
        let original = "Hello world this is a test";
        let wrapped = wrap_text(original, 10);
        let rejoined = wrapped.join(" ");
        // Should contain all the words
        assert!(rejoined.contains("Hello"));
        assert!(rejoined.contains("world"));
        assert!(rejoined.contains("test"));
    }

    #[test]
    fn test_wrap_text_zero_width() {
        let result = wrap_text("Some text", 0);
        assert_eq!(result, Vec::<String>::new());
    }

    #[test]
    fn test_wrap_text_empty_string() {
        let result = wrap_text("", 10);
        assert_eq!(result, vec![""]);
    }

    #[test]
    fn test_add_line_numbers_basic() {
        let lines = vec!["First line".to_string(), "Second line".to_string()];
        let result = add_line_numbers(&lines, 1);

        assert_eq!(result.len(), 2);
        assert!(result[0].starts_with("1 |"));
        assert!(result[1].starts_with("2 |"));
    }

    #[test]
    fn test_add_line_numbers_custom_start() {
        let lines = vec!["Line".to_string()];
        let result = add_line_numbers(&lines, 42);

        assert!(result[0].starts_with("42 |"));
    }

    #[test]
    fn test_add_line_numbers_alignment() {
        let lines = vec![
            "Line 1".to_string(),
            "Line 2".to_string(),
            "Line 10".to_string(),
        ];
        let result = add_line_numbers(&lines, 8);

        // Line numbers should be right-aligned with consistent width
        assert!(result[0].starts_with(" 8 |"));
        assert!(result[1].starts_with(" 9 |"));
        assert!(result[2].starts_with("10 |"));
    }

    #[test]
    fn test_add_line_numbers_empty() {
        let lines: Vec<String> = vec![];
        let result = add_line_numbers(&lines, 1);
        assert_eq!(result.len(), 0);
    }

    #[test]
    fn test_centered_rect_normal() {
        let parent = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        let result = centered_rect(parent, 60, 30);

        assert_eq!(result.width, 60);
        assert_eq!(result.height, 30);
        assert_eq!(result.x, 20); // (100 - 60) / 2
        assert_eq!(result.y, 10); // (50 - 30) / 2
    }

    #[test]
    fn test_centered_rect_too_large() {
        let parent = Rect {
            x: 0,
            y: 0,
            width: 50,
            height: 25,
        };
        let result = centered_rect(parent, 100, 50);

        // Should be clamped to parent size
        assert_eq!(result.width, 50);
        assert_eq!(result.height, 25);
        assert_eq!(result.x, 0);
        assert_eq!(result.y, 0);
    }

    #[test]
    fn test_split_pane_layout() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        let (left, right) = split_pane_layout(area);

        // Should be split 50/50
        assert_eq!(left.width + right.width, 100);
        assert_eq!(left.height, 50);
        assert_eq!(right.height, 50);
    }

    #[test]
    fn test_three_column_layout() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        let (left, center, right) = three_column_layout(area, 20, 30);

        // Widths should sum to approximately 100 (may have rounding differences)
        let total = left.width + center.width + right.width;
        assert!(total >= 99 && total <= 100);
    }

    #[test]
    fn test_three_column_layout_overflow_protection() {
        let area = Rect {
            x: 0,
            y: 0,
            width: 100,
            height: 50,
        };
        // Request more than 100%
        let (left, center, right) = three_column_layout(area, 60, 60);

        // Should be clamped
        assert!(left.width <= 100);
        assert!(right.width <= 100);
        assert!(center.width <= 100);
    }

    #[test]
    fn test_responsive_column_widths_full_size() {
        let widths = vec![30, 25, 25, 20];
        let result = responsive_column_widths(120, &widths);
        assert_eq!(result, widths);
    }

    #[test]
    fn test_responsive_column_widths_compressed() {
        let widths = vec![30, 25, 25, 20];
        let result = responsive_column_widths(100, &widths);

        // Should be scaled down to 90%
        assert_eq!(result, vec![27, 22, 22, 18]);
    }

    #[test]
    fn test_responsive_column_widths_highly_compressed() {
        let widths = vec![30, 25, 25, 20];
        let result = responsive_column_widths(90, &widths);

        // Should drop last column
        assert_eq!(result.len(), 3);
        assert_eq!(result, vec![30, 25, 25]);
    }

    #[test]
    fn test_responsive_column_widths_three_columns() {
        let widths = vec![40, 30, 30];
        let result = responsive_column_widths(90, &widths);

        // With only 3 columns, should scale instead of dropping
        assert_eq!(result.len(), 3);
        assert_eq!(result, vec![32, 24, 24]);
    }
}
