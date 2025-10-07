//! Terminal user interface rendering module.
//!
//! This module contains all UI rendering logic using the ratatui framework.
//! It defines functions for drawing different views (clusters, services, tasks, logs)
//! and UI components (header, footer, overlays).

use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, Paragraph, Row, Table, Wrap},
    Frame,
};
use std::time::SystemTime;

use crate::app::{App, AppState, ModalState};
use crate::charts::{render_chart, ChartConfig, ChartDatapoint};

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
            Constraint::Length(3), // Header
            Constraint::Min(0),    // Content
            Constraint::Length(5), // Footer (multi-line)
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
            AppState::Metrics => draw_metrics(f, chunks[1], app),
        }
    }

    draw_footer(f, chunks[2], app);

    // Draw search input if in search mode
    if app.search_mode {
        draw_search_input(f, app);
    }

    // Draw modals
    match app.modal_state {
        ModalState::ProfileSelector => draw_profile_selector(f, app),
        ModalState::RegionSelector => draw_region_selector(f, app),
        ModalState::None => {}
    }

    // Draw loading indicator overlay if loading (rendered last so it's on top)
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
                return draw_custom_header(f, area, &format!("ECS Voyager - Services ({cluster})"));
            }
            "ECS Voyager - Services"
        }
        AppState::Tasks => {
            if let (Some(cluster), Some(service)) = (&app.selected_cluster, &app.selected_service) {
                return draw_custom_header(
                    f,
                    area,
                    &format!("ECS Voyager - Tasks ({cluster}/{service})"),
                );
            }
            "ECS Voyager - Tasks"
        }
        AppState::Details => "ECS Voyager - Details",
        AppState::Logs => {
            if let Some(task) = &app.selected_task {
                return draw_custom_header(
                    f,
                    area,
                    &format!("ECS Voyager - Logs (Task: {})", task.task_id),
                );
            }
            "ECS Voyager - Logs"
        }
        AppState::Metrics => {
            if let Some(service) = &app.selected_service {
                return draw_custom_header(
                    f,
                    area,
                    &format!("ECS Voyager - Metrics (Service: {service})"),
                );
            }
            "ECS Voyager - Metrics"
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
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

/// Renders the footer section with keybindings and status information.
///
/// Shows a multi-line status bar with:
/// - Line 1: Keybindings organized by category
/// - Line 2: AWS context (region, profile) and status
/// - Line 3: Last refresh time and item counts
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the footer
/// * `app` - The application state
fn draw_footer(f: &mut Frame, area: Rect, app: &App) {
    let footer_text = if app.show_help {
        vec![Line::from(vec![
            Span::styled("Press ", Style::default().fg(Color::Gray)),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" to close help", Style::default().fg(Color::Gray)),
        ])]
    } else {
        // Line 1: Keybindings
        let line1 = Line::from(vec![
            Span::styled("[", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "q",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":quit ", Style::default().fg(Color::Gray)),
            Span::styled(
                "?",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":help ", Style::default().fg(Color::Gray)),
            Span::styled(
                "r",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":refresh ", Style::default().fg(Color::Gray)),
            Span::styled(
                "P",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":profile ", Style::default().fg(Color::Gray)),
            Span::styled(
                "R",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":region", Style::default().fg(Color::Gray)),
            Span::styled("] ", Style::default().fg(Color::DarkGray)),
            Span::styled("• ", Style::default().fg(Color::DarkGray)),
            Span::styled("[", Style::default().fg(Color::DarkGray)),
            Span::styled(
                "1-3",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":views ", Style::default().fg(Color::Gray)),
            Span::styled(
                "↑↓/jk",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":nav ", Style::default().fg(Color::Gray)),
            Span::styled(
                "Enter",
                Style::default()
                    .fg(Color::Yellow)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(":select", Style::default().fg(Color::Gray)),
            Span::styled("]", Style::default().fg(Color::DarkGray)),
        ]);

        // Line 2: AWS context and status
        let connection_indicator = if app.loading { "○" } else { "●" };
        let connection_color = if app.loading {
            Color::Yellow
        } else {
            Color::Green
        };
        let status_text = if app.loading {
            format!("{} {}", get_spinner(), app.status_message)
        } else {
            app.status_message.clone()
        };

        let item_count = match app.state {
            AppState::Clusters => format!("{} clusters", app.clusters.len()),
            AppState::Services => format!("{} services", app.services.len()),
            AppState::Tasks => format!("{} tasks", app.tasks.len()),
            AppState::Logs => format!("{} logs", app.logs.len()),
            AppState::Details => "details".to_string(),
            AppState::Metrics => "metrics".to_string(),
        };

        let line2 = Line::from(vec![
            Span::styled("Region: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &app.current_region,
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("Profile: ", Style::default().fg(Color::Gray)),
            Span::styled(
                &app.current_profile,
                Style::default()
                    .fg(Color::Magenta)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled(connection_indicator, Style::default().fg(connection_color)),
            Span::styled(" ", Style::default()),
            Span::styled(status_text, Style::default().fg(connection_color)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled(item_count, Style::default().fg(Color::White)),
        ]);

        // Line 3: Refresh info
        let elapsed = app.last_refresh.elapsed().as_secs();
        let refresh_text = if elapsed < 60 {
            format!("{elapsed}s ago")
        } else {
            let mins = elapsed / 60;
            let secs = elapsed % 60;
            format!("{mins}m {secs}s ago")
        };

        let refresh_interval = app.config.behavior.refresh_interval;
        let auto_refresh_status = if app.config.behavior.auto_refresh {
            format!("ON ({refresh_interval}s)")
        } else {
            "OFF".to_string()
        };

        let line3 = Line::from(vec![
            Span::styled("Last refresh: ", Style::default().fg(Color::Gray)),
            Span::styled(refresh_text, Style::default().fg(Color::White)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("Auto-refresh: ", Style::default().fg(Color::Gray)),
            Span::styled(auto_refresh_status, Style::default().fg(Color::White)),
            if app.search_mode || !app.search_query.is_empty() {
                Span::styled(
                    format!(
                        " | Search: {}",
                        if app.search_query.is_empty() {
                            "_"
                        } else {
                            &app.search_query
                        }
                    ),
                    Style::default().fg(Color::Yellow),
                )
            } else {
                Span::raw("")
            },
        ]);

        vec![line1, line2, line3]
    };

    let footer = Paragraph::new(footer_text).block(Block::default().borders(Borders::ALL));
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
        format!(
            "Clusters ({}) - /:search | Enter:select",
            filtered_clusters.len()
        )
    } else {
        format!(
            "Clusters ({}/{}) - Esc:clear | Enter:select",
            filtered_clusters.len(),
            app.clusters.len()
        )
    };

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));

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

    let header = Row::new(vec![
        "Name",
        "Status",
        "Desired",
        "Running",
        "Pending",
        "Launch Type",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
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
        format!(
            "Services ({}) - /:search | Enter:tasks | d:describe | x:restart",
            filtered_services.len()
        )
    } else {
        format!(
            "Services ({}/{}) - Esc:clear | Enter:tasks | d:describe | x:restart",
            filtered_services.len(),
            app.services.len()
        )
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
    .block(Block::default().title(title).borders(Borders::ALL));

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

    let header = Row::new(vec![
        "Task ID", "Status", "Desired", "Instance", "CPU", "Memory",
    ])
    .style(
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )
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
        format!(
            "Tasks ({}) - /:search | l:logs | d:describe | x:stop",
            filtered_tasks.len()
        )
    } else {
        format!(
            "Tasks ({}/{}) - Esc:clear | l:logs | d:describe | x:stop",
            filtered_tasks.len(),
            app.tasks.len()
        )
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
    .block(Block::default().title(title).borders(Borders::ALL));

    f.render_widget(table, area);
}

/// Renders the details view showing comprehensive information about a resource.
///
/// Displays detailed information about a selected service or task in a scrollable
/// text view. The content is formatted as multi-line text with word wrapping.
/// Supports scrolling with up/down arrow keys or j/k for long content.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the details view
/// * `app` - The application state containing details text and scroll position
fn draw_details(f: &mut Frame, area: Rect, app: &App) {
    let default_text = "No details available".to_string();

    // Choose between formatted view and JSON view
    let (content, view_type) = if app.show_json_view {
        (app.details_json.as_ref().unwrap_or(&default_text), "JSON")
    } else {
        (app.details.as_ref().unwrap_or(&default_text), "Formatted")
    };

    let title = format!("Details - {} View (↑↓:scroll | J:toggle | Esc/h:back)", view_type);

    let paragraph = Paragraph::new(content.as_str())
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.details_scroll as u16, 0));

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
    // Get filtered logs
    let filtered_logs = app.get_filtered_logs();

    if filtered_logs.is_empty() {
        let no_logs = if !app.log_search_query.is_empty() || app.log_level_filter.is_some() {
            Paragraph::new("No logs match the current search/filter criteria.\n\nTry:\n- Press Esc to clear search\n- Press 'f' to cycle through log level filters")
        } else {
            Paragraph::new("No logs available for this task.\n\nThis could mean:\n- The task has no CloudWatch Logs configured\n- The log stream hasn't been created yet\n- The task hasn't produced any logs")
        }
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
    let total_logs = filtered_logs.len();

    // Determine which logs to show based on scroll position
    let start_idx = if app.auto_tail {
        total_logs.saturating_sub(available_height)
    } else {
        app.log_scroll
            .min(total_logs.saturating_sub(available_height.min(total_logs)))
    };

    let end_idx = (start_idx + available_height).min(total_logs);

    // Format log entries as Lines
    let log_lines: Vec<Line> = filtered_logs[start_idx..end_idx]
        .iter()
        .map(|log| {
            // Convert timestamp (milliseconds) to datetime
            let datetime = DateTime::from_timestamp_millis(log.timestamp)
                .map(|dt| dt.with_timezone(&Local))
                .unwrap_or_else(Local::now);

            let timestamp_str = datetime.format("%Y-%m-%d %H:%M:%S%.3f").to_string();

            Line::from(vec![
                Span::styled(
                    format!("[{timestamp_str}] "),
                    Style::default().fg(Color::DarkGray),
                ),
                Span::styled(
                    format!("[{}] ", log.container_name),
                    Style::default().fg(Color::Cyan),
                ),
                Span::styled(&log.message, Style::default().fg(Color::White)),
            ])
        })
        .collect();

    let scroll_indicator = if total_logs > available_height {
        let start = start_idx + 1;
        format!(" [{start}-{end_idx}/{total_logs}]")
    } else {
        format!(" [{total_logs}]")
    };

    // Build filter/search status
    let mut filter_status = String::new();
    if let Some(ref level) = app.log_level_filter {
        filter_status.push_str(&format!(" | Filter: {level:?}"));
    }
    if !app.log_search_query.is_empty() {
        filter_status.push_str(&format!(" | Search: '{}'", app.log_search_query));
    }

    let title = if app.auto_tail {
        format!("Logs{scroll_indicator}{filter_status} (AUTO-TAIL | /:search f:filter e:export t:toggle)")
    } else {
        format!("Logs{scroll_indicator}{filter_status} (↑↓:scroll | /:search f:filter e:export t:toggle)")
    };

    let logs_widget = Paragraph::new(log_lines)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(title)
                .borders(Borders::ALL)
                .border_style(Style::default().fg(if app.auto_tail {
                    Color::Green
                } else {
                    Color::White
                })),
        )
        .wrap(Wrap { trim: false });

    f.render_widget(logs_widget, area);
}

/// Renders the metrics view showing CloudWatch metrics for a service.
///
/// Displays CPU and Memory utilization metrics with ASCII charts, statistics,
/// and CloudWatch alarm status.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the metrics view
/// * `app` - The application state containing metrics data
fn draw_metrics(f: &mut Frame, area: Rect, app: &App) {
    if app.metrics.is_none() {
        let no_metrics = Paragraph::new("No metrics available for this service.\n\nThis could mean:\n- The service has no CloudWatch metrics enabled\n- The service hasn't been running long enough to generate metrics\n- There was an error fetching metrics\n\nPress 'm' from Services view to load metrics")
            .style(Style::default().fg(Color::Yellow))
            .block(
                Block::default()
                    .title("Metrics (Press Esc or h to go back)")
                    .borders(Borders::ALL),
            )
            .wrap(Wrap { trim: false });
        f.render_widget(no_metrics, area);
        return;
    }

    let metrics = app.metrics.as_ref().unwrap();

    // Split area into alarms section and charts section
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(if metrics.alarms.is_empty() { 0 } else { 8 }),
            Constraint::Min(0),
        ])
        .split(area);

    // Draw alarms section if alarms exist and config allows
    if !metrics.alarms.is_empty() && app.config.metrics.show_alarms {
        draw_alarms_section(f, chunks[0], metrics);
    }

    // Use appropriate chunk for metrics content
    let metrics_area = if metrics.alarms.is_empty() || !app.config.metrics.show_alarms {
        area
    } else {
        chunks[1]
    };

    let mut content_lines: Vec<Line> = vec![];

    // Service and time range info
    content_lines.push(Line::from(vec![
        Span::styled("Service: ", Style::default().fg(Color::Gray)),
        Span::styled(
            &metrics.service_name,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::styled(" | Cluster: ", Style::default().fg(Color::Gray)),
        Span::styled(
            &metrics.cluster_name,
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
    ]));
    content_lines.push(Line::from(""));

    // CPU Metrics Section
    if app.config.metrics.show_charts && !metrics.cpu_datapoints.is_empty() {
        // Render CPU chart
        let cpu_chart_datapoints: Vec<ChartDatapoint> = metrics
            .cpu_datapoints
            .iter()
            .filter_map(|dp| dp.average.map(|avg| ChartDatapoint {
                timestamp: dp.timestamp,
                value: avg,
            }))
            .collect();

        let chart_config = ChartConfig {
            width: 60,
            height: 10,
            min_value: None,  // Auto-scale based on data
            max_value: None,  // Auto-scale based on data
            line_color: Color::Green,
            show_y_labels: true,
        };

        let chart_lines = render_chart(&cpu_chart_datapoints, &chart_config, "CPU Utilization (%)");
        content_lines.extend(chart_lines);
        content_lines.push(Line::from(""));
    } else {
        // Fallback to text-based metrics
        content_lines.push(Line::from(vec![Span::styled(
            "  CPU Utilization",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        content_lines.push(Line::from(""));
    }

    // CPU Statistics
    if !metrics.cpu_datapoints.is_empty() {
        let avg_cpu: f64 = metrics
            .cpu_datapoints
            .iter()
            .filter_map(|dp| dp.average)
            .sum::<f64>()
            / metrics.cpu_datapoints.iter().filter(|dp| dp.average.is_some()).count() as f64;
        let max_cpu = metrics
            .cpu_datapoints
            .iter()
            .filter_map(|dp| dp.maximum)
            .fold(0.0f64, |a, b| a.max(b));

        content_lines.push(Line::from(vec![
            Span::styled("  Average: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{avg_cpu:.2}%"), Style::default().fg(Color::Green)),
            Span::styled("  |  Maximum: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{max_cpu:.2}%"), Style::default().fg(Color::Yellow)),
            Span::styled("  |  Data points: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", metrics.cpu_datapoints.len()),
                Style::default().fg(Color::White),
            ),
        ]));
    } else {
        content_lines.push(Line::from(Span::styled(
            "  No CPU data available",
            Style::default().fg(Color::Yellow),
        )));
    }

    content_lines.push(Line::from(""));
    content_lines.push(Line::from(""));

    // Memory Metrics Section
    if app.config.metrics.show_charts && !metrics.memory_datapoints.is_empty() {
        // Render Memory chart
        let mem_chart_datapoints: Vec<ChartDatapoint> = metrics
            .memory_datapoints
            .iter()
            .filter_map(|dp| dp.average.map(|avg| ChartDatapoint {
                timestamp: dp.timestamp,
                value: avg,
            }))
            .collect();

        if !mem_chart_datapoints.is_empty() {
            let chart_config = ChartConfig {
                width: 60,
                height: 10,
                min_value: None,  // Auto-scale based on data
                max_value: None,  // Auto-scale based on data
                line_color: Color::Cyan,
                show_y_labels: true,
            };

            let chart_lines = render_chart(&mem_chart_datapoints, &chart_config, "Memory Utilization (%)");
            content_lines.extend(chart_lines);
        } else {
            content_lines.push(Line::from(Span::styled(
                "  Memory Utilization",
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            )));
            content_lines.push(Line::from(Span::styled(
                "  [All memory datapoints have None for average value]",
                Style::default().fg(Color::Yellow),
            )));
        }
        content_lines.push(Line::from(""));
    } else {
        // Fallback to text-based metrics
        content_lines.push(Line::from(vec![Span::styled(
            "  Memory Utilization",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]));
        content_lines.push(Line::from(""));
    }

    // Memory Statistics
    if !metrics.memory_datapoints.is_empty() {
        let avg_mem: f64 = metrics
            .memory_datapoints
            .iter()
            .filter_map(|dp| dp.average)
            .sum::<f64>()
            / metrics.memory_datapoints.iter().filter(|dp| dp.average.is_some()).count() as f64;
        let max_mem = metrics
            .memory_datapoints
            .iter()
            .filter_map(|dp| dp.maximum)
            .fold(0.0f64, |a, b| a.max(b));

        content_lines.push(Line::from(vec![
            Span::styled("  Average: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{avg_mem:.2}%"), Style::default().fg(Color::Green)),
            Span::styled("  |  Maximum: ", Style::default().fg(Color::Gray)),
            Span::styled(format!("{max_mem:.2}%"), Style::default().fg(Color::Yellow)),
            Span::styled("  |  Data points: ", Style::default().fg(Color::Gray)),
            Span::styled(
                format!("{}", metrics.memory_datapoints.len()),
                Style::default().fg(Color::White),
            ),
        ]));
    } else {
        content_lines.push(Line::from(Span::styled(
            "  No Memory data available",
            Style::default().fg(Color::Yellow),
        )));
    }

    let time_range_label = metrics.time_range.label();

    let metrics_widget = Paragraph::new(content_lines)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .title(format!(
                    "Metrics [{}] (T:cycle range | r:refresh | Esc/h:back | ↑↓:scroll)",
                    time_range_label
                ))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false })
        .scroll((app.metrics_scroll as u16, 0));

    f.render_widget(metrics_widget, metrics_area);
}

/// Renders the CloudWatch alarms section.
///
/// Displays alarms associated with the service, showing alarm name, state,
/// and state reason in a compact format.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the alarms section
/// * `metrics` - The metrics data containing alarms
fn draw_alarms_section(f: &mut Frame, area: Rect, metrics: &crate::aws::Metrics) {
    let mut alarm_lines: Vec<Line> = vec![];

    alarm_lines.push(Line::from(vec![Span::styled(
        "CloudWatch Alarms",
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));
    alarm_lines.push(Line::from(""));

    for alarm in &metrics.alarms {
        let state_color = match alarm.state.as_str() {
            "OK" => Color::Green,
            "ALARM" => Color::Red,
            "INSUFFICIENT_DATA" => Color::Yellow,
            _ => Color::Gray,
        };

        let state_symbol = match alarm.state.as_str() {
            "OK" => "✓",
            "ALARM" => "✗",
            "INSUFFICIENT_DATA" => "?",
            _ => "•",
        };

        alarm_lines.push(Line::from(vec![
            Span::styled(format!("  {state_symbol} "), Style::default().fg(state_color)),
            Span::styled(&alarm.name, Style::default().fg(Color::White)),
            Span::styled(" [", Style::default().fg(Color::DarkGray)),
            Span::styled(&alarm.state, Style::default().fg(state_color)),
            Span::styled("]", Style::default().fg(Color::DarkGray)),
        ]));

        if let Some(reason) = &alarm.state_reason {
            alarm_lines.push(Line::from(vec![
                Span::styled("    ", Style::default()),
                Span::styled(reason, Style::default().fg(Color::DarkGray)),
            ]));
        }
    }

    let alarms_widget = Paragraph::new(alarm_lines)
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });

    f.render_widget(alarms_widget, area);
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
        Line::from(vec![Span::styled(
            "Navigation",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
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
        Line::from(vec![Span::styled(
            "Views",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
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
        Line::from(vec![Span::styled(
            "Actions",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  r           ", Style::default().fg(Color::Yellow)),
            Span::raw("Refresh current view"),
        ]),
        Line::from(vec![
            Span::styled("  P           ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch AWS profile"),
        ]),
        Line::from(vec![
            Span::styled("  R           ", Style::default().fg(Color::Yellow)),
            Span::raw("Switch AWS region"),
        ]),
        Line::from(vec![
            Span::styled("  d           ", Style::default().fg(Color::Yellow)),
            Span::raw("Describe selected item"),
        ]),
        Line::from(vec![
            Span::styled("  J           ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle JSON view (in Details view)"),
        ]),
        Line::from(vec![
            Span::styled("  l           ", Style::default().fg(Color::Yellow)),
            Span::raw("View logs (from Tasks view)"),
        ]),
        Line::from(vec![
            Span::styled("  m           ", Style::default().fg(Color::Yellow)),
            Span::raw("View metrics (from Services view)"),
        ]),
        Line::from(vec![
            Span::styled("  T           ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle time range (in Metrics view: 1h/6h/24h/7d)"),
        ]),
        Line::from(vec![
            Span::styled("  t           ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle auto-tail (in Logs view)"),
        ]),
        Line::from(vec![
            Span::styled("  /           ", Style::default().fg(Color::Yellow)),
            Span::raw("Search/Filter view | Search logs (in Logs view)"),
        ]),
        Line::from(vec![
            Span::styled("  f           ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle log level filter (in Logs view)"),
        ]),
        Line::from(vec![
            Span::styled("  e           ", Style::default().fg(Color::Yellow)),
            Span::raw("Export logs to file (in Logs view)"),
        ]),
        Line::from(vec![
            Span::styled("  x           ", Style::default().fg(Color::Yellow)),
            Span::raw("Execute action (restart service/stop task)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "General",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  ?           ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle this help"),
        ]),
        Line::from(vec![
            Span::styled("  q           ", Style::default().fg(Color::Yellow)),
            Span::raw("Quit"),
        ]),
    ];

    let help =
        Paragraph::new(help_text).block(Block::default().title("Help").borders(Borders::ALL));

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
        Line::from(vec![Span::styled(
            format!("  {spinner}  "),
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(""),
        Line::from(vec![Span::styled(
            &app.status_message,
            Style::default().fg(Color::Yellow),
        )]),
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

/// Renders the profile selector modal.
///
/// Displays a centered modal dialog with a list of available AWS profiles.
/// The currently selected profile is highlighted. Users can navigate with
/// arrow keys and select with Enter.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `app` - The application state containing available profiles
fn draw_profile_selector(f: &mut Frame, app: &App) {
    let area = f.area();
    let width = 60.min(area.width.saturating_sub(4));
    let height = (app.available_profiles.len() + 4).min(20) as u16;

    let modal_area = Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    // Clear the area behind the modal
    f.render_widget(Clear, modal_area);

    // Create list items
    let items: Vec<ListItem> = app
        .available_profiles
        .iter()
        .enumerate()
        .map(|(i, profile)| {
            let mut style = if i == app.modal_selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Mark current profile with indicator
            let display_text = if profile == &app.current_profile {
                format!("● {profile}")
            } else {
                format!("  {profile}")
            };

            if profile == &app.current_profile && i != app.modal_selected_index {
                style = style.fg(Color::Green);
            }

            ListItem::new(display_text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title("Select AWS Profile (↑↓:navigate | Enter:select | Esc:cancel)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    f.render_widget(list, modal_area);
}

/// Renders the region selector modal.
///
/// Displays a centered modal dialog with a list of common AWS regions.
/// The currently selected region is highlighted. Users can navigate with
/// arrow keys and select with Enter.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `app` - The application state containing available regions
fn draw_region_selector(f: &mut Frame, app: &App) {
    let area = f.area();
    let width = 60.min(area.width.saturating_sub(4));
    let height = (app.available_regions.len() + 4).min(20) as u16;

    let modal_area = Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    // Clear the area behind the modal
    f.render_widget(Clear, modal_area);

    // Create list items
    let items: Vec<ListItem> = app
        .available_regions
        .iter()
        .enumerate()
        .map(|(i, region)| {
            let mut style = if i == app.modal_selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            // Mark current region with indicator
            let display_text = if region == &app.current_region {
                format!("● {region}")
            } else {
                format!("  {region}")
            };

            if region == &app.current_region && i != app.modal_selected_index {
                style = style.fg(Color::Cyan);
            }

            ListItem::new(display_text).style(style)
        })
        .collect();

    let list = List::new(items).block(
        Block::default()
            .title("Select AWS Region (↑↓:navigate | Enter:select | Esc:cancel)")
            .borders(Borders::ALL)
            .border_style(Style::default().fg(Color::Cyan))
            .style(Style::default().bg(Color::Black)),
    );

    f.render_widget(list, modal_area);
}
