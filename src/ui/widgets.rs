//! Reusable UI widgets for ECS Voyager.
//!
//! This module provides common UI components like spinners, progress bars,
//! toast notifications, dialogs, input fields, and dropdowns.

use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};
use std::time::{SystemTime, UNIX_EPOCH};

use super::theme::Theme;

/// Renders an animated loading spinner with a message
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - Optional area to render in (if None, renders centered modal)
/// * `message` - Status message to display
/// * `theme` - Theme for colors
#[allow(dead_code)]
pub fn render_spinner(f: &mut Frame, area: Option<Rect>, message: &str, theme: &Theme) {
    let target_area = area.unwrap_or_else(|| {
        let screen = f.area();
        let width = 50.min(screen.width.saturating_sub(4));
        let height = 7;

        Rect {
            x: screen.width.saturating_sub(width) / 2,
            y: screen.height.saturating_sub(height) / 2,
            width,
            height,
        }
    });

    if area.is_none() {
        f.render_widget(Clear, target_area);
    }

    let spinner = get_spinner_frame();
    let lines = vec![
        Line::from(""),
        Line::from(vec![Span::styled(
            format!("  {spinner}  "),
            Style::default()
                .fg(theme.primary())
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            message,
            Style::default().fg(theme.warning()),
        )]),
        Line::from(""),
    ];

    let widget = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .title("Loading")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary()))
            .style(Style::default().bg(theme.background())),
    );

    f.render_widget(widget, target_area);
}

/// Returns the current frame of a spinner animation
///
/// Uses a 10-frame Braille pattern spinner that updates every 80ms
#[allow(dead_code)]
pub fn get_spinner_frame() -> &'static str {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let index = ((now / 80) % frames.len() as u128) as usize;
    frames[index]
}

/// Renders a progress bar widget
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - Area to render the progress bar
/// * `progress` - Progress value (0.0 to 1.0)
/// * `label` - Label text to display
/// * `theme` - Theme for colors
#[allow(dead_code)]
pub fn render_progress_bar(f: &mut Frame, area: Rect, progress: f32, label: &str, theme: &Theme) {
    let progress = progress.clamp(0.0, 1.0);
    let bar_width = (area.width.saturating_sub(4) as f32 * progress) as u16;

    // Create progress bar string
    let filled = "█".repeat(bar_width as usize);
    let empty = "░".repeat((area.width.saturating_sub(4).saturating_sub(bar_width)) as usize);
    let percentage = (progress * 100.0) as u16;

    let lines = vec![
        Line::from(vec![Span::styled(
            label,
            Style::default().fg(theme.foreground()),
        )]),
        Line::from(vec![
            Span::styled(filled, Style::default().fg(theme.success())),
            Span::styled(empty, Style::default().fg(theme.muted())),
        ]),
        Line::from(vec![Span::styled(
            format!("{percentage}%"),
            Style::default()
                .fg(theme.primary())
                .add_modifier(Modifier::BOLD),
        )]),
    ];

    let widget = Paragraph::new(lines).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.border())),
    );

    f.render_widget(widget, area);
}

/// Toast notification type (for future use)
#[derive(Debug, Clone, PartialEq, Eq)]
#[allow(dead_code)]
pub enum ToastType {
    Success,
    Error,
    Warning,
    Info,
}

/// Renders a toast notification
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `message` - Message to display
/// * `toast_type` - Type of toast (affects color)
/// * `theme` - Theme for colors
#[allow(dead_code)]
pub fn render_toast(f: &mut Frame, message: &str, toast_type: ToastType, theme: &Theme) {
    let screen = f.area();
    let width = message.len().min(60) as u16 + 4;
    let height = 3;

    let area = Rect {
        x: screen.width.saturating_sub(width) / 2,
        y: screen.height.saturating_sub(height + 2),
        width,
        height,
    };

    f.render_widget(Clear, area);

    let (icon, color) = match toast_type {
        ToastType::Success => ("✓", theme.success()),
        ToastType::Error => ("✗", theme.error()),
        ToastType::Warning => ("⚠", theme.warning()),
        ToastType::Info => ("ℹ", theme.info()),
    };

    let text = Line::from(vec![
        Span::styled(
            format!("{icon} "),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled(message, Style::default().fg(theme.foreground())),
    ]);

    let widget = Paragraph::new(text).alignment(Alignment::Center).block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(Style::default().fg(color))
            .style(Style::default().bg(theme.background())),
    );

    f.render_widget(widget, area);
}

/// Renders a confirmation dialog
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `title` - Dialog title
/// * `message` - Message to display
/// * `confirm_selected` - Whether the confirm button is selected
/// * `theme` - Theme for colors
#[allow(dead_code)]
pub fn render_confirmation_dialog(
    f: &mut Frame,
    title: &str,
    message: &str,
    confirm_selected: bool,
    theme: &Theme,
) {
    let screen = f.area();
    let width = 60.min(screen.width.saturating_sub(4));
    let height = 10;

    let area = Rect {
        x: screen.width.saturating_sub(width) / 2,
        y: screen.height.saturating_sub(height) / 2,
        width,
        height,
    };

    f.render_widget(Clear, area);

    // Split area for message and buttons
    let _chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(1), Constraint::Length(3)])
        .split(area);

    // Render message
    let msg_widget = Paragraph::new(message)
        .style(Style::default().fg(theme.foreground()))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: false })
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.warning()))
                .style(Style::default().bg(theme.background())),
        );

    f.render_widget(msg_widget, area);

    // Render buttons
    let button_area = Rect {
        x: area.x,
        y: area.y + area.height.saturating_sub(4),
        width: area.width,
        height: 3,
    };

    let button_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
        .split(button_area);

    // Yes button
    let yes_style = if confirm_selected {
        Style::default()
            .fg(theme.highlight_fg())
            .bg(theme.success())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.success())
    };

    let yes_button = Paragraph::new("[ Yes ]")
        .alignment(Alignment::Center)
        .style(yes_style);
    f.render_widget(yes_button, button_chunks[0]);

    // No button
    let no_style = if !confirm_selected {
        Style::default()
            .fg(theme.highlight_fg())
            .bg(theme.error())
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(theme.error())
    };

    let no_button = Paragraph::new("[ No ]")
        .alignment(Alignment::Center)
        .style(no_style);
    f.render_widget(no_button, button_chunks[1]);
}

/// Renders a text input field with a blinking cursor
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - Area to render the input field
/// * `label` - Label for the input field
/// * `value` - Current input value
/// * `show_cursor` - Whether to show the cursor (for blinking effect)
/// * `theme` - Theme for colors
#[allow(dead_code)]
pub fn render_input_field(
    f: &mut Frame,
    area: Rect,
    label: &str,
    value: &str,
    show_cursor: bool,
    theme: &Theme,
) {
    let display_value = if value.is_empty() {
        if show_cursor {
            "_".to_string()
        } else {
            " ".to_string()
        }
    } else if show_cursor {
        format!("{value}_")
    } else {
        value.to_string()
    };

    let widget = Paragraph::new(display_value)
        .style(Style::default().fg(theme.foreground()))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title(label)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(theme.primary()))
                .style(Style::default().bg(theme.background())),
        );

    f.render_widget(widget, area);
}

/// Renders a dropdown/select menu
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - Area to render the dropdown
/// * `title` - Dropdown title
/// * `items` - List of items to display
/// * `selected_index` - Currently selected item index
/// * `current_value` - Current value (to mark with indicator)
/// * `theme` - Theme for colors
#[allow(dead_code)]
pub fn render_dropdown<T: AsRef<str>>(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: &[T],
    selected_index: usize,
    current_value: Option<&str>,
    theme: &Theme,
) {
    f.render_widget(Clear, area);

    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let item_str = item.as_ref();
            let mut style = if i == selected_index {
                Style::default()
                    .fg(theme.highlight_fg())
                    .bg(theme.highlight_bg())
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(theme.foreground())
            };

            // Mark current value
            let display_text = if Some(item_str) == current_value {
                if i != selected_index {
                    style = style.fg(theme.success());
                }
                format!("● {item_str}")
            } else {
                format!("  {item_str}")
            };

            ListItem::new(display_text).style(style)
        })
        .collect();

    let widget = List::new(list_items).block(
        Block::default()
            .title(format!("{title} (↑↓:navigate | Enter:select | Esc:cancel)"))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary()))
            .style(Style::default().bg(theme.background())),
    );

    f.render_widget(widget, area);
}

/// Multi-select checkbox item state (for future use)
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct CheckboxItem {
    pub label: String,
    pub checked: bool,
}

/// Renders a multi-select checkbox list
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - Area to render the checkboxes
/// * `title` - Title for the checkbox list
/// * `items` - List of checkbox items
/// * `selected_index` - Currently selected item index
/// * `theme` - Theme for colors
#[allow(dead_code)]
pub fn render_checkbox_list(
    f: &mut Frame,
    area: Rect,
    title: &str,
    items: &[CheckboxItem],
    selected_index: usize,
    theme: &Theme,
) {
    let list_items: Vec<ListItem> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            let checkbox = if item.checked { "[✓]" } else { "[ ]" };
            let style = if i == selected_index {
                Style::default()
                    .fg(theme.highlight_fg())
                    .bg(theme.highlight_bg())
                    .add_modifier(Modifier::BOLD)
            } else if item.checked {
                Style::default().fg(theme.success())
            } else {
                Style::default().fg(theme.foreground())
            };

            ListItem::new(format!("{checkbox} {}", item.label)).style(style)
        })
        .collect();

    let widget = List::new(list_items).block(
        Block::default()
            .title(format!(
                "{title} (↑↓:navigate | Space:toggle | Enter:confirm)"
            ))
            .borders(Borders::ALL)
            .border_style(Style::default().fg(theme.primary()))
            .style(Style::default().bg(theme.background())),
    );

    f.render_widget(widget, area);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spinner_frame_cycles() {
        // Test that spinner returns valid frames
        for _ in 0..20 {
            let frame = get_spinner_frame();
            assert!(!frame.is_empty());
            assert!(frame.len() <= 3); // Braille patterns are 1-3 bytes in UTF-8
        }
    }

    #[test]
    fn test_toast_type_equality() {
        assert_eq!(ToastType::Success, ToastType::Success);
        assert_ne!(ToastType::Success, ToastType::Error);
        assert_eq!(ToastType::Warning, ToastType::Warning);
    }

    #[test]
    fn test_checkbox_item_creation() {
        let item = CheckboxItem {
            label: "Test".to_string(),
            checked: true,
        };
        assert_eq!(item.label, "Test");
        assert!(item.checked);
    }

    #[test]
    fn test_checkbox_item_clone() {
        let item = CheckboxItem {
            label: "Test".to_string(),
            checked: false,
        };
        let cloned = item.clone();
        assert_eq!(item.label, cloned.label);
        assert_eq!(item.checked, cloned.checked);
    }
}
