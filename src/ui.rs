//! Terminal user interface rendering module.
//!
//! This module contains all UI rendering logic using the ratatui framework.
//! It defines functions for drawing different views (clusters, services, tasks, logs)
//! and UI components (header, footer, overlays).

use ratatui::{
    layout::{Constraint, Direction, Layout, Rect, Alignment},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Row, Table, Wrap, Clear},
    Frame,
};
use std::time::SystemTime;
use chrono::{DateTime, Local};

use crate::app::{App, AppState};

/// Main rendering function that draws the entire UI.
///
/// Divides the terminal into three sections (header, content, footer) and delegates
/// rendering to specialized functions based on the current application state.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `app` - The application state containing data to display
pub fn draw(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),  // Header
            Constraint::Min(0),      // Content
            Constraint::Length(3),  // Footer
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);

    if app.show_help {
        draw_help(f, chunks[1]);
    } else {
        match app.state {
            AppState::Clusters => draw_clusters(f, chunks[1], app),
            AppState::Services => draw_services(f, chunks[1], app),
            AppState::Tasks => draw_tasks(f, chunks[1], app),
            AppState::Details => draw_details(f, chunks[1], app),
            AppState::Logs => draw_logs(f, chunks[1], app),
        }
    }

    draw_footer(f, chunks[2], app);

    // Draw search input if in search mode
    if app.search_mode {
        draw_search_input(f, app);
    }

    // Draw loading indicator overlay if loading
    if app.loading {
        draw_loading_overlay(f, app);
    }
}

/// Renders the header section showing the current view and context.
///
/// Displays a title that changes based on the current state, including selected
/// cluster, service, and task information when applicable.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the header
/// * `app` - The application state
fn draw_header(f: &mut Frame, area: Rect, app: &App) {
    let title = match app.state {
        AppState::Clusters => "ECS Voyager - Clusters",
        AppState::Services => {
            if let Some(cluster) = &app.selected_cluster {
                return draw_custom_header(f, area, &format!("ECS Voyager - Services ({})", cluster));
            }
            "ECS Voyager - Services"
        }
        AppState::Tasks => {
            if let (Some(cluster), Some(service)) = (&app.selected_cluster, &app.selected_service) {
                return draw_custom_header(f, area, &format!("ECS Voyager - Tasks ({}/{})", cluster, service));
            }
            "ECS Voyager - Tasks"
        }
        AppState::Details => "ECS Voyager - Details",
        AppState::Logs => {
            if let Some(task) = &app.selected_task {
                return draw_custom_header(f, area, &format!("ECS Voyager - Logs (Task: {})", task.task_id));
            }
            "ECS Voyager - Logs"
        }
    };

    draw_custom_header(f, area, title);
}

/// Renders a header with a custom title string.
///
/// Helper function that creates a styled paragraph for the header.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the header
/// * `title` - The title text to display
fn draw_custom_header(f: &mut Frame, area: Rect, title: &str) {
    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

/// Renders the footer section with keybindings and status information.
///
/// Shows available keyboard shortcuts and the current status message. When loading,
/// displays a spinner animation. If search is active, shows the search query.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the footer
/// * `app` - The application state
fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let footer_text = if app.show_help {
        vec![
            Line::from(vec![
                Span::styled("Press ", Style::default().fg(Color::Gray)),
                Span::styled("?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(" to close help", Style::default().fg(Color::Gray)),
            ])
        ]
    } else {
        let status_color = if app.loading { Color::Yellow } else { Color::Green };
        let status_text = if app.loading {
            format!("{} {}", get_spinner(), app.status_message)
        } else {
            app.status_message.clone()
        };

        vec![
            Line::from(vec![
                Span::styled("q", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":quit ", Style::default().fg(Color::Gray)),
                Span::styled("?", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":help ", Style::default().fg(Color::Gray)),
                Span::styled("r", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":refresh ", Style::default().fg(Color::Gray)),
                Span::styled("1-3", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":switch view ", Style::default().fg(Color::Gray)),
                Span::styled("↑↓/jk", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":navigate ", Style::default().fg(Color::Gray)),
                Span::styled("Enter", Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                Span::styled(":select ", Style::default().fg(Color::Gray)),
            ]),
            Line::from(vec![
                Span::styled("Status: ", Style::default().fg(Color::Gray)),
                Span::styled(status_text, Style::default().fg(status_color)),
                if app.search_mode || !app.search_query.is_empty() {
                    Span::styled(
                        format!(" | Search: {}", if app.search_query.is_empty() { "_" } else { &app.search_query }),
                        Style::default().fg(Color::Yellow)
                    )
                } else {
                    Span::raw("")
                },
            ])
        ]
    };

    let footer = Paragraph::new(footer_text)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, area);
}

/// Renders the clusters list view.
///
/// Displays all ECS clusters (filtered by search query if active) as a vertical list.
/// The currently selected cluster is highlighted. Shows count of filtered vs total clusters.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the clusters list
/// * `app` - The application state containing cluster data
fn draw_clusters(f: &mut Frame, area: Rect, app: &App) {
    let filtered_clusters = app.get_filtered_clusters();

    let items: Vec<ListItem> = filtered_clusters
        .iter()
        .enumerate()
        .map(|(i, cluster)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(cluster.as_str()).style(style)
        })
        .collect();

    let title = if app.search_query.is_empty() {
        format!("Clusters ({}) - /:search | Enter:select", filtered_clusters.len())
    } else {
        format!("Clusters ({}/{}) - Esc:clear | Enter:select", filtered_clusters.len(), app.clusters.len())
    };

    let list = List::new(items)
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL),
        );

    f.render_widget(list, area);
}

/// Renders the services table view.
///
/// Displays services for the selected cluster in a table format with columns for name,
/// status, desired/running/pending counts, and launch type. The currently selected
/// service row is highlighted. Shows available actions in the title.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the services table
/// * `app` - The application state containing service data
fn draw_services(f: &mut Frame, area: Rect, app: &App) {
    let filtered_services = app.get_filtered_services();

    let header = Row::new(vec!["Name", "Status", "Desired", "Running", "Pending", "Launch Type"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = filtered_services
        .iter()
        .enumerate()
        .map(|(i, service)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                service.name.clone(),
                service.status.clone(),
                service.desired_count.to_string(),
                service.running_count.to_string(),
                service.pending_count.to_string(),
                service.launch_type.clone(),
            ])
            .style(style)
        })
        .collect();

    let title = if app.search_query.is_empty() {
        format!("Services ({}) - /:search | Enter:tasks | d:describe | x:restart", filtered_services.len())
    } else {
        format!("Services ({}/{}) - Esc:clear | Enter:tasks | d:describe | x:restart", filtered_services.len(), app.services.len())
    };

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(15),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(10),
            Constraint::Percentage(25),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(title)
            .borders(Borders::ALL),
    );

    f.render_widget(table, area);
}

/// Renders the tasks table view.
///
/// Displays tasks for the selected service in a table format with columns for task ID,
/// status, desired status, container instance, CPU, and memory. The currently selected
/// task row is highlighted. Shows available actions in the title.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the tasks table
/// * `app` - The application state containing task data
fn draw_tasks(f: &mut Frame, area: Rect, app: &App) {
    let filtered_tasks = app.get_filtered_tasks();

    let header = Row::new(vec!["Task ID", "Status", "Desired", "Instance", "CPU", "Memory"])
        .style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD))
        .bottom_margin(1);

    let rows: Vec<Row> = filtered_tasks
        .iter()
        .enumerate()
        .map(|(i, task)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            Row::new(vec![
                task.task_id.clone(),
                task.status.clone(),
                task.desired_status.clone(),
                task.container_instance.clone(),
                task.cpu.clone(),
                task.memory.clone(),
            ])
            .style(style)
        })
        .collect();

    let title = if app.search_query.is_empty() {
        format!("Tasks ({}) - /:search | l:logs | d:describe | x:stop", filtered_tasks.len())
    } else {
        format!("Tasks ({}/{}) - Esc:clear | l:logs | d:describe | x:stop", filtered_tasks.len(), app.tasks.len())
    };

    let table = Table::new(
        rows,
        [
            Constraint::Percentage(25),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
            Constraint::Percentage(20),
            Constraint::Percentage(12),
            Constraint::Percentage(13),
        ],
    )
    .header(header)
    .block(
        Block::default()
            .title(title)
            .borders(Borders::ALL),
    );

    f.render_widget(table, area);
}

/// Renders the details view showing comprehensive information about a resource.
///
/// Displays detailed information about a selected service or task in a scrollable
/// text view. The content is formatted as multi-line text with word wrapping.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the details view
/// * `app` - The application state containing details text
fn draw_details(f: &mut Frame, area: Rect, app: &App) {
    let default_text = "No details available".to_string();
    let content = app.details.as_ref().unwrap_or(&default_text);

    let paragraph = Paragraph::new(content.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title("Details (Press Esc or h to go back)")
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(paragraph, area);
}

/// Renders the logs view showing CloudWatch Logs for a task.
///
/// Displays log entries with timestamps, container names, and messages. Supports scrolling
/// and auto-tail mode. Shows a scroll indicator with current position. Each log line is
/// color-coded: timestamps in dark gray, container names in cyan, messages in white.
///
/// When no logs are available, displays a helpful message explaining possible reasons.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the logs view
/// * `app` - The application state containing log entries
fn draw_logs(f: &mut Frame, area: Rect, app: &App) {
    if app.logs.is_empty() {
        let no_logs = Paragraph::new("No logs available for this task.\n\nThis could mean:\n- The task has no CloudWatch Logs configured\n- The log stream hasn't been created yet\n- The task hasn't produced any logs")
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .title("Logs (Press Esc or h to go back | r:refresh)")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(no_logs, area);
        return;
    }

    // Calculate visible window
    let available_height = area.height.saturating_sub(2) as usize; // Account for borders
    let total_logs = app.logs.len();

    // Determine which logs to show based on scroll position
    let start_idx = if app.auto_tail {
        total_logs.saturating_sub(available_height)
    } else {
        app.log_scroll.min(total_logs.saturating_sub(available_height.min(total_logs)))
    };

    let end_idx = (start_idx + available_height).min(total_logs);

    // Format log entries as Lines
    let log_lines: Vec<Line> = app.logs[start_idx..end_idx]
        .iter()
        .map(|log| {
            // Convert timestamp (milliseconds) to datetime
            let datetime = DateTime::from_timestamp_millis(log.timestamp)
                .map(|dt| dt.with_timezone(&Local))
                .unwrap_or_else(|| Local::now());

            let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S%.3f").to_string();

            Line::from(vec![
                Span::styled(
                    format!("[{}] ", timestamp_str),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("[{}] ", log.container_name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(
                    &log.message,
                    Style::default().fg(Color::White),
                ),
            ])
        })
        .collect();

    let scroll_indicator = if total_logs > available_height {
        format!(" [{}-{}/{}]", start_idx + 1, end_idx, total_logs)
    } else {
        format!(" [{}]", total_logs)
    };

    let title = if app.auto_tail {
        format!("Logs{} (AUTO-TAIL | t:toggle | Esc/h:back | r:refresh)", scroll_indicator)
    } else {
        format!("Logs{} (↑↓:scroll | t:toggle tail | Esc/h:back | r:refresh)", scroll_indicator)
    };

    let logs_widget = Paragraph::new(log_lines)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.auto_tail { Color::Green } else { Color::White })),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(logs_widget, area);
}

/// Renders the help overlay showing all keyboard shortcuts.
///
/// Displays a comprehensive list of keybindings organized by category:
/// Navigation, Views, Actions, and General. Each keybinding is shown with
/// a description of its function.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the help screen
fn draw_help(f: &mut Frame, area: Rect) {
    let help_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled("Navigation", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  ↑/k         ", Style::default().fg(Color::Yellow)),
            Span::raw("Move up"),
        ]),
        Line::from(vec![
            Span::styled("  ↓/j         ", Style::default().fg(Color::Yellow)),
            Span::raw("Move down"),
        ]),
        Line::from(vec![
            Span::styled("  Enter       ", Style::default().fg(Color::Yellow)),
            Span::raw("Select/Drill down"),
        ]),
        Line::from(vec![
            Span::styled("  Esc/h       ", Style::default().fg(Color::Yellow)),
            Span::raw("Go back"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Views", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  1           ", Style::default().fg(Color::Yellow)),
            Span::raw("Clusters view"),
        ]),
        Line::from(vec![
            Span::styled("  2           ", Style::default().fg(Color::Yellow)),
            Span::raw("Services view"),
        ]),
        Line::from(vec![
            Span::styled("  3           ", Style::default().fg(Color::Yellow)),
            Span::raw("Tasks view"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("Actions", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  r           ", Style::default().fg(Color::Yellow)),
            Span::raw("Refresh current view"),
        ]),
        Line::from(vec![
            Span::styled("  d           ", Style::default().fg(Color::Yellow)),
            Span::raw("Describe selected item"),
        ]),
        Line::from(vec![
            Span::styled("  l           ", Style::default().fg(Color::Yellow)),
            Span::raw("View logs (from Tasks view)"),
        ]),
        Line::from(vec![
            Span::styled("  t           ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle auto-tail (in Logs view)"),
        ]),
        Line::from(vec![
            Span::styled("  /           ", Style::default().fg(Color::Yellow)),
            Span::raw("Search/Filter current view"),
        ]),
        Line::from(vec![
            Span::styled("  x           ", Style::default().fg(Color::Yellow)),
            Span::raw("Execute action (restart service/stop task)"),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled("General", Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD)),
        ]),
        Line::from(vec![
            Span::styled("  ?           ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  q           ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
    ];

    let help = Paragraph::new(help_text)
        .block(
            Block::default()
                .title("Help")
                .borders(Borders::ALL),
        );

    f.render_widget(help, area);
}

/// Renders a centered loading overlay with a spinner and status message.
///
/// Displays a modal dialog box in the center of the screen with an animated spinner
/// and the current loading status message. Used to indicate background operations.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `app` - The application state containing the status message
fn draw_loading_overlay(f: &mut Frame, app: &App) {
    // Create a centered overlay
    let area = f.area();
    let width = 50.min(area.width.saturating_sub(4));
    let height = 7;

    let overlay_area = Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    // Clear the area behind the overlay
    f.render_widget(Clear, overlay_area);

    let spinner = get_spinner();
    let loading_text = vec![
        Line::from(""),
        Line::from(vec![
            Span::styled(
                format!("  {}  ", spinner),
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::styled(
                &app.status_message,
                Style::default().fg(Color::Yellow),
            ),
        ]),
        Line::from(""),
    ];

    let loading_block = Paragraph::new(loading_text)
        .alignment(Alignment::Center)
        .block(
            Block::default()
                .title("Loading")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Cyan))
                .style(Style::default().bg(Color::Black)),
        );

    f.render_widget(loading_block, overlay_area);
}

/// Renders the search input overlay.
///
/// Displays a modal dialog box near the bottom of the screen where users can type
/// their search query. Shows the current query text or an underscore cursor when empty.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `app` - The application state containing the search query
fn draw_search_input(f: &mut Frame, app: &App) {
    let area = f.area();
    let width = 60.min(area.width.saturating_sub(4));
    let height = 3;

    let search_area = Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(10),
        width,
        height,
    };

    f.render_widget(Clear, search_area);

    let search_text = if app.search_query.is_empty() {
        "_".to_string()
    } else {
        app.search_query.clone()
    };

    let search_widget = Paragraph::new(search_text)
        .style(Style::default().fg(Color::Yellow))
        .alignment(Alignment::Left)
        .block(
            Block::default()
                .title("Search (Esc to cancel)")
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Yellow))
                .style(Style::default().bg(Color::Black)),
        );

    f.render_widget(search_widget, search_area);
}

/// Returns the current frame of a spinner animation based on system time.
///
/// Uses a 10-frame Braille pattern spinner that updates approximately every 80ms.
/// Provides visual feedback for loading states.
///
/// # Returns
/// A static string reference containing the current spinner frame character
fn get_spinner() -> &'static str {
    // Get current time in milliseconds and cycle through spinner frames
    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_millis();

    let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
    let index = ((now / 80) % frames.len() as u128) as usize;
    frames[index]
}
