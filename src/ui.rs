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

fn draw_custom_header(f: &mut Frame, area: Rect, title: &str) {
    let header = Paragraph::new(title)
        .style(Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD))
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

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
