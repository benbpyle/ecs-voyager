//! UI module for ECS Voyager.
//!
//! This module contains all UI-related functionality including:
//! - Theme system with customizable colors
//! - Reusable UI widgets (spinners, progress bars, dialogs, etc.)
//! - Text processing utilities (truncation, wrapping, line numbers)
//! - Layout helpers for responsive design
//! - Main rendering functions for all views

pub mod render;
pub mod theme;
pub mod utils;
pub mod widgets;

// Re-export commonly used items
pub use render::draw;
pub use theme::{Theme, ThemeColors, ThemePreset};
pub use utils::{
    add_line_numbers, centered_rect, responsive_column_widths, split_pane_layout,
    three_column_layout, truncate_middle, truncate_text, validate_terminal_size, wrap_text,
    MIN_TERMINAL_HEIGHT, MIN_TERMINAL_WIDTH,
};
pub use widgets::{
    get_spinner_frame, render_checkbox_list, render_confirmation_dialog, render_dropdown,
    render_input_field, render_progress_bar, render_spinner, render_toast, CheckboxItem,
    ToastType,
};
