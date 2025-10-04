use anyhow::Result;
use aws_sdk_ecs::Client;
use aws_sdk_cloudwatchlogs::Client as LogsClient;
use crate::app::{ServiceInfo, TaskInfo, LogEntry};

pub struct EcsClient {
    client: Client,
    logs_client: LogsClient,
}

impl EcsClient {
    /// Creates a new ECS client with optional region and profile configuration.
    ///
    /// # Arguments
    /// * `region` - Optional AWS region override (e.g., "us-east-1")
    /// * `profile` - Optional AWS profile name from ~/.aws/credentials
    ///
    /// # Returns
    /// Returns a new `EcsClient` instance configured with the specified options,
    /// or an error if AWS SDK initialization fails.
    ///
    /// # Errors
    /// This function will return an error if:
    /// - AWS credentials cannot be resolved
    /// - The specified profile doesn't exist
    /// - The specified region is invalid
    pub async fn new(region: Option<String>, profile: Option<String>) -> Result<Self> {
        let mut config_loader = aws_config::from_env();

        // Set region if provided
        if let Some(region_str) = region {
            config_loader = config_loader.region(
                aws_config::Region::new(region_str)
            );
        }

        // Set profile if provided
        if let Some(profile_name) = profile {
            config_loader = config_loader.profile_name(profile_name);
        }

        let config = config_loader.load().await;
        let client = Client::new(&config);
        let logs_client = LogsClient::new(&config);
        Ok(Self { client, logs_client })
    }

    pub async fn list_clusters(&self) -> Result<Vec<String>> {
        let resp = self.client.list_clusters().send().await?;

        let cluster_arns = resp.cluster_arns();
        let clusters = cluster_arns
            .iter()
            .map(|arn| {
                // Extract cluster name from ARN
                arn.split('/').last().unwrap_or(arn).to_string()
            })
            .collect();

        Ok(clusters)
    }

    pub async fn list_services(&self, cluster: &str) -> Result<Vec<ServiceInfo>> {
        let resp = self.client
            .list_services()
            .cluster(cluster)
            .send()
            .await?;

        let service_arns = resp.service_arns();

        if service_arns.is_empty() {
            return Ok(Vec::new());
        }

        // Describe services to get detailed info
        let describe_resp = self.client
            .describe_services()
            .cluster(cluster)
            .set_services(Some(service_arns.to_vec()))
            .send()
            .await?;

        let services = describe_resp
            .services()
            .iter()
            .map(|s| {
                let name = s.service_name().unwrap_or("unknown").to_string();
                let status = s.status().unwrap_or("unknown").to_string();
                let desired_count = s.desired_count();
                let running_count = s.running_count();
                let pending_count = s.pending_count();
                let launch_type = s.launch_type()
                    .map(|lt| lt.as_str().to_string())
                    .unwrap_or_else(|| "unknown".to_string());

                ServiceInfo {
                    name,
                    status,
                    desired_count,
                    running_count,
                    pending_count,
                    launch_type,
                }
            })
            .collect();

        Ok(services)
    }

    pub async fn list_tasks(&self, cluster: &str, service: &str) -> Result<Vec<TaskInfo>> {
        let resp = self.client
            .list_tasks()
            .cluster(cluster)
            .service_name(service)
            .send()
            .await?;

        let task_arns = resp.task_arns();

        if task_arns.is_empty() {
            return Ok(Vec::new());
        }

        // Describe tasks to get detailed info
        let describe_resp = self.client
            .describe_tasks()
            .cluster(cluster)
            .set_tasks(Some(task_arns.to_vec()))
            .send()
            .await?;

        let tasks = describe_resp
            .tasks()
            .iter()
            .map(|t| {
                let task_arn = t.task_arn().unwrap_or("unknown").to_string();
                let task_id = task_arn.split('/').last().unwrap_or("unknown").to_string();
                let status = t.last_status().unwrap_or("unknown").to_string();
                let desired_status = t.desired_status().unwrap_or("unknown").to_string();
                let container_instance = t.container_instance_arn()
                    .and_then(|ci| ci.split('/').last())
                    .unwrap_or("none")
                    .to_string();
                let cpu = t.cpu().unwrap_or("unknown").to_string();
                let memory = t.memory().unwrap_or("unknown").to_string();

                TaskInfo {
                    task_arn,
                    task_id,
                    status,
                    desired_status,
                    container_instance,
                    cpu,
                    memory,
                }
            })
            .collect();

        Ok(tasks)
    }

    pub async fn describe_service(&self, cluster: &str, service: &str) -> Result<String> {
        let resp = self.client
            .describe_services()
            .cluster(cluster)
            .services(service)
            .send()
            .await?;

        // Format the response manually since AWS types don't implement Serialize
        let mut output = String::new();
        output.push_str(&format!("Cluster: {}\n\n", cluster));

        for svc in resp.services() {
            output.push_str(&format!("Service Name: {}\n", svc.service_name().unwrap_or("N/A")));
            output.push_str(&format!("Service ARN: {}\n", svc.service_arn().unwrap_or("N/A")));
            output.push_str(&format!("Status: {}\n", svc.status().unwrap_or("N/A")));
            output.push_str(&format!("Desired Count: {}\n", svc.desired_count()));
            output.push_str(&format!("Running Count: {}\n", svc.running_count()));
            output.push_str(&format!("Pending Count: {}\n", svc.pending_count()));
            output.push_str(&format!("Launch Type: {}\n", svc.launch_type().map(|lt| lt.as_str()).unwrap_or("N/A")));
            output.push_str(&format!("Task Definition: {}\n", svc.task_definition().unwrap_or("N/A")));
            output.push_str("\n");
        }

        Ok(output)
    }

    pub async fn describe_task(&self, cluster: &str, task_arn: &str) -> Result<String> {
        let resp = self.client
            .describe_tasks()
            .cluster(cluster)
            .tasks(task_arn)
            .send()
            .await?;

        // Format the response manually since AWS types don't implement Serialize
        let mut output = String::new();
        output.push_str(&format!("Cluster: {}\n\n", cluster));

        for task in resp.tasks() {
            output.push_str(&format!("Task ARN: {}\n", task.task_arn().unwrap_or("N/A")));
            output.push_str(&format!("Task Definition: {}\n", task.task_definition_arn().unwrap_or("N/A")));
            output.push_str(&format!("Last Status: {}\n", task.last_status().unwrap_or("N/A")));
            output.push_str(&format!("Desired Status: {}\n", task.desired_status().unwrap_or("N/A")));
            output.push_str(&format!("CPU: {}\n", task.cpu().unwrap_or("N/A")));
            output.push_str(&format!("Memory: {}\n", task.memory().unwrap_or("N/A")));
            output.push_str(&format!("Launch Type: {}\n", task.launch_type().map(|lt| lt.as_str()).unwrap_or("N/A")));

            output.push_str("\nContainers:\n");
            for container in task.containers() {
                output.push_str(&format!("  - Name: {}\n", container.name().unwrap_or("N/A")));
                output.push_str(&format!("    Image: {}\n", container.image().unwrap_or("N/A")));
                output.push_str(&format!("    Last Status: {}\n", container.last_status().unwrap_or("N/A")));
                if let Some(exit_code) = container.exit_code() {
                    output.push_str(&format!("    Exit Code: {}\n", exit_code));
                }
            }
            output.push_str("\n");
        }

        Ok(output)
    }

    pub async fn restart_service(&self, cluster: &str, service: &str) -> Result<()> {
        self.client
            .update_service()
            .cluster(cluster)
            .service(service)
            .force_new_deployment(true)
            .send()
            .await?;

        Ok(())
    }

    pub async fn stop_task(&self, cluster: &str, task_arn: &str) -> Result<()> {
        self.client
            .stop_task()
            .cluster(cluster)
            .task(task_arn)
            .reason("Stopped via ecs-voyager")
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_task_logs(&self, cluster: &str, task_arn: &str) -> Result<Vec<LogEntry>> {
        // First, describe the task to get the task definition and container details
        let task_resp = self.client
            .describe_tasks()
            .cluster(cluster)
            .tasks(task_arn)
            .send()
            .await?;

        let mut all_logs = Vec::new();

        if let Some(task) = task_resp.tasks().first() {
            // Get the task definition to find log configuration
            if let Some(task_def_arn) = task.task_definition_arn() {
                let task_def_resp = self.client
                    .describe_task_definition()
                    .task_definition(task_def_arn)
                    .send()
                    .await?;

                if let Some(task_definition) = task_def_resp.task_definition() {
                    // Extract task ID from ARN for log stream name
                    let task_id = task_arn.split('/').last().unwrap_or(task_arn);

                    // Iterate through containers to get logs from each
                    for container_def in task_definition.container_definitions() {
                        let container_name = container_def.name().unwrap_or("unknown");

                        // Check if container has CloudWatch Logs configured
                        if let Some(log_config) = container_def.log_configuration() {
                            if log_config.log_driver().as_str() == "awslogs" {
                                if let Some(options) = log_config.options() {
                                    // Get log group and stream prefix
                                    if let Some(log_group) = options.get("awslogs-group") {
                                        let stream_prefix = options
                                            .get("awslogs-stream-prefix")
                                            .map(|s| s.as_str())
                                            .unwrap_or("ecs");

                                        // Construct log stream name
                                        let log_stream = format!("{}/{}/{}", stream_prefix, container_name, task_id);

                                        // Fetch logs from CloudWatch Logs
                                        match self.fetch_logs_from_stream(log_group, &log_stream, container_name).await {
                                            Ok(mut logs) => all_logs.append(&mut logs),
                                            Err(e) => {
                                                // Log stream might not exist yet or other error - continue with other containers
                                                eprintln!("Failed to fetch logs for container {}: {}", container_name, e);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Sort logs by timestamp
        all_logs.sort_by_key(|log| log.timestamp);

        Ok(all_logs)
    }

    async fn fetch_logs_from_stream(
        &self,
        log_group: &str,
        log_stream: &str,
        container_name: &str,
    ) -> Result<Vec<LogEntry>> {
        let mut logs = Vec::new();

        // Get the last 100 log events (you can adjust this or add pagination)
        let resp = self.logs_client
            .get_log_events()
            .log_group_name(log_group)
            .log_stream_name(log_stream)
            .limit(100)
            .start_from_head(false) // Get most recent logs first
            .send()
            .await?;

        for event in resp.events() {
            if let (Some(timestamp), Some(message)) = (event.timestamp(), event.message()) {
                logs.push(LogEntry {
                    timestamp,
                    message: message.to_string(),
                    container_name: container_name.to_string(),
                });
            }
        }

        Ok(logs)
    }
}
