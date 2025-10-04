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
    /// Initializes the AWS ECS client using configuration settings,
    /// sets up the initial state, and performs the first data refresh to load
    /// the list of clusters.
    ///
    /// # Arguments
    /// * `config` - Application configuration loaded from config file
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
    pub async fn new(config: Config) -> Result<Self> {
        // Initialize ECS client with config settings
        let ecs_client = EcsClient::new(
            config.aws.region.clone(),
            config.aws.profile.clone(),
        ).await?;

        // Determine initial state based on config
        let initial_state = match config.behavior.default_view.as_str() {
            "services" => AppState::Services,
            "tasks" => AppState::Tasks,
            _ => AppState::Clusters,
        };

        let mut app = Self {
            state: initial_state,
            previous_state: None,
            show_help: false,
            selected_index: 0,
            ecs_client,
            config,
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
        // Skip auto-refresh if disabled in config
        if !self.config.behavior.auto_refresh {
            return false;
        }

        // Auto-refresh logs more frequently when in Logs view
        let refresh_interval = if self.state == AppState::Logs && self.auto_tail {
            Duration::from_secs(5)
        } else {
            Duration::from_secs(self.config.behavior.refresh_interval)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{Config, AwsConfig, BehaviorConfig, UiConfig};
    use std::mem::ManuallyDrop;

    // Helper function to create a test config
    fn create_test_config() -> Config {
        Config {
            aws: AwsConfig {
                region: None,
                profile: None,
            },
            behavior: BehaviorConfig {
                auto_refresh: true,
                refresh_interval: 30,
                default_view: "clusters".to_string(),
            },
            ui: UiConfig {
                theme: "dark".to_string(),
            },
        }
    }

    // Helper function to create a mock App for testing
    // We wrap in ManuallyDrop to avoid dropping the uninitialized EcsClient
    // We use MaybeUninit to safely create an uninitialized EcsClient
    fn create_test_app() -> ManuallyDrop<App> {
        use std::mem::MaybeUninit;

        let fake_client = MaybeUninit::<EcsClient>::uninit();
        ManuallyDrop::new(App {
            state: AppState::Clusters,
            previous_state: None,
            show_help: false,
            selected_index: 0,
            ecs_client: unsafe { fake_client.assume_init() },
            config: create_test_config(),
            clusters: vec![
                "cluster-prod".to_string(),
                "cluster-dev".to_string(),
                "cluster-staging".to_string(),
            ],
            services: vec![
                ServiceInfo {
                    name: "web-service".to_string(),
                    status: "ACTIVE".to_string(),
                    desired_count: 3,
                    running_count: 3,
                    pending_count: 0,
                    launch_type: "FARGATE".to_string(),
                },
                ServiceInfo {
                    name: "api-service".to_string(),
                    status: "ACTIVE".to_string(),
                    desired_count: 5,
                    running_count: 4,
                    pending_count: 1,
                    launch_type: "EC2".to_string(),
                },
                ServiceInfo {
                    name: "worker-service".to_string(),
                    status: "DRAINING".to_string(),
                    desired_count: 2,
                    running_count: 1,
                    pending_count: 0,
                    launch_type: "FARGATE".to_string(),
                },
            ],
            tasks: vec![
                TaskInfo {
                    task_arn: "arn:aws:ecs:us-east-1:123456789012:task/task-abc123".to_string(),
                    task_id: "task-abc123".to_string(),
                    status: "RUNNING".to_string(),
                    desired_status: "RUNNING".to_string(),
                    container_instance: "instance-1".to_string(),
                    cpu: "256".to_string(),
                    memory: "512".to_string(),
                },
                TaskInfo {
                    task_arn: "arn:aws:ecs:us-east-1:123456789012:task/task-def456".to_string(),
                    task_id: "task-def456".to_string(),
                    status: "PENDING".to_string(),
                    desired_status: "RUNNING".to_string(),
                    container_instance: "instance-2".to_string(),
                    cpu: "512".to_string(),
                    memory: "1024".to_string(),
                },
                TaskInfo {
                    task_arn: "arn:aws:ecs:us-east-1:123456789012:task/task-ghi789".to_string(),
                    task_id: "task-ghi789".to_string(),
                    status: "STOPPED".to_string(),
                    desired_status: "STOPPED".to_string(),
                    container_instance: "none".to_string(),
                    cpu: "256".to_string(),
                    memory: "512".to_string(),
                },
            ],
            selected_cluster: None,
            selected_service: None,
            selected_task: None,
            details: None,
            logs: vec![],
            log_scroll: 0,
            auto_tail: true,
            search_mode: false,
            search_query: String::new(),
            status_message: "Ready".to_string(),
            loading: false,
            last_refresh: Instant::now(),
        })
    }

    // Test search filtering
    #[test]
    fn test_get_filtered_clusters_empty_query() {
        let app = create_test_app();
        let filtered = app.get_filtered_clusters();
        assert_eq!(filtered.len(), 3);
        assert_eq!(filtered, app.clusters);
    }

    #[test]
    fn test_get_filtered_clusters_with_query() {
        let mut app = create_test_app();
        app.search_query = "prod".to_string();
        let filtered = app.get_filtered_clusters();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "cluster-prod");
    }

    #[test]
    fn test_get_filtered_clusters_case_insensitive() {
        let mut app = create_test_app();
        app.search_query = "PROD".to_string();
        let filtered = app.get_filtered_clusters();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "cluster-prod");
    }

    #[test]
    fn test_get_filtered_clusters_partial_match() {
        let mut app = create_test_app();
        app.search_query = "dev".to_string();
        let filtered = app.get_filtered_clusters();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "cluster-dev");
    }

    #[test]
    fn test_get_filtered_clusters_no_match() {
        let mut app = create_test_app();
        app.search_query = "nonexistent".to_string();
        let filtered = app.get_filtered_clusters();
        assert_eq!(filtered.len(), 0);
    }

    #[test]
    fn test_get_filtered_services_empty_query() {
        let app = create_test_app();
        let filtered = app.get_filtered_services();
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_get_filtered_services_by_name() {
        let mut app = create_test_app();
        app.search_query = "web".to_string();
        let filtered = app.get_filtered_services();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "web-service");
    }

    #[test]
    fn test_get_filtered_services_by_status() {
        let mut app = create_test_app();
        app.search_query = "DRAINING".to_string();
        let filtered = app.get_filtered_services();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].name, "worker-service");
    }

    #[test]
    fn test_get_filtered_services_by_launch_type() {
        let mut app = create_test_app();
        app.search_query = "FARGATE".to_string();
        let filtered = app.get_filtered_services();
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|s| s.name == "web-service"));
        assert!(filtered.iter().any(|s| s.name == "worker-service"));
    }

    #[test]
    fn test_get_filtered_tasks_empty_query() {
        let app = create_test_app();
        let filtered = app.get_filtered_tasks();
        assert_eq!(filtered.len(), 3);
    }

    #[test]
    fn test_get_filtered_tasks_by_id() {
        let mut app = create_test_app();
        app.search_query = "abc123".to_string();
        let filtered = app.get_filtered_tasks();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].task_id, "task-abc123");
    }

    #[test]
    fn test_get_filtered_tasks_by_status() {
        let mut app = create_test_app();
        app.search_query = "RUNNING".to_string();
        let filtered = app.get_filtered_tasks();
        // Should match 2 tasks: one with status=RUNNING and one with desired_status=RUNNING
        assert_eq!(filtered.len(), 2);
        assert!(filtered.iter().any(|t| t.task_id == "task-abc123"));
        assert!(filtered.iter().any(|t| t.task_id == "task-def456"));
    }

    #[test]
    fn test_get_filtered_tasks_by_desired_status() {
        let mut app = create_test_app();
        app.search_query = "STOPPED".to_string();
        let filtered = app.get_filtered_tasks();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].task_id, "task-ghi789");
    }

    // Test navigation
    #[test]
    fn test_next_wraps_around() {
        let mut app = create_test_app();
        app.state = AppState::Clusters;
        app.selected_index = 2; // Last item
        app.next();
        assert_eq!(app.selected_index, 0); // Should wrap to first
    }

    #[test]
    fn test_next_increments() {
        let mut app = create_test_app();
        app.state = AppState::Clusters;
        app.selected_index = 0;
        app.next();
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_previous_wraps_around() {
        let mut app = create_test_app();
        app.state = AppState::Clusters;
        app.selected_index = 0; // First item
        app.previous();
        assert_eq!(app.selected_index, 2); // Should wrap to last
    }

    #[test]
    fn test_previous_decrements() {
        let mut app = create_test_app();
        app.state = AppState::Clusters;
        app.selected_index = 2;
        app.previous();
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_next_in_logs_scrolls_down() {
        let mut app = create_test_app();
        app.state = AppState::Logs;
        app.logs = vec![
            LogEntry {
                timestamp: 1000,
                message: "log1".to_string(),
                container_name: "container1".to_string(),
            },
            LogEntry {
                timestamp: 2000,
                message: "log2".to_string(),
                container_name: "container1".to_string(),
            },
        ];
        app.log_scroll = 0;
        app.auto_tail = true;

        app.next();

        assert_eq!(app.log_scroll, 1);
        assert_eq!(app.auto_tail, false);
    }

    #[test]
    fn test_previous_in_logs_scrolls_up() {
        let mut app = create_test_app();
        app.state = AppState::Logs;
        app.log_scroll = 5;
        app.auto_tail = true;

        app.previous();

        assert_eq!(app.log_scroll, 4);
        assert_eq!(app.auto_tail, false);
    }

    #[test]
    fn test_previous_in_logs_saturates_at_zero() {
        let mut app = create_test_app();
        app.state = AppState::Logs;
        app.log_scroll = 0;

        app.previous();

        assert_eq!(app.log_scroll, 0);
    }

    // Test state transitions
    #[test]
    fn test_set_view_changes_state() {
        let mut app = create_test_app();
        app.state = AppState::Clusters;
        app.selected_index = 5;

        app.set_view(AppState::Services);

        assert_eq!(app.state, AppState::Services);
        assert_eq!(app.previous_state, Some(AppState::Clusters));
        assert_eq!(app.selected_index, 0); // Should reset index
    }

    #[test]
    fn test_back_from_services_to_clusters() {
        let mut app = create_test_app();
        app.state = AppState::Services;
        app.selected_service = Some("test-service".to_string());

        app.back();

        assert_eq!(app.state, AppState::Clusters);
        assert_eq!(app.selected_service, None);
    }

    #[test]
    fn test_back_from_tasks_to_services() {
        let mut app = create_test_app();
        app.state = AppState::Tasks;

        app.back();

        assert_eq!(app.state, AppState::Services);
    }

    #[test]
    fn test_back_from_details_to_tasks() {
        let mut app = create_test_app();
        app.state = AppState::Details;
        app.details = Some("test details".to_string());

        app.back();

        assert_eq!(app.state, AppState::Tasks);
        assert_eq!(app.details, None);
    }

    #[test]
    fn test_back_from_logs_to_tasks() {
        let mut app = create_test_app();
        app.state = AppState::Logs;
        app.logs = vec![
            LogEntry {
                timestamp: 1000,
                message: "test".to_string(),
                container_name: "container1".to_string(),
            },
        ];
        app.log_scroll = 5;
        app.auto_tail = false;

        app.back();

        assert_eq!(app.state, AppState::Tasks);
        assert_eq!(app.logs.len(), 0);
        assert_eq!(app.log_scroll, 0);
        assert_eq!(app.auto_tail, true);
    }

    #[test]
    fn test_back_from_clusters_does_nothing() {
        let mut app = create_test_app();
        app.state = AppState::Clusters;

        app.back();

        assert_eq!(app.state, AppState::Clusters);
    }

    // Test auto-tail toggle
    #[test]
    fn test_toggle_auto_tail_enables() {
        let mut app = create_test_app();
        app.auto_tail = false;
        app.logs = vec![
            LogEntry {
                timestamp: 1000,
                message: "log1".to_string(),
                container_name: "container1".to_string(),
            },
            LogEntry {
                timestamp: 2000,
                message: "log2".to_string(),
                container_name: "container1".to_string(),
            },
        ];

        app.toggle_auto_tail();

        assert_eq!(app.auto_tail, true);
        assert_eq!(app.log_scroll, 1); // Should scroll to last log (len - 1)
        assert!(app.status_message.contains("enabled"));
    }

    #[test]
    fn test_toggle_auto_tail_disables() {
        let mut app = create_test_app();
        app.auto_tail = true;

        app.toggle_auto_tail();

        assert_eq!(app.auto_tail, false);
        assert!(app.status_message.contains("disabled"));
    }

    #[test]
    fn test_toggle_auto_tail_with_empty_logs() {
        let mut app = create_test_app();
        app.auto_tail = false;
        app.logs = vec![];

        app.toggle_auto_tail();

        assert_eq!(app.auto_tail, true);
        // Should not panic with empty logs
    }

    // Test search mode
    #[test]
    fn test_enter_search_mode() {
        let mut app = create_test_app();
        app.search_mode = false;
        app.search_query = "old query".to_string();
        app.selected_index = 5;

        app.enter_search_mode();

        assert_eq!(app.search_mode, true);
        assert_eq!(app.search_query, "");
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_exit_search_mode() {
        let mut app = create_test_app();
        app.search_mode = true;

        app.exit_search_mode();

        assert_eq!(app.search_mode, false);
    }

    #[test]
    fn test_clear_search() {
        let mut app = create_test_app();
        app.search_mode = true;
        app.search_query = "test query".to_string();
        app.selected_index = 5;

        app.clear_search();

        assert_eq!(app.search_mode, false);
        assert_eq!(app.search_query, "");
        assert_eq!(app.selected_index, 0);
    }

    #[test]
    fn test_update_search() {
        let mut app = create_test_app();
        app.search_query = "test".to_string();
        app.selected_index = 5;

        app.update_search('!');

        assert_eq!(app.search_query, "test!");
        assert_eq!(app.selected_index, 0); // Should reset index
    }

    #[test]
    fn test_update_search_multiple_chars() {
        let mut app = create_test_app();
        app.search_query = String::new();

        app.update_search('h');
        app.update_search('e');
        app.update_search('l');
        app.update_search('l');
        app.update_search('o');

        assert_eq!(app.search_query, "hello");
    }

    #[test]
    fn test_delete_search_char() {
        let mut app = create_test_app();
        app.search_query = "test".to_string();
        app.selected_index = 5;

        app.delete_search_char();

        assert_eq!(app.search_query, "tes");
        assert_eq!(app.selected_index, 0); // Should reset index
    }

    #[test]
    fn test_delete_search_char_empty() {
        let mut app = create_test_app();
        app.search_query = String::new();

        app.delete_search_char();

        assert_eq!(app.search_query, "");
        // Should not panic with empty string
    }

    // Test help toggle
    #[test]
    fn test_toggle_help() {
        let mut app = create_test_app();
        app.show_help = false;

        app.toggle_help();
        assert_eq!(app.show_help, true);

        app.toggle_help();
        assert_eq!(app.show_help, false);
    }

    // Test should_refresh
    #[test]
    fn test_should_refresh_logs_state() {
        let mut app = create_test_app();
        app.state = AppState::Logs;
        app.auto_tail = true;
        app.last_refresh = Instant::now() - Duration::from_secs(6);

        assert_eq!(app.should_refresh(), true);
    }

    #[test]
    fn test_should_refresh_logs_state_not_yet() {
        let mut app = create_test_app();
        app.state = AppState::Logs;
        app.auto_tail = true;
        app.last_refresh = Instant::now() - Duration::from_secs(3);

        assert_eq!(app.should_refresh(), false);
    }

    #[test]
    fn test_should_refresh_other_state() {
        let mut app = create_test_app();
        app.state = AppState::Clusters;
        app.last_refresh = Instant::now() - Duration::from_secs(31);

        assert_eq!(app.should_refresh(), true);
    }

    #[test]
    fn test_should_refresh_other_state_not_yet() {
        let mut app = create_test_app();
        app.state = AppState::Services;
        app.last_refresh = Instant::now() - Duration::from_secs(20);

        assert_eq!(app.should_refresh(), false);
    }

    #[test]
    fn test_should_refresh_disabled_in_config() {
        let mut app = create_test_app();
        app.config.behavior.auto_refresh = false;
        app.last_refresh = Instant::now() - Duration::from_secs(100);

        assert_eq!(app.should_refresh(), false);
    }

    // Test edge cases
    #[test]
    fn test_next_with_empty_list() {
        let mut app = create_test_app();
        app.state = AppState::Clusters;
        app.clusters = vec![];
        app.selected_index = 0;

        app.next();

        assert_eq!(app.selected_index, 0); // Should stay at 0
    }

    #[test]
    fn test_previous_with_empty_list() {
        let mut app = create_test_app();
        app.state = AppState::Services;
        app.services = vec![];
        app.selected_index = 0;

        app.previous();

        assert_eq!(app.selected_index, 0); // Should stay at 0
    }

    #[test]
    fn test_search_with_special_characters() {
        let mut app = create_test_app();
        app.clusters = vec![
            "cluster-prod-1".to_string(),
            "cluster_dev_2".to_string(),
            "cluster.staging.3".to_string(),
        ];
        app.search_query = "-".to_string();

        let filtered = app.get_filtered_clusters();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "cluster-prod-1");
    }

    #[test]
    fn test_search_with_underscore() {
        let mut app = create_test_app();
        app.clusters = vec![
            "cluster-prod-1".to_string(),
            "cluster_dev_2".to_string(),
            "cluster.staging.3".to_string(),
        ];
        app.search_query = "_".to_string();

        let filtered = app.get_filtered_clusters();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "cluster_dev_2");
    }

    #[test]
    fn test_search_with_dot() {
        let mut app = create_test_app();
        app.clusters = vec![
            "cluster-prod-1".to_string(),
            "cluster_dev_2".to_string(),
            "cluster.staging.3".to_string(),
        ];
        app.search_query = ".".to_string();

        let filtered = app.get_filtered_clusters();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0], "cluster.staging.3");
    }

    // Test ServiceInfo and TaskInfo structures
    #[test]
    fn test_service_info_clone() {
        let service = ServiceInfo {
            name: "test".to_string(),
            status: "ACTIVE".to_string(),
            desired_count: 3,
            running_count: 3,
            pending_count: 0,
            launch_type: "FARGATE".to_string(),
        };

        let cloned = service.clone();
        assert_eq!(service.name, cloned.name);
        assert_eq!(service.status, cloned.status);
        assert_eq!(service.desired_count, cloned.desired_count);
    }

    #[test]
    fn test_task_info_clone() {
        let task = TaskInfo {
            task_arn: "arn:test".to_string(),
            task_id: "id123".to_string(),
            status: "RUNNING".to_string(),
            desired_status: "RUNNING".to_string(),
            container_instance: "instance-1".to_string(),
            cpu: "256".to_string(),
            memory: "512".to_string(),
        };

        let cloned = task.clone();
        assert_eq!(task.task_arn, cloned.task_arn);
        assert_eq!(task.task_id, cloned.task_id);
    }

    #[test]
    fn test_log_entry_clone() {
        let log = LogEntry {
            timestamp: 12345,
            message: "test message".to_string(),
            container_name: "container1".to_string(),
        };

        let cloned = log.clone();
        assert_eq!(log.timestamp, cloned.timestamp);
        assert_eq!(log.message, cloned.message);
        assert_eq!(log.container_name, cloned.container_name);
    }

    #[test]
    fn test_app_state_equality() {
        assert_eq!(AppState::Clusters, AppState::Clusters);
        assert_ne!(AppState::Clusters, AppState::Services);
        assert_eq!(AppState::Logs, AppState::Logs);
    }

    #[test]
    fn test_app_state_clone() {
        let state = AppState::Tasks;
        let cloned = state.clone();
        assert_eq!(state, cloned);
    }
}
