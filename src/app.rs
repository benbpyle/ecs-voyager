use anyhow::Result;
use std::time::{Duration, Instant};

use crate::aws::EcsClient;

#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    Clusters,
    Services,
    Tasks,
    Details,
    Logs,
}

pub struct App {
    pub state: AppState,
    pub previous_state: Option<AppState>,
    pub show_help: bool,
    pub selected_index: usize,
    pub ecs_client: EcsClient,

    // Data
    pub clusters: Vec<String>,
    pub services: Vec<ServiceInfo>,
    pub tasks: Vec<TaskInfo>,
    pub selected_cluster: Option<String>,
    pub selected_service: Option<String>,
    pub selected_task: Option<TaskInfo>,
    pub details: Option<String>,
    pub logs: Vec<LogEntry>,
    pub log_scroll: usize,
    pub auto_tail: bool,

    // Search
    pub search_mode: bool,
    pub search_query: String,

    // Status
    pub status_message: String,
    pub loading: bool,
    pub last_refresh: Instant,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub status: String,
    pub desired_count: i32,
    pub running_count: i32,
    pub pending_count: i32,
    pub launch_type: String,
}

#[derive(Debug, Clone)]
pub struct TaskInfo {
    pub task_arn: String,
    pub task_id: String,
    pub status: String,
    pub desired_status: String,
    pub container_instance: String,
    pub cpu: String,
    pub memory: String,
}

#[derive(Debug, Clone)]
pub struct LogEntry {
    pub timestamp: i64,
    pub message: String,
    pub container_name: String,
}

impl App {
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
