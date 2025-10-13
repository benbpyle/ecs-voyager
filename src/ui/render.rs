//! Terminal user interface rendering module.
//!
//! This module contains all UI rendering logic using the ratatui framework.
//! It defines functions for drawing different views (clusters, services, tasks, logs)
//! and UI components (header, footer, overlays).

use chrono::{DateTime, Local};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Clear, List, ListItem, ListState, Paragraph, Row, Table, Wrap},
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
    // Calculate info header height based on view
    let info_header_height = if app.show_help {
        0
    } else {
        match app.state {
            AppState::Clusters | AppState::Services | AppState::Tasks => 3,
            AppState::Logs | AppState::Details => 3,
            _ => 0,
        }
    };

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),                     // Header
            Constraint::Length(info_header_height),    // Info header (context)
            Constraint::Min(0),                        // Content
            Constraint::Length(5),                     // Footer (multi-line)
        ])
        .split(f.area());

    draw_header(f, chunks[0], app);

    // Draw info header if applicable
    if info_header_height > 0 {
        draw_info_header(f, chunks[1], app);
    }

    // Content is always chunks[2] regardless of info_header_height
    // (when info_header_height is 0, chunks[1] exists but has 0 height)
    let content_area = chunks[2];

    if app.show_help {
        draw_help(f, content_area);
    } else {
        match app.state {
            AppState::Clusters => draw_clusters(f, content_area, app),
            AppState::Services => draw_services(f, content_area, app),
            AppState::Tasks => draw_tasks(f, content_area, app),
            AppState::Details => draw_details(f, content_area, app),
            AppState::Logs => draw_logs(f, content_area, app),
            AppState::Metrics => draw_metrics(f, content_area, app),
            AppState::TaskDefinitions => draw_task_definitions(f, content_area, app),
            AppState::TaskDefinitionDetail => draw_details(f, content_area, app),
        }
    }

    // Footer is always chunks[3]
    draw_footer(f, chunks[3], app);

    // Draw search input if in search mode
    if app.search_mode {
        draw_search_input(f, app);
    }

    // Draw modals
    match app.modal_state {
        ModalState::ProfileSelector => draw_profile_selector(f, app),
        ModalState::RegionSelector => draw_region_selector(f, app),
        ModalState::ServiceEditor => draw_service_editor(f, app),
        ModalState::PortForwardingSetup => draw_port_forwarding_setup(f, app),
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
                return draw_custom_header(f, area, &format!("ECS Voyager - Services ({cluster})"), app);
            }
            "ECS Voyager - Services"
        }
        AppState::Tasks => {
            if let (Some(cluster), Some(service)) = (&app.selected_cluster, &app.selected_service) {
                return draw_custom_header(
                    f,
                    area,
                    &format!("ECS Voyager - Tasks ({cluster}/{service})"),
                    app,
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
                    app,
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
                    app,
                );
            }
            "ECS Voyager - Metrics"
        }
        AppState::TaskDefinitions => "ECS Voyager - Task Definitions",
        AppState::TaskDefinitionDetail => "ECS Voyager - Task Definition Details",
    };

    draw_custom_header(f, area, title, app);
}

/// Renders a header with a custom title string.
///
/// Helper function that creates a styled paragraph for the header.
/// If read-only mode is enabled, adds a [READ-ONLY] badge to the title.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the header
/// * `title` - The title text to display
/// * `app` - The application state to check for read-only mode
fn draw_custom_header(f: &mut Frame, area: Rect, title: &str, app: &App) {
    let title_text = if app.config.behavior.read_only {
        format!("{title} [READ-ONLY]")
    } else {
        title.to_string()
    };

    let header = Paragraph::new(title_text)
        .style(
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(header, area);
}

/// Renders the info header showing overview information for the current view.
///
/// Displays context-specific information based on the current view:
/// - Clusters: Region, profile, total count
/// - Services: Cluster info, service counts and aggregates
/// - Tasks: Service info, task counts by status
/// - Logs/Details: Selected resource context
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the info header
/// * `app` - The application state
fn draw_info_header(f: &mut Frame, area: Rect, app: &App) {
    let content = match app.state {
        AppState::Clusters => {
            // Show region, profile, and cluster count
            vec![Line::from(vec![
                Span::styled("Region: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &app.current_region,
                    Style::default()
                        .fg(Color::Cyan)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Profile: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    &app.current_profile,
                    Style::default()
                        .fg(Color::Magenta)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                Span::styled("Total Clusters: ", Style::default().fg(Color::Gray)),
                Span::styled(
                    app.clusters.len().to_string(),
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD),
                ),
            ])]
        }
        AppState::Services => {
            // Show cluster info, service counts and aggregates
            let cluster_name = app
                .selected_cluster
                .as_deref()
                .unwrap_or("None selected");
            let total_services = app.services.len();

            // Calculate aggregate stats
            let total_desired: i32 = app.services.iter().map(|s| s.desired_count).sum();
            let total_running: i32 = app.services.iter().map(|s| s.running_count).sum();
            let total_pending: i32 = app.services.iter().map(|s| s.pending_count).sum();

            // Count by status
            let active_count = app
                .services
                .iter()
                .filter(|s| s.status.to_uppercase() == "ACTIVE")
                .count();
            let draining_count = app
                .services
                .iter()
                .filter(|s| s.status.to_uppercase() == "DRAINING")
                .count();

            // Count by launch type
            let fargate_count = app
                .services
                .iter()
                .filter(|s| s.launch_type.to_uppercase() == "FARGATE")
                .count();
            let ec2_count = app
                .services
                .iter()
                .filter(|s| s.launch_type.to_uppercase() == "EC2")
                .count();

            vec![
                Line::from(vec![
                    Span::styled("Cluster: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        cluster_name,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Services: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        total_services.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled(
                        format!(" (Active: {active_count}, Draining: {draining_count})"),
                        Style::default().fg(Color::Gray),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Tasks: ", Style::default().fg(Color::Gray)),
                    Span::styled("Desired: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        total_desired.to_string(),
                        Style::default().fg(Color::White),
                    ),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Running: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        total_running.to_string(),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Pending: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        total_pending.to_string(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Launch Types: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        format!("Fargate: {fargate_count}, EC2: {ec2_count}"),
                        Style::default().fg(Color::White),
                    ),
                ]),
            ]
        }
        AppState::Tasks => {
            // Show service info, task counts by status
            let cluster_name = app
                .selected_cluster
                .as_deref()
                .unwrap_or("None selected");
            let service_name = app
                .selected_service
                .as_deref()
                .unwrap_or("None selected");
            let total_tasks = app.tasks.len();

            // Count by status
            let running_count = app
                .tasks
                .iter()
                .filter(|t| t.status.to_uppercase() == "RUNNING")
                .count();
            let pending_count = app
                .tasks
                .iter()
                .filter(|t| t.status.to_uppercase() == "PENDING")
                .count();
            let stopped_count = app
                .tasks
                .iter()
                .filter(|t| t.status.to_uppercase() == "STOPPED")
                .count();
            let other_count = total_tasks - running_count - pending_count - stopped_count;

            vec![
                Line::from(vec![
                    Span::styled("Cluster: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        cluster_name,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Service: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        service_name,
                        Style::default()
                            .fg(Color::Magenta)
                            .add_modifier(Modifier::BOLD),
                    ),
                ]),
                Line::from(vec![
                    Span::styled("Total Tasks: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        total_tasks.to_string(),
                        Style::default()
                            .fg(Color::Green)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Running: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        running_count.to_string(),
                        Style::default().fg(Color::Green),
                    ),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Pending: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        pending_count.to_string(),
                        Style::default().fg(Color::Yellow),
                    ),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled("Stopped: ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        stopped_count.to_string(),
                        Style::default().fg(Color::Red),
                    ),
                    if other_count > 0 {
                        Span::styled(
                            format!("  |  Other: {other_count}"),
                            Style::default().fg(Color::Gray),
                        )
                    } else {
                        Span::styled("", Style::default())
                    },
                ]),
            ]
        }
        AppState::Logs => {
            // Show task context and log info
            if let Some(task) = &app.selected_task {
                let filtered_logs = app.get_filtered_logs();
                let log_count_display = if filtered_logs.len() != app.logs.len() {
                    format!("{}/{}", filtered_logs.len(), app.logs.len())
                } else {
                    app.logs.len().to_string()
                };

                vec![
                    Line::from(vec![
                        Span::styled("Task: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            &task.task_id,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                        Span::styled("Status: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            &task.status,
                            Style::default().fg(if task.status.to_uppercase() == "RUNNING" {
                                Color::Green
                            } else {
                                Color::Yellow
                            }),
                        ),
                        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                        Span::styled("CPU: ", Style::default().fg(Color::Gray)),
                        Span::styled(&task.cpu, Style::default().fg(Color::White)),
                        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                        Span::styled("Memory: ", Style::default().fg(Color::Gray)),
                        Span::styled(&task.memory, Style::default().fg(Color::White)),
                    ]),
                    Line::from(vec![
                        Span::styled("Log Entries: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            log_count_display,
                            Style::default()
                                .fg(Color::Green)
                                .add_modifier(Modifier::BOLD),
                        ),
                        if app.auto_tail {
                            Span::styled(
                                "  |  Auto-Tail: ON",
                                Style::default().fg(Color::Green),
                            )
                        } else {
                            Span::styled(
                                "  |  Auto-Tail: OFF",
                                Style::default().fg(Color::DarkGray),
                            )
                        },
                    ]),
                ]
            } else {
                vec![Line::from(vec![Span::styled(
                    "No task selected",
                    Style::default().fg(Color::Yellow),
                )])]
            }
        }
        AppState::Details => {
            // Show resource context based on what's selected
            if let Some(task) = &app.selected_task {
                vec![
                    Line::from(vec![
                        Span::styled("Task Details: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            &task.task_id,
                            Style::default()
                                .fg(Color::Cyan)
                                .add_modifier(Modifier::BOLD),
                        ),
                        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                        Span::styled("Status: ", Style::default().fg(Color::Gray)),
                        Span::styled(
                            &task.status,
                            Style::default().fg(if task.status.to_uppercase() == "RUNNING" {
                                Color::Green
                            } else {
                                Color::Yellow
                            }),
                        ),
                    ]),
                    Line::from(vec![
                        Span::styled("CPU: ", Style::default().fg(Color::Gray)),
                        Span::styled(&task.cpu, Style::default().fg(Color::White)),
                        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                        Span::styled("Memory: ", Style::default().fg(Color::Gray)),
                        Span::styled(&task.memory, Style::default().fg(Color::White)),
                        Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                        Span::styled(
                            if app.show_json_view {
                                "View: JSON"
                            } else {
                                "View: Formatted"
                            },
                            Style::default().fg(Color::Magenta),
                        ),
                    ]),
                ]
            } else if let Some(service) = &app.selected_service {
                vec![Line::from(vec![
                    Span::styled("Service Details: ", Style::default().fg(Color::Gray)),
                    Span::styled(
                        service,
                        Style::default()
                            .fg(Color::Cyan)
                            .add_modifier(Modifier::BOLD),
                    ),
                    Span::styled("  |  ", Style::default().fg(Color::DarkGray)),
                    Span::styled(
                        if app.show_json_view {
                            "View: JSON"
                        } else {
                            "View: Formatted"
                        },
                        Style::default().fg(Color::Magenta),
                    ),
                ])]
            } else {
                vec![Line::from(vec![Span::styled(
                    "Resource Details",
                    Style::default().fg(Color::Gray),
                )])]
            }
        }
        _ => vec![],
    };

    let info_header = Paragraph::new(content)
        .style(Style::default().fg(Color::White))
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::DarkGray)),
        );

    f.render_widget(info_header, area);
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
            AppState::TaskDefinitions => format!("{} families", app.task_definition_families.len()),
            AppState::TaskDefinitionDetail => "task definition".to_string(),
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

        // Build filter status string
        let mut filter_parts = Vec::new();
        if let Some(ref status) = app.service_status_filter {
            filter_parts.push(format!("Status:{status}"));
        }
        if let Some(ref lt) = app.launch_type_filter {
            filter_parts.push(format!("Type:{lt}"));
        }
        if let Some(ref task_status) = app.task_status_filter {
            filter_parts.push(format!("TaskStatus:{task_status}"));
        }
        let filter_text = if !filter_parts.is_empty() {
            format!(" | Filters: {}", filter_parts.join(", "))
        } else {
            String::new()
        };

        let mut line3_spans = vec![
            Span::styled("Last refresh: ", Style::default().fg(Color::Gray)),
            Span::styled(refresh_text, Style::default().fg(Color::White)),
            Span::styled(" | ", Style::default().fg(Color::DarkGray)),
            Span::styled("Auto-refresh: ", Style::default().fg(Color::Gray)),
            Span::styled(auto_refresh_status, Style::default().fg(Color::White)),
        ];

        // Add search status if active
        if app.search_mode || !app.search_query.is_empty() {
            let search_mode_indicator = if app.search_regex_mode {
                "Regex"
            } else {
                "Search"
            };
            line3_spans.push(Span::styled(
                format!(
                    " | {}: {}",
                    search_mode_indicator,
                    if app.search_query.is_empty() {
                        "_".to_string()
                    } else {
                        app.search_query.clone()
                    }
                ),
                Style::default().fg(Color::Yellow),
            ));
        }

        // Add filter status if active
        if !filter_text.is_empty() {
            line3_spans.push(Span::styled(
                filter_text,
                Style::default().fg(Color::Magenta),
            ));
        }

        let line3 = Line::from(line3_spans);

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
            "Services ({}) - /:search | s:edit | Enter:tasks | d:describe | x:restart",
            filtered_services.len()
        )
    } else {
        format!(
            "Services ({}/{}) - Esc:clear | s:edit | Enter:tasks | d:describe | x:restart",
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
            "Tasks ({}) - /:search | e:exec | l:logs | d:describe | x:stop",
            filtered_tasks.len()
        )
    } else {
        format!(
            "Tasks ({}/{}) - Esc:clear | e:exec | l:logs | d:describe | x:stop",
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

/// Renders the task definitions list view.
///
/// Displays all ECS task definition families (filtered by search query if active) as a vertical list.
/// The currently selected family is highlighted. Shows count of filtered vs total families.
///
/// # Arguments
/// * `f` - The ratatui Frame to render into
/// * `area` - The rectangular area allocated for the task definitions list
/// * `app` - The application state containing task definition family data
fn draw_task_definitions(f: &mut Frame, area: Rect, app: &App) {
    let filtered_families = app.get_filtered_task_definition_families();

    let items: Vec<ListItem> = filtered_families
        .iter()
        .enumerate()
        .map(|(i, family)| {
            let style = if i == app.selected_index {
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD)
            } else {
                Style::default().fg(Color::White)
            };

            ListItem::new(family.as_str()).style(style)
        })
        .collect();

    let title = if app.search_query.is_empty() {
        format!(
            "Task Definition Families ({}) - /:search | Enter:view | d:describe",
            filtered_families.len()
        )
    } else {
        format!(
            "Task Definition Families ({}/{}) - Esc:clear | Enter:view | d:describe",
            filtered_families.len(),
            app.task_definition_families.len()
        )
    };

    let list = List::new(items).block(Block::default().title(title).borders(Borders::ALL));

    f.render_widget(list, area);
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

    let title = format!("Details - {view_type} View (↑↓:scroll | J:toggle | Esc/h:back)");

    let paragraph = Paragraph::new(content.as_str())
        .style(Style::default().fg(Color::White))
        .block(Block::default().title(title).borders(Borders::ALL))
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
            .filter_map(|dp| {
                dp.average.map(|avg| ChartDatapoint {
                    timestamp: dp.timestamp,
                    value: avg,
                })
            })
            .collect();

        let chart_config = ChartConfig {
            width: 60,
            height: 10,
            min_value: None, // Auto-scale based on data
            max_value: None, // Auto-scale based on data
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
            / metrics
                .cpu_datapoints
                .iter()
                .filter(|dp| dp.average.is_some())
                .count() as f64;
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
            .filter_map(|dp| {
                dp.average.map(|avg| ChartDatapoint {
                    timestamp: dp.timestamp,
                    value: avg,
                })
            })
            .collect();

        if !mem_chart_datapoints.is_empty() {
            let chart_config = ChartConfig {
                width: 60,
                height: 10,
                min_value: None, // Auto-scale based on data
                max_value: None, // Auto-scale based on data
                line_color: Color::Cyan,
                show_y_labels: true,
            };

            let chart_lines = render_chart(
                &mem_chart_datapoints,
                &chart_config,
                "Memory Utilization (%)",
            );
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
            / metrics
                .memory_datapoints
                .iter()
                .filter(|dp| dp.average.is_some())
                .count() as f64;
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
                    "Metrics [{time_range_label}] (T:cycle range | r:refresh | Esc/h:back | ↑↓:scroll)"
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
            Span::styled(
                format!("  {state_symbol} "),
                Style::default().fg(state_color),
            ),
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
            Span::styled("  e           ", Style::default().fg(Color::Yellow)),
            Span::raw("ECS Exec shell (from Tasks view)"),
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
            Span::styled("  s           ", Style::default().fg(Color::Yellow)),
            Span::raw("Edit service (from Services view)"),
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
            Span::styled("  x           ", Style::default().fg(Color::Yellow)),
            Span::raw("Execute action (restart service/stop task)"),
        ]),
        Line::from(""),
        Line::from(vec![Span::styled(
            "Search & Filters",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        )]),
        Line::from(vec![
            Span::styled("  /           ", Style::default().fg(Color::Yellow)),
            Span::raw("Enter search mode (Clusters/Services/Tasks)"),
        ]),
        Line::from(vec![
            Span::styled("  M           ", Style::default().fg(Color::Yellow)),
            Span::raw("Toggle regex mode for search"),
        ]),
        Line::from(vec![
            Span::styled("  F           ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle status filter (Services/Tasks)"),
        ]),
        Line::from(vec![
            Span::styled("  L           ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle launch type filter (Services)"),
        ]),
        Line::from(vec![
            Span::styled("  C           ", Style::default().fg(Color::Yellow)),
            Span::raw("Clear all active filters"),
        ]),
        Line::from(vec![
            Span::styled("  f           ", Style::default().fg(Color::Yellow)),
            Span::raw("Cycle log level filter (Logs view)"),
        ]),
        Line::from(vec![
            Span::styled("  e           ", Style::default().fg(Color::Yellow)),
            Span::raw("Export logs to file (Logs view)"),
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

/// Renders the service editor modal.
///
/// Displays a centered modal dialog with fields to edit the service configuration:
/// - Desired Count input field
/// - Task Definition revision selector
fn draw_service_editor(f: &mut Frame, app: &App) {
    use ratatui::layout::Constraint;
    use ratatui::widgets::Paragraph;

    let area = f.area();
    let width = 80.min(area.width.saturating_sub(4));

    // Calculate height based on number of task definition revisions
    let revisions_count = app.service_editor_available_revisions.len().min(10);
    let height = (revisions_count + 12) as u16; // 12 = header + fields + padding

    let modal_area = Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    // Clear the area behind the modal
    f.render_widget(Clear, modal_area);

    // Create main container
    let block = Block::default()
        .title("Edit Service (Tab:switch field | Enter:save | Esc:cancel)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    f.render_widget(block, modal_area);

    // Split the modal into sections
    let inner = modal_area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Service info
            Constraint::Length(3), // Desired count field
            Constraint::Length(1), // Spacing
            Constraint::Length(2), // Task definition label
            Constraint::Min(5),    // Task definition list
        ])
        .split(inner);

    // Service name display
    let service_name = app.selected_service.as_deref().unwrap_or("Unknown");
    let service_info = Paragraph::new(format!("Service: {service_name}")).style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(service_info, chunks[0]);

    // Desired Count input field
    let desired_count_style = if app.service_editor_editing_field == 0 {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let desired_count_text = format!(
        "Desired Count: {}{}",
        app.service_editor_desired_count_input,
        if app.service_editor_editing_field == 0 {
            "█"
        } else {
            ""
        }
    );

    let desired_count_widget = Paragraph::new(desired_count_text)
        .style(desired_count_style)
        .block(Block::default().borders(Borders::ALL).border_style(
            if app.service_editor_editing_field == 0 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            },
        ));
    f.render_widget(desired_count_widget, chunks[1]);

    // Task Definition label
    let task_def_label = Paragraph::new("Task Definition Revision:").style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(task_def_label, chunks[3]);

    // Task Definition revision list
    if !app.service_editor_available_revisions.is_empty() {
        let items: Vec<ListItem> = app
            .service_editor_available_revisions
            .iter()
            .enumerate()
            .map(|(i, revision)| {
                // Extract just the family:revision part from the ARN
                let display_revision = revision.split('/').next_back().unwrap_or(revision);

                let is_current = revision.contains(&app.service_editor_current_task_def);
                let is_selected = i == app.service_editor_selected_revision;

                let style = if is_selected && app.service_editor_editing_field == 1 {
                    Style::default()
                        .fg(Color::Black)
                        .bg(Color::Cyan)
                        .add_modifier(Modifier::BOLD)
                } else if is_current {
                    Style::default()
                        .fg(Color::Green)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::White)
                };

                let prefix = if is_current { "● " } else { "  " };

                ListItem::new(format!("{prefix}{display_revision}")).style(style)
            })
            .collect();

        // Create a stateful list
        let mut list_state = ListState::default();
        list_state.select(Some(app.service_editor_selected_revision));

        let list = List::new(items)
            .block(Block::default().borders(Borders::ALL).border_style(
                if app.service_editor_editing_field == 1 {
                    Style::default().fg(Color::Cyan)
                } else {
                    Style::default().fg(Color::Gray)
                },
            ))
            .highlight_style(
                Style::default()
                    .fg(Color::Black)
                    .bg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );

        f.render_stateful_widget(list, chunks[4], &mut list_state);
    } else {
        let no_revisions = Paragraph::new("No task definition revisions found")
            .style(Style::default().fg(Color::Yellow))
            .block(Block::default().borders(Borders::ALL));
        f.render_widget(no_revisions, chunks[4]);
    }
}

/// Renders the port forwarding setup modal.
///
/// Displays a centered modal dialog with fields to configure port forwarding:
/// - Local port number input field
/// - Remote port number input field
fn draw_port_forwarding_setup(f: &mut Frame, app: &App) {
    let area = f.area();
    let width = 60.min(area.width.saturating_sub(4));
    let height = 12;

    let modal_area = Rect {
        x: area.width.saturating_sub(width) / 2,
        y: area.height.saturating_sub(height) / 2,
        width,
        height,
    };

    // Clear the area behind the modal
    f.render_widget(Clear, modal_area);

    // Create main container
    let block = Block::default()
        .title("Port Forwarding Setup (Tab:switch field | Enter:start | Esc:cancel)")
        .borders(Borders::ALL)
        .border_style(Style::default().fg(Color::Cyan))
        .style(Style::default().bg(Color::Black));

    f.render_widget(block, modal_area);

    // Split the modal into sections
    let inner = modal_area.inner(Margin {
        horizontal: 2,
        vertical: 1,
    });

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Task info
            Constraint::Length(3), // Local port field
            Constraint::Length(3), // Remote port field
        ])
        .split(inner);

    // Task info display
    let task_id = app
        .selected_task
        .as_ref()
        .map(|t| t.task_id.as_str())
        .unwrap_or("Unknown");
    let task_info = Paragraph::new(format!("Task: {task_id}")).style(
        Style::default()
            .fg(Color::White)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(task_info, chunks[0]);

    // Local port input field
    let local_port_style = if app.port_forward_editing_field == 0 {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let local_port_text = format!(
        "Local Port:  {}{}",
        app.port_forward_local_port,
        if app.port_forward_editing_field == 0 {
            "█"
        } else {
            ""
        }
    );

    let local_port_widget = Paragraph::new(local_port_text)
        .style(local_port_style)
        .block(Block::default().borders(Borders::ALL).border_style(
            if app.port_forward_editing_field == 0 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            },
        ));
    f.render_widget(local_port_widget, chunks[1]);

    // Remote port input field
    let remote_port_style = if app.port_forward_editing_field == 1 {
        Style::default()
            .fg(Color::Black)
            .bg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::White)
    };

    let remote_port_text = format!(
        "Remote Port: {}{}",
        app.port_forward_remote_port,
        if app.port_forward_editing_field == 1 {
            "█"
        } else {
            ""
        }
    );

    let remote_port_widget = Paragraph::new(remote_port_text)
        .style(remote_port_style)
        .block(Block::default().borders(Borders::ALL).border_style(
            if app.port_forward_editing_field == 1 {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::Gray)
            },
        ));
    f.render_widget(remote_port_widget, chunks[2]);
}
