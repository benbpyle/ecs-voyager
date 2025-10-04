//! Application state management module.
//!
//! This module defines the core application state, data structures for ECS resources,
//! and methods for navigating between views and managing data.

use anyhow::Result;
use std::time::{Duration, Instant};

use crate::aws::EcsClient;
use crate::config::Config;

/// Represents the current view/screen in the application.
///
/// The application follows a hierarchical navigation pattern:
/// Clusters -> Services -> Tasks -> Details/Logs
#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    /// View showing list of ECS clusters
    Clusters,
    /// View showing services for selected cluster
    Services,
    /// View showing tasks for selected service
    Tasks,
    /// View showing detailed information about a resource
    Details,
    /// View showing CloudWatch logs for a task
    Logs,
}

/// Main application state container.
///
/// Holds all UI state, data from AWS, and manages navigation between views.
/// The app maintains a selected cluster/service/task context as the user
/// navigates through the hierarchy.
pub struct App {
    /// Current view state
    pub state: AppState,
    /// Previous state for navigation history
    pub previous_state: Option<AppState>,
    /// Whether help overlay is shown
    pub show_help: bool,
    /// Currently selected item index in lists
    pub selected_index: usize,
    /// AWS ECS client for API calls
    pub ecs_client: EcsClient,
    /// Application configuration
    pub config: Config,

    // Data
    /// List of ECS cluster names
    pub clusters: Vec<String>,
    /// List of services in selected cluster
    pub services: Vec<ServiceInfo>,
    /// List of tasks for selected service
    pub tasks: Vec<TaskInfo>,
    /// Currently selected cluster name
    pub selected_cluster: Option<String>,
    /// Currently selected service name
    pub selected_service: Option<String>,
    /// Currently selected task information
    pub selected_task: Option<TaskInfo>,
    /// Detailed description text for resources
    pub details: Option<String>,
    /// Log entries for selected task
    pub logs: Vec<LogEntry>,
    /// Current scroll position in logs
    pub log_scroll: usize,
    /// Whether to auto-scroll to latest logs
    pub auto_tail: bool,

    // Search
    /// Whether search input mode is active
    pub search_mode: bool,
    /// Current search/filter query string
    pub search_query: String,

    // Status
    /// Status message displayed to user
    pub status_message: String,
    /// Whether a background operation is in progress
    pub loading: bool,
    /// Timestamp of last data refresh
    pub last_refresh: Instant,
}

/// Information about an ECS service.
///
/// Contains service metadata including name, status, and task counts.
#[derive(Debug, Clone)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,
    /// Service status (e.g., ACTIVE, DRAINING)
    pub status: String,
    /// Number of tasks that should be running
    pub desired_count: i32,
    /// Number of tasks currently running
    pub running_count: i32,
    /// Number of tasks pending startup
    pub pending_count: i32,
    /// Launch type (EC2, FARGATE, or EXTERNAL)
    pub launch_type: String,
}

/// Information about an ECS task.
///
/// Contains task metadata including ARN, status, and resource allocation.
#[derive(Debug, Clone)]
pub struct TaskInfo {
    /// Full ARN of the task
    pub task_arn: String,
    /// Short task ID (last segment of ARN)
    pub task_id: String,
    /// Current task status (e.g., RUNNING, STOPPED)
    pub status: String,
    /// Desired task status
    pub desired_status: String,
    /// Container instance ARN (for EC2 launch type)
    pub container_instance: String,
    /// CPU units allocated to task
    pub cpu: String,
    /// Memory (MB) allocated to task
    pub memory: String,
}

/// A single log entry from CloudWatch Logs.
///
/// Represents one log line from a container with timestamp and metadata.
#[derive(Debug, Clone)]
pub struct LogEntry {
    /// Unix timestamp in milliseconds
    pub timestamp: i64,
    /// Log message text
    pub message: String,
    /// Name of the container that produced the log
    pub container_name: String,
}

impl App {
    /// Creates a new application instance and loads initial data.
    ///
    /// Initializes the AWS ECS client using credentials from the environment,
    /// sets up the initial state, and performs the first data refresh to load
    /// the list of clusters.
    ///
    /// # Returns
    /// Returns a new `App` instance with cluster data loaded, or an error if
    /// initialization fails.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - AWS credentials are not configured or invalid
    /// - AWS SDK client initialization fails
    /// - The initial cluster list API call fails
    pub async fn new() -> Result<Self> {
        let ecs_client = EcsClient::new().await?;
        let mut app = Self {
            state: AppState::Clusters,
            previous_state: None,
            show_help: false,
            selected_index: 0,
            ecs_client,
            clusters: Vec::new(),
            services: Vec::new(),
            tasks: Vec::new(),
            selected_cluster: None,
            selected_service: None,
            selected_task: None,
            details: None,
            logs: Vec::new(),
            log_scroll: 0,
            auto_tail: true,
            search_mode: false,
            search_query: String::new(),
            status_message: "Loading clusters...".to_string(),
            loading: false,
            last_refresh: Instant::now(),
        };

        app.refresh().await?;
        Ok(app)
    }

    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
    }

    pub fn set_view(&mut self, state: AppState) {
        self.previous_state = Some(self.state.clone());
        self.state = state;
        self.selected_index = 0;
    }

    pub fn next(&mut self) {
        let len = match self.state {
            AppState::Clusters => self.clusters.len(),
            AppState::Services => self.services.len(),
            AppState::Tasks => self.tasks.len(),
            AppState::Details => 0,
            AppState::Logs => {
                // Scroll down in logs
                if self.logs.len() > 0 {
                    self.log_scroll = self.log_scroll.saturating_add(1);
                    self.auto_tail = false;
                }
                return;
            }
        };

        if len > 0 {
            self.selected_index = (self.selected_index + 1) % len;
        }
    }

    pub fn previous(&mut self) {
        let len = match self.state {
            AppState::Clusters => self.clusters.len(),
            AppState::Services => self.services.len(),
            AppState::Tasks => self.tasks.len(),
            AppState::Details => 0,
            AppState::Logs => {
                // Scroll up in logs
                self.log_scroll = self.log_scroll.saturating_sub(1);
                self.auto_tail = false;
                return;
            }
        };

        if len > 0 {
            self.selected_index = if self.selected_index == 0 {
                len - 1
            } else {
                self.selected_index - 1
            };
        }
    }

    pub async fn select(&mut self) -> Result<()> {
        match self.state {
            AppState::Clusters => {
                if let Some(cluster) = self.clusters.get(self.selected_index) {
                    self.selected_cluster = Some(cluster.clone());
                    self.loading = true;
                    self.status_message = format!("Loading services for cluster: {}", cluster);
                    self.services = self.ecs_client.list_services(cluster).await?;
                    self.loading = false;
                    self.set_view(AppState::Services);
                    self.status_message = format!("Loaded {} services", self.services.len());
                }
            }
            AppState::Services => {
                if let Some(service) = self.services.get(self.selected_index) {
                    self.selected_service = Some(service.name.clone());
                    if let Some(cluster) = &self.selected_cluster {
                        self.loading = true;
                        self.status_message = format!("Loading tasks for service: {}", service.name);
                        self.tasks = self.ecs_client.list_tasks(cluster, &service.name).await?;
                        self.loading = false;
                        self.set_view(AppState::Tasks);
                        self.status_message = format!("Loaded {} tasks", self.tasks.len());
                    }
                }
            }
            AppState::Tasks => {
                // View task details
                if let Some(task) = self.tasks.get(self.selected_index) {
                    if let Some(cluster) = &self.selected_cluster {
                        self.loading = true;
                        self.status_message = "Loading task details...".to_string();
                        self.details = Some(self.ecs_client.describe_task(cluster, &task.task_arn).await?);
                        self.loading = false;
                        self.set_view(AppState::Details);
                        self.status_message = "Task details loaded".to_string();
                    }
                }
            }
            AppState::Details => {}
            AppState::Logs => {}
        }
        Ok(())
    }

    pub fn back(&mut self) {
        match self.state {
            AppState::Services => {
                self.set_view(AppState::Clusters);
                self.selected_service = None;
            }
            AppState::Tasks => {
                self.set_view(AppState::Services);
            }
            AppState::Details => {
                self.set_view(AppState::Tasks);
                self.details = None;
            }
            AppState::Logs => {
                self.set_view(AppState::Tasks);
                self.logs.clear();
                self.log_scroll = 0;
                self.auto_tail = true;
            }
            AppState::Clusters => {}
        }
    }

    pub async fn refresh(&mut self) -> Result<()> {
        self.loading = true;
        self.last_refresh = Instant::now();

        match self.state {
            AppState::Clusters => {
                self.status_message = "Refreshing clusters...".to_string();
                self.clusters = self.ecs_client.list_clusters().await?;
                self.status_message = format!("Loaded {} clusters", self.clusters.len());
            }
            AppState::Services => {
                if let Some(cluster) = &self.selected_cluster {
                    self.status_message = "Refreshing services...".to_string();
                    self.services = self.ecs_client.list_services(cluster).await?;
                    self.status_message = format!("Loaded {} services", self.services.len());
                }
            }
            AppState::Tasks => {
                if let (Some(cluster), Some(service)) = (&self.selected_cluster, &self.selected_service) {
                    self.status_message = "Refreshing tasks...".to_string();
                    self.tasks = self.ecs_client.list_tasks(cluster, service).await?;
                    self.status_message = format!("Loaded {} tasks", self.tasks.len());
                }
            }
            AppState::Details => {}
            AppState::Logs => {
                // Refresh logs if we have a selected task
                if let (Some(cluster), Some(task)) = (&self.selected_cluster, &self.selected_task) {
                    self.status_message = "Refreshing logs...".to_string();
                    self.logs = self.ecs_client.get_task_logs(cluster, &task.task_arn).await?;
                    if self.auto_tail && !self.logs.is_empty() {
                        self.log_scroll = self.logs.len().saturating_sub(1);
                    }
                    self.status_message = format!("Loaded {} log entries", self.logs.len());
                }
            }
        }

        self.loading = false;
        Ok(())
    }

    pub async fn describe(&mut self) -> Result<()> {
        match self.state {
            AppState::Services => {
                if let Some(service) = self.services.get(self.selected_index) {
                    if let Some(cluster) = &self.selected_cluster {
                        self.loading = true;
                        self.status_message = format!("Describing service: {}", service.name);
                        self.details = Some(self.ecs_client.describe_service(cluster, &service.name).await?);
                        self.loading = false;
                        self.set_view(AppState::Details);
                        self.status_message = "Service details loaded".to_string();
                    }
                }
            }
            AppState::Tasks => {
                if let Some(task) = self.tasks.get(self.selected_index) {
                    if let Some(cluster) = &self.selected_cluster {
                        self.loading = true;
                        self.status_message = "Describing task...".to_string();
                        self.details = Some(self.ecs_client.describe_task(cluster, &task.task_arn).await?);
                        self.loading = false;
                        self.set_view(AppState::Details);
                        self.status_message = "Task details loaded".to_string();
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub async fn execute_action(&mut self) -> Result<()> {
        match self.state {
            AppState::Services => {
                if let Some(service) = self.services.get(self.selected_index) {
                    if let Some(cluster) = &self.selected_cluster {
                        self.loading = true;
                        self.status_message = format!("Restarting service: {}", service.name);
                        self.ecs_client.restart_service(cluster, &service.name).await?;
                        self.status_message = format!("Service {} restarted", service.name);
                        self.refresh().await?;
                        self.loading = false;
                    }
                }
            }
            AppState::Tasks => {
                if let Some(task) = self.tasks.get(self.selected_index) {
                    if let Some(cluster) = &self.selected_cluster {
                        self.loading = true;
                        self.status_message = format!("Stopping task: {}", task.task_id);
                        self.ecs_client.stop_task(cluster, &task.task_arn).await?;
                        self.status_message = format!("Task {} stopped", task.task_id);
                        self.refresh().await?;
                        self.loading = false;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub fn should_refresh(&self) -> bool {
        // Auto-refresh logs more frequently when in Logs view
        let refresh_interval = if self.state == AppState::Logs && self.auto_tail {
            Duration::from_secs(5)
        } else {
            Duration::from_secs(30)
        };
        self.last_refresh.elapsed() > refresh_interval
    }

    pub async fn view_logs(&mut self) -> Result<()> {
        if self.state == AppState::Tasks {
            if let Some(task) = self.tasks.get(self.selected_index) {
                self.selected_task = Some(task.clone());
                if let Some(cluster) = &self.selected_cluster {
                    self.loading = true;
                    self.status_message = format!("Loading logs for task: {}", task.task_id);
                    self.logs = self.ecs_client.get_task_logs(cluster, &task.task_arn).await?;
                    self.loading = false;
                    self.log_scroll = if !self.logs.is_empty() {
                        self.logs.len().saturating_sub(1)
                    } else {
                        0
                    };
                    self.auto_tail = true;
                    self.set_view(AppState::Logs);
                    self.status_message = format!("Loaded {} log entries (auto-tail enabled)", self.logs.len());
                }
            }
        }
        Ok(())
    }

    pub fn toggle_auto_tail(&mut self) {
        self.auto_tail = !self.auto_tail;
        if self.auto_tail && !self.logs.is_empty() {
            self.log_scroll = self.logs.len().saturating_sub(1);
        }
        self.status_message = format!(
            "Auto-tail {}",
            if self.auto_tail { "enabled" } else { "disabled" }
        );
    }

    // Search methods
    pub fn enter_search_mode(&mut self) {
        self.search_mode = true;
        self.search_query.clear();
        self.selected_index = 0;
    }

    pub fn exit_search_mode(&mut self) {
        self.search_mode = false;
    }

    pub fn clear_search(&mut self) {
        self.search_mode = false;
        self.search_query.clear();
        self.selected_index = 0;
    }

    pub fn update_search(&mut self, c: char) {
        self.search_query.push(c);
        self.selected_index = 0;
    }

    pub fn delete_search_char(&mut self) {
        self.search_query.pop();
        self.selected_index = 0;
    }

    pub fn get_filtered_clusters(&self) -> Vec<String> {
        if self.search_query.is_empty() {
            self.clusters.clone()
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.clusters
                .iter()
                .filter(|cluster| cluster.to_lowercase().contains(&query_lower))
                .cloned()
                .collect()
        }
    }

    pub fn get_filtered_services(&self) -> Vec<ServiceInfo> {
        if self.search_query.is_empty() {
            self.services.clone()
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.services
                .iter()
                .filter(|service| {
                    service.name.to_lowercase().contains(&query_lower)
                        || service.status.to_lowercase().contains(&query_lower)
                        || service.launch_type.to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect()
        }
    }

    pub fn get_filtered_tasks(&self) -> Vec<TaskInfo> {
        if self.search_query.is_empty() {
            self.tasks.clone()
        } else {
            let query_lower = self.search_query.to_lowercase();
            self.tasks
                .iter()
                .filter(|task| {
                    task.task_id.to_lowercase().contains(&query_lower)
                        || task.status.to_lowercase().contains(&query_lower)
                        || task.desired_status.to_lowercase().contains(&query_lower)
                })
                .cloned()
                .collect()
        }
    }
}
