//! AWS ECS and CloudWatch Logs integration module.
//!
//! This module provides a client wrapper for AWS ECS and CloudWatch Logs services,
//! with methods for listing clusters, services, tasks, and retrieving logs.

use crate::app::{LogEntry, ServiceInfo, TaskInfo};
use anyhow::{Context, Result};
use aws_sdk_cloudwatch::Client as CloudWatchClient;
use aws_sdk_cloudwatchlogs::Client as LogsClient;
use aws_sdk_ecs::Client;

/// Client for interacting with AWS ECS, CloudWatch Logs, and CloudWatch Metrics.
///
/// Wraps the AWS SDK clients and provides convenient methods for common operations
/// used by the TUI application.
pub struct EcsClient {
    /// AWS ECS SDK client
    client: Client,
    /// AWS CloudWatch Logs SDK client
    logs_client: LogsClient,
    /// AWS CloudWatch Metrics SDK client
    metrics_client: CloudWatchClient,
}

/// Represents a CloudWatch metric datapoint.
#[derive(Debug, Clone)]
pub struct MetricDatapoint {
    /// Timestamp of the datapoint
    pub timestamp: i64,
    /// Average value
    pub average: Option<f64>,
    /// Maximum value
    pub maximum: Option<f64>,
    /// Minimum value
    #[allow(dead_code)]
    pub minimum: Option<f64>,
    /// Sum of values
    #[allow(dead_code)]
    pub sum: Option<f64>,
    /// Sample count
    #[allow(dead_code)]
    pub sample_count: Option<f64>,
}

/// Represents a CloudWatch alarm.
#[derive(Debug, Clone)]
pub struct CloudWatchAlarm {
    /// Alarm name
    pub name: String,
    /// Alarm description
    pub description: Option<String>,
    /// Current state (OK, ALARM, INSUFFICIENT_DATA)
    pub state: String,
    /// State reason (why alarm is in this state)
    pub state_reason: Option<String>,
    /// Metric name this alarm monitors
    pub metric_name: String,
}

/// Time range options for metrics display.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TimeRange {
    /// Last 1 hour (60 minutes)
    OneHour,
    /// Last 6 hours (360 minutes)
    SixHours,
    /// Last 24 hours (1440 minutes)
    OneDay,
    /// Last 7 days (10080 minutes)
    SevenDays,
}

impl TimeRange {
    /// Returns the number of minutes for this time range.
    pub fn minutes(&self) -> i32 {
        match self {
            TimeRange::OneHour => 60,
            TimeRange::SixHours => 360,
            TimeRange::OneDay => 1440,
            TimeRange::SevenDays => 10080,
        }
    }

    /// Returns a human-readable label for this time range.
    pub fn label(&self) -> &'static str {
        match self {
            TimeRange::OneHour => "1h",
            TimeRange::SixHours => "6h",
            TimeRange::OneDay => "24h",
            TimeRange::SevenDays => "7d",
        }
    }

    /// Returns the next time range in the cycle.
    pub fn next(&self) -> TimeRange {
        match self {
            TimeRange::OneHour => TimeRange::SixHours,
            TimeRange::SixHours => TimeRange::OneDay,
            TimeRange::OneDay => TimeRange::SevenDays,
            TimeRange::SevenDays => TimeRange::OneHour,
        }
    }

    /// Converts minutes to the closest TimeRange variant.
    pub fn from_minutes(minutes: i32) -> TimeRange {
        if minutes <= 60 {
            TimeRange::OneHour
        } else if minutes <= 360 {
            TimeRange::SixHours
        } else if minutes <= 1440 {
            TimeRange::OneDay
        } else {
            TimeRange::SevenDays
        }
    }
}

/// Container for service or task metrics.
#[derive(Debug, Clone)]
pub struct Metrics {
    /// CPU utilization percentage datapoints
    pub cpu_datapoints: Vec<MetricDatapoint>,
    /// Memory utilization percentage datapoints
    pub memory_datapoints: Vec<MetricDatapoint>,
    /// CloudWatch alarms related to this service
    pub alarms: Vec<CloudWatchAlarm>,
    /// Time range for these metrics
    pub time_range: TimeRange,
    /// Cluster name
    pub cluster_name: String,
    /// Service name
    pub service_name: String,
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
            config_loader = config_loader.region(aws_config::Region::new(region_str));
        }

        // Set profile if provided
        if let Some(profile_name) = profile {
            config_loader = config_loader.profile_name(profile_name);
        }

        let config = config_loader.load().await;
        let client = Client::new(&config);
        let logs_client = LogsClient::new(&config);
        let metrics_client = CloudWatchClient::new(&config);
        Ok(Self {
            client,
            logs_client,
            metrics_client,
        })
    }

    /// Lists all ECS clusters in the configured region.
    ///
    /// Returns cluster names extracted from the full ARNs. If no clusters exist,
    /// returns an empty vector.
    ///
    /// # Returns
    /// A vector of cluster names (not full ARNs)
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS API call fails due to network or permission issues
    /// - The response cannot be parsed
    pub async fn list_clusters(&self) -> Result<Vec<String>> {
        let resp = self.client.list_clusters().send().await?;

        let cluster_arns = resp.cluster_arns();
        let clusters = cluster_arns
            .iter()
            .map(|arn| {
                // Extract cluster name from ARN
                arn.split('/').next_back().unwrap_or(arn).to_string()
            })
            .collect();

        Ok(clusters)
    }

    /// Lists all services in a specific ECS cluster.
    ///
    /// First retrieves service ARNs, then fetches detailed information for each service
    /// including status, task counts, and launch type.
    ///
    /// # Arguments
    /// * `cluster` - The cluster name or ARN
    ///
    /// # Returns
    /// A vector of `ServiceInfo` structs containing service details, or an empty vector
    /// if no services exist in the cluster
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS ListServices or DescribeServices API calls fail
    /// - The cluster doesn't exist
    /// - Insufficient permissions to access the cluster or services
    pub async fn list_services(&self, cluster: &str) -> Result<Vec<ServiceInfo>> {
        let resp = self.client.list_services().cluster(cluster).send().await?;

        let service_arns = resp.service_arns();

        if service_arns.is_empty() {
            return Ok(Vec::new());
        }

        // Describe services to get detailed info
        let describe_resp = self
            .client
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
                let launch_type = s
                    .launch_type()
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

    /// Lists all tasks for a specific service in a cluster.
    ///
    /// Retrieves task ARNs for the service, then fetches detailed information including
    /// status, resource allocation, and container instance details.
    ///
    /// # Arguments
    /// * `cluster` - The cluster name or ARN
    /// * `service` - The service name or ARN
    ///
    /// # Returns
    /// A vector of `TaskInfo` structs containing task details, or an empty vector
    /// if no tasks are running for the service
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS ListTasks or DescribeTasks API calls fail
    /// - The cluster or service doesn't exist
    /// - Insufficient permissions to access tasks
    pub async fn list_tasks(&self, cluster: &str, service: &str) -> Result<Vec<TaskInfo>> {
        let resp = self
            .client
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
        let describe_resp = self
            .client
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
                let task_id = task_arn
                    .split('/')
                    .next_back()
                    .unwrap_or("unknown")
                    .to_string();
                let status = t.last_status().unwrap_or("unknown").to_string();
                let desired_status = t.desired_status().unwrap_or("unknown").to_string();
                let container_instance = t
                    .container_instance_arn()
                    .and_then(|ci| ci.split('/').next_back())
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

    /// Retrieves detailed information about a specific service.
    ///
    /// Fetches comprehensive service details including ARN, status, task counts,
    /// launch type, and task definition ARN, formatting them as human-readable text.
    ///
    /// # Arguments
    /// * `cluster` - The cluster name or ARN
    /// * `service` - The service name or ARN
    ///
    /// # Returns
    /// A formatted string containing service details for display in the UI
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS DescribeServices API call fails
    /// - The service doesn't exist in the specified cluster
    /// - Insufficient permissions to describe the service
    pub async fn describe_service(&self, cluster: &str, service: &str) -> Result<(String, String)> {
        let resp = self
            .client
            .describe_services()
            .cluster(cluster)
            .services(service)
            .send()
            .await?;

        // Format the response manually since AWS types don't implement Serialize
        let mut output = String::new();
        output.push_str(&format!("Cluster: {cluster}\n\n"));

        let mut json_parts = Vec::new();

        for svc in resp.services() {
            // Formatted text view
            output.push_str(&format!(
                "Service Name: {}\n",
                svc.service_name().unwrap_or("N/A")
            ));
            output.push_str(&format!(
                "Service ARN: {}\n",
                svc.service_arn().unwrap_or("N/A")
            ));
            output.push_str(&format!("Status: {}\n", svc.status().unwrap_or("N/A")));
            output.push_str(&format!("Desired Count: {}\n", svc.desired_count()));
            output.push_str(&format!("Running Count: {}\n", svc.running_count()));
            output.push_str(&format!("Pending Count: {}\n", svc.pending_count()));
            output.push_str(&format!(
                "Launch Type: {}\n",
                svc.launch_type().map(|lt| lt.as_str()).unwrap_or("N/A")
            ));
            output.push_str(&format!(
                "Task Definition: {}\n",
                svc.task_definition().unwrap_or("N/A")
            ));

            // Add deployment info
            let deployments = svc.deployments();
            if !deployments.is_empty() {
                output.push_str("\nDeployments:\n");
                for deployment in deployments {
                    output.push_str(&format!("  - Status: {}\n", deployment.status().unwrap_or("N/A")));
                    output.push_str(&format!("    Running Count: {}\n", deployment.running_count()));
                    output.push_str(&format!("    Desired Count: {}\n", deployment.desired_count()));
                }
            }

            // Add network configuration
            if let Some(network_config) = svc.network_configuration() {
                output.push_str("\nNetwork Configuration:\n");
                if let Some(awsvpc) = network_config.awsvpc_configuration() {
                    output.push_str(&format!("  Subnets: {}\n", awsvpc.subnets().join(", ")));
                    output.push_str(&format!("  Assign Public IP: {}\n",
                        awsvpc.assign_public_ip().map(|ip| ip.as_str()).unwrap_or("N/A")));
                    let sgs = awsvpc.security_groups();
                    if !sgs.is_empty() {
                        output.push_str(&format!("  Security Groups: {}\n", sgs.join(", ")));
                    }
                }
            }

            output.push('\n');

            // Build JSON representation
            json_parts.push(format!(r#"{{
  "serviceName": "{}",
  "serviceArn": "{}",
  "status": "{}",
  "desiredCount": {},
  "runningCount": {},
  "pendingCount": {},
  "launchType": "{}",
  "taskDefinition": "{}"
}}"#,
                svc.service_name().unwrap_or("N/A"),
                svc.service_arn().unwrap_or("N/A"),
                svc.status().unwrap_or("N/A"),
                svc.desired_count(),
                svc.running_count(),
                svc.pending_count(),
                svc.launch_type().map(|lt| lt.as_str()).unwrap_or("N/A"),
                svc.task_definition().unwrap_or("N/A")
            ));
        }

        let json_output = format!("[{}]", json_parts.join(",\n"));
        Ok((output, json_output))
    }

    /// Retrieves detailed information about a specific task.
    ///
    /// Fetches comprehensive task details including ARN, task definition, status,
    /// resource allocation, launch type, and container information, formatting them
    /// as human-readable text.
    ///
    /// # Arguments
    /// * `cluster` - The cluster name or ARN
    /// * `task_arn` - The full task ARN
    ///
    /// # Returns
    /// A formatted string containing task and container details for display in the UI
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS DescribeTasks API call fails
    /// - The task doesn't exist in the specified cluster
    /// - Insufficient permissions to describe the task
    pub async fn describe_task(&self, cluster: &str, task_arn: &str) -> Result<(String, String)> {
        let resp = self
            .client
            .describe_tasks()
            .cluster(cluster)
            .tasks(task_arn)
            .send()
            .await?;

        // Format the response manually since AWS types don't implement Serialize
        let mut output = String::new();
        output.push_str(&format!("Cluster: {cluster}\n\n"));

        let mut json_parts = Vec::new();

        for task in resp.tasks() {
            output.push_str(&format!("Task ARN: {}\n", task.task_arn().unwrap_or("N/A")));
            output.push_str(&format!(
                "Task Definition: {}\n",
                task.task_definition_arn().unwrap_or("N/A")
            ));
            output.push_str(&format!(
                "Last Status: {}\n",
                task.last_status().unwrap_or("N/A")
            ));
            output.push_str(&format!(
                "Desired Status: {}\n",
                task.desired_status().unwrap_or("N/A")
            ));
            output.push_str(&format!("CPU: {}\n", task.cpu().unwrap_or("N/A")));
            output.push_str(&format!("Memory: {}\n", task.memory().unwrap_or("N/A")));
            output.push_str(&format!(
                "Launch Type: {}\n",
                task.launch_type().map(|lt| lt.as_str()).unwrap_or("N/A")
            ));

            output.push_str("\nContainers:\n");
            let mut container_jsons = Vec::new();
            for container in task.containers() {
                output.push_str(&format!(
                    "  - Name: {}\n",
                    container.name().unwrap_or("N/A")
                ));
                output.push_str(&format!(
                    "    Image: {}\n",
                    container.image().unwrap_or("N/A")
                ));
                output.push_str(&format!(
                    "    Last Status: {}\n",
                    container.last_status().unwrap_or("N/A")
                ));
                if let Some(exit_code) = container.exit_code() {
                    output.push_str(&format!("    Exit Code: {exit_code}\n"));
                }

                // Build container JSON
                let exit_code_json = container.exit_code().map(|ec| format!(r#", "exitCode": {ec}"#)).unwrap_or_default();
                container_jsons.push(format!(r#"{{
      "name": "{}",
      "image": "{}",
      "lastStatus": "{}"{exit_code_json}
    }}"#,
                    container.name().unwrap_or("N/A"),
                    container.image().unwrap_or("N/A"),
                    container.last_status().unwrap_or("N/A")
                ));
            }
            output.push('\n');

            // Build task JSON
            json_parts.push(format!(r#"{{
  "taskArn": "{}",
  "taskDefinitionArn": "{}",
  "lastStatus": "{}",
  "desiredStatus": "{}",
  "cpu": "{}",
  "memory": "{}",
  "launchType": "{}",
  "containers": [
{}
  ]
}}"#,
                task.task_arn().unwrap_or("N/A"),
                task.task_definition_arn().unwrap_or("N/A"),
                task.last_status().unwrap_or("N/A"),
                task.desired_status().unwrap_or("N/A"),
                task.cpu().unwrap_or("N/A"),
                task.memory().unwrap_or("N/A"),
                task.launch_type().map(|lt| lt.as_str()).unwrap_or("N/A"),
                container_jsons.join(",\n")
            ));
        }

        let json_output = format!("[{}]", json_parts.join(",\n"));
        Ok((output, json_output))
    }

    /// Forces a new deployment of a service, restarting all tasks.
    ///
    /// Uses the UpdateService API with `force_new_deployment` set to true,
    /// which causes ECS to start new tasks and stop old ones, effectively
    /// restarting the service.
    ///
    /// # Arguments
    /// * `cluster` - The cluster name or ARN
    /// * `service` - The service name or ARN
    ///
    /// # Returns
    /// Returns `Ok(())` on success
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS UpdateService API call fails
    /// - The service is in a state that prevents updates
    /// - Insufficient permissions to update the service
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

    /// Stops a specific task in a cluster.
    ///
    /// Sends a stop request to ECS, which will terminate the task containers.
    /// The reason "Stopped via ecs-voyager" is included in the stop request for auditing.
    ///
    /// # Arguments
    /// * `cluster` - The cluster name or ARN
    /// * `task_arn` - The full task ARN
    ///
    /// # Returns
    /// Returns `Ok(())` on success
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS StopTask API call fails
    /// - The task doesn't exist or is already stopped
    /// - Insufficient permissions to stop tasks
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

    /// Retrieves CloudWatch Logs for all containers in a task.
    ///
    /// This method:
    /// 1. Describes the task to get the task definition ARN
    /// 2. Describes the task definition to get log configuration
    /// 3. For each container with awslogs configuration, fetches log events
    /// 4. Combines and sorts all logs by timestamp
    ///
    /// Only works with tasks that have CloudWatch Logs (awslogs) configured.
    ///
    /// # Arguments
    /// * `cluster` - The cluster name or ARN
    /// * `task_arn` - The full task ARN
    ///
    /// # Returns
    /// A vector of `LogEntry` structs sorted by timestamp, or an empty vector if
    /// no logs are available
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS DescribeTasks or DescribeTaskDefinition API calls fail
    /// - The task doesn't exist
    /// - CloudWatch Logs API calls fail (log streams not found are handled gracefully)
    /// - Insufficient permissions to access logs
    pub async fn get_task_logs(&self, cluster: &str, task_arn: &str) -> Result<Vec<LogEntry>> {
        // First, describe the task to get the task definition and container details
        let task_resp = self
            .client
            .describe_tasks()
            .cluster(cluster)
            .tasks(task_arn)
            .send()
            .await?;

        let mut all_logs = Vec::new();

        if let Some(task) = task_resp.tasks().first() {
            // Get the task definition to find log configuration
            if let Some(task_def_arn) = task.task_definition_arn() {
                let task_def_resp = self
                    .client
                    .describe_task_definition()
                    .task_definition(task_def_arn)
                    .send()
                    .await?;

                if let Some(task_definition) = task_def_resp.task_definition() {
                    // Extract task ID from ARN for log stream name
                    let task_id = task_arn.split('/').next_back().unwrap_or(task_arn);

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
                                        let log_stream =
                                            format!("{stream_prefix}/{container_name}/{task_id}");

                                        // Fetch logs from CloudWatch Logs
                                        match self
                                            .fetch_logs_from_stream(
                                                log_group,
                                                &log_stream,
                                                container_name,
                                            )
                                            .await
                                        {
                                            Ok(mut logs) => all_logs.append(&mut logs),
                                            Err(e) => {
                                                // Log stream might not exist yet or other error - continue with other containers
                                                eprintln!("Failed to fetch logs for container {container_name}: {e}");
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

    /// Fetches log events from a specific CloudWatch Logs stream.
    ///
    /// Retrieves the most recent 100 log events from the specified log stream.
    /// This is a helper method used by `get_task_logs`.
    ///
    /// # Arguments
    /// * `log_group` - The CloudWatch Logs group name
    /// * `log_stream` - The CloudWatch Logs stream name
    /// * `container_name` - The container name to associate with log entries
    ///
    /// # Returns
    /// A vector of `LogEntry` structs from this log stream
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS GetLogEvents API call fails
    /// - The log group or stream doesn't exist
    /// - Insufficient permissions to read logs
    async fn fetch_logs_from_stream(
        &self,
        log_group: &str,
        log_stream: &str,
        container_name: &str,
    ) -> Result<Vec<LogEntry>> {
        let mut logs = Vec::new();

        // Get the last 100 log events (you can adjust this or add pagination)
        let resp = self
            .logs_client
            .get_log_events()
            .log_group_name(log_group)
            .log_stream_name(log_stream)
            .limit(100)
            .start_from_head(false) // Get most recent logs first
            .send()
            .await?;

        for event in resp.events() {
            if let (Some(timestamp), Some(message)) = (event.timestamp(), event.message()) {
                logs.push(LogEntry::new(
                    timestamp,
                    message.to_string(),
                    container_name.to_string(),
                ));
            }
        }

        Ok(logs)
    }

    /// Fetches CloudWatch alarms for an ECS service.
    ///
    /// Retrieves alarms that monitor the specified ECS service. Searches for alarms
    /// with metric dimensions matching the service and cluster name.
    ///
    /// # Arguments
    /// * `cluster_name` - Name of the ECS cluster
    /// * `service_name` - Name of the ECS service
    ///
    /// # Returns
    /// Returns vector of `CloudWatchAlarm` structs
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS DescribeAlarms API call fails
    /// - Insufficient permissions to read alarms
    pub async fn get_service_alarms(
        &self,
        cluster_name: &str,
        service_name: &str,
    ) -> Result<Vec<CloudWatchAlarm>> {
        // Describe alarms for this service
        let response = self
            .metrics_client
            .describe_alarms()
            .send()
            .await?;

        let mut alarms = Vec::new();

        // Filter alarms that are related to this ECS service
        for alarm in response.metric_alarms() {
            // Check if alarm dimensions match our service
            let metrics = alarm.metrics();
            if !metrics.is_empty() {
                for metric_data in metrics {
                    if let Some(metric) = metric_data.metric_stat() {
                        if let Some(metric_obj) = metric.metric() {
                            // Check if this is an ECS metric for our service
                            let is_ecs_service_metric = metric_obj
                                .dimensions()
                                .iter()
                                .any(|dim| {
                                    (dim.name() == Some("ServiceName") && dim.value() == Some(service_name))
                                        || (dim.name() == Some("ClusterName") && dim.value() == Some(cluster_name))
                                });

                            if is_ecs_service_metric {
                                alarms.push(CloudWatchAlarm {
                                    name: alarm.alarm_name().unwrap_or("Unknown").to_string(),
                                    description: alarm.alarm_description().map(|s| s.to_string()),
                                    state: alarm
                                        .state_value()
                                        .map(|s| s.as_str().to_string())
                                        .unwrap_or_else(|| "UNKNOWN".to_string()),
                                    state_reason: alarm.state_reason().map(|s| s.to_string()),
                                    metric_name: metric_obj.metric_name().unwrap_or("Unknown").to_string(),
                                });
                                break;
                            }
                        }
                    }
                }
            }
        }

        Ok(alarms)
    }

    /// Fetches CloudWatch metrics for an ECS service.
    ///
    /// Retrieves CPU and Memory utilization metrics for the specified service
    /// over the configured time range, along with CloudWatch alarms.
    ///
    /// # Arguments
    /// * `cluster_name` - Name of the ECS cluster
    /// * `service_name` - Name of the ECS service
    /// * `time_range` - Time range for metrics (1h, 6h, 24h, 7d)
    ///
    /// # Returns
    /// Returns `Metrics` containing CPU/memory datapoints and alarms
    ///
    /// # Errors
    /// This function will return an error if:
    /// - The AWS GetMetricStatistics API call fails
    /// - Insufficient permissions to read metrics
    /// Helper function to create a CloudWatch Dimension with required name and value.
    ///
    /// Both name and value are required by the CloudWatch API, even though the SDK
    /// allows them to be optional. This helper ensures they are always set.
    fn create_dimension(name: &str, value: &str) -> aws_sdk_cloudwatch::types::Dimension {
        aws_sdk_cloudwatch::types::Dimension::builder()
            .name(name)
            .value(value)
            .build()
    }

    pub async fn get_service_metrics(
        &self,
        cluster_name: &str,
        service_name: &str,
        time_range: TimeRange,
    ) -> Result<Metrics> {
        use std::time::{SystemTime, UNIX_EPOCH};

        // Validate inputs
        if cluster_name.is_empty() {
            anyhow::bail!("Cluster name cannot be empty");
        }
        if service_name.is_empty() {
            anyhow::bail!("Service name cannot be empty");
        }

        let now = SystemTime::now();
        let end_time = now
            .duration_since(UNIX_EPOCH)
            .context("Failed to get current time")?
            .as_secs() as i64;
        let time_range_minutes = time_range.minutes();
        let start_time = end_time - (time_range_minutes as i64 * 60);

        eprintln!(
            "Fetching metrics for service: {} in cluster: {} (time range: {} minutes)",
            service_name, cluster_name, time_range_minutes
        );

        // Create dimensions once (both metrics use the same dimensions)
        let service_dimension = Self::create_dimension("ServiceName", service_name);
        let cluster_dimension = Self::create_dimension("ClusterName", cluster_name);

        // Fetch CPU utilization
        let cpu_response = self
            .metrics_client
            .get_metric_statistics()
            .namespace("AWS/ECS")
            .metric_name("CPUUtilization")
            .dimensions(service_dimension.clone())
            .dimensions(cluster_dimension.clone())
            .start_time(aws_sdk_cloudwatch::primitives::DateTime::from_secs(
                start_time,
            ))
            .end_time(aws_sdk_cloudwatch::primitives::DateTime::from_secs(
                end_time,
            ))
            .period(300) // 5 minute periods
            .statistics(aws_sdk_cloudwatch::types::Statistic::Average)
            .statistics(aws_sdk_cloudwatch::types::Statistic::Maximum)
            .send()
            .await
            .context("Failed to fetch CPU utilization metrics from CloudWatch")?;

        eprintln!(
            "Received {} CPU datapoints",
            cpu_response.datapoints().len()
        );

        // Fetch Memory utilization
        let memory_response = self
            .metrics_client
            .get_metric_statistics()
            .namespace("AWS/ECS")
            .metric_name("MemoryUtilization")
            .dimensions(service_dimension)
            .dimensions(cluster_dimension)
            .start_time(aws_sdk_cloudwatch::primitives::DateTime::from_secs(
                start_time,
            ))
            .end_time(aws_sdk_cloudwatch::primitives::DateTime::from_secs(
                end_time,
            ))
            .period(300)
            .statistics(aws_sdk_cloudwatch::types::Statistic::Average)
            .statistics(aws_sdk_cloudwatch::types::Statistic::Maximum)
            .send()
            .await
            .context("Failed to fetch memory utilization metrics from CloudWatch")?;

        eprintln!(
            "Received {} memory datapoints",
            memory_response.datapoints().len()
        );

        // Convert datapoints and sort by timestamp
        let mut cpu_datapoints: Vec<MetricDatapoint> = cpu_response
            .datapoints()
            .iter()
            .map(|dp| MetricDatapoint {
                timestamp: dp.timestamp().map(|t| t.secs()).unwrap_or(0),
                average: dp.average(),
                maximum: dp.maximum(),
                minimum: dp.minimum(),
                sum: dp.sum(),
                sample_count: dp.sample_count(),
            })
            .collect();
        cpu_datapoints.sort_by_key(|dp| dp.timestamp);

        let mut memory_datapoints: Vec<MetricDatapoint> = memory_response
            .datapoints()
            .iter()
            .map(|dp| MetricDatapoint {
                timestamp: dp.timestamp().map(|t| t.secs()).unwrap_or(0),
                average: dp.average(),
                maximum: dp.maximum(),
                minimum: dp.minimum(),
                sum: dp.sum(),
                sample_count: dp.sample_count(),
            })
            .collect();
        memory_datapoints.sort_by_key(|dp| dp.timestamp);

        // Fetch alarms for this service
        let alarms = self
            .get_service_alarms(cluster_name, service_name)
            .await
            .unwrap_or_default();

        Ok(Metrics {
            cpu_datapoints,
            memory_datapoints,
            alarms,
            time_range,
            cluster_name: cluster_name.to_string(),
            service_name: service_name.to_string(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test ARN parsing/extraction logic
    #[test]
    fn test_cluster_arn_extraction() {
        let full_arn = "arn:aws:ecs:us-east-1:123456789012:cluster/my-cluster";
        let extracted = full_arn.split('/').next_back().unwrap_or(full_arn);
        assert_eq!(extracted, "my-cluster");
    }

    #[test]
    fn test_cluster_arn_extraction_simple_name() {
        let simple_name = "my-cluster";
        let extracted = simple_name.split('/').next_back().unwrap_or(simple_name);
        assert_eq!(extracted, "my-cluster");
    }

    #[test]
    fn test_task_id_extraction_from_arn() {
        let task_arn = "arn:aws:ecs:us-east-1:123456789012:task/cluster-name/1234567890abcdef";
        let task_id = task_arn.split('/').next_back().unwrap_or("unknown");
        assert_eq!(task_id, "1234567890abcdef");
    }

    #[test]
    fn test_task_id_extraction_with_multiple_slashes() {
        let task_arn = "arn:aws:ecs:region:account:task/cluster/task-id";
        let task_id = task_arn.split('/').next_back().unwrap_or("unknown");
        assert_eq!(task_id, "task-id");
    }

    #[test]
    fn test_container_instance_extraction() {
        let ci_arn = "arn:aws:ecs:us-east-1:123456789012:container-instance/abc123";
        let ci_id = ci_arn.split('/').next_back().unwrap_or("none");
        assert_eq!(ci_id, "abc123");
    }

    #[test]
    fn test_container_instance_extraction_none() {
        let ci_arn_option: Option<&str> = None;
        let ci_id = ci_arn_option
            .and_then(|ci| ci.split('/').next_back())
            .unwrap_or("none");
        assert_eq!(ci_id, "none");
    }

    // Test log stream name construction
    #[test]
    fn test_log_stream_name_format() {
        let stream_prefix = "ecs";
        let container_name = "nginx";
        let task_id = "abc123def456";

        let log_stream = format!("{stream_prefix}/{container_name}/{task_id}");
        assert_eq!(log_stream, "ecs/nginx/abc123def456");
    }

    #[test]
    fn test_log_stream_name_with_custom_prefix() {
        let stream_prefix = "my-app";
        let container_name = "web-server";
        let task_id = "12345";

        let log_stream = format!("{stream_prefix}/{container_name}/{task_id}");
        assert_eq!(log_stream, "my-app/web-server/12345");
    }

    // Test ServiceInfo construction logic
    #[test]
    fn test_service_info_fields() {
        let service = ServiceInfo {
            name: "test-service".to_string(),
            status: "ACTIVE".to_string(),
            desired_count: 3,
            running_count: 2,
            pending_count: 1,
            launch_type: "FARGATE".to_string(),
        };

        assert_eq!(service.name, "test-service");
        assert_eq!(service.status, "ACTIVE");
        assert_eq!(service.desired_count, 3);
        assert_eq!(service.running_count, 2);
        assert_eq!(service.pending_count, 1);
        assert_eq!(service.launch_type, "FARGATE");
    }

    // Test TaskInfo construction logic
    #[test]
    fn test_task_info_fields() {
        let task = TaskInfo {
            task_arn: "arn:aws:ecs:us-east-1:123456789012:task/task-id".to_string(),
            task_id: "task-id".to_string(),
            status: "RUNNING".to_string(),
            desired_status: "RUNNING".to_string(),
            container_instance: "instance-1".to_string(),
            cpu: "256".to_string(),
            memory: "512".to_string(),
        };

        assert_eq!(
            task.task_arn,
            "arn:aws:ecs:us-east-1:123456789012:task/task-id"
        );
        assert_eq!(task.task_id, "task-id");
        assert_eq!(task.status, "RUNNING");
        assert_eq!(task.desired_status, "RUNNING");
        assert_eq!(task.container_instance, "instance-1");
        assert_eq!(task.cpu, "256");
        assert_eq!(task.memory, "512");
    }

    // Test LogEntry construction and ordering
    #[test]
    fn test_log_entry_creation() {
        let log = LogEntry::new(
            1234567890,
            "Test log message".to_string(),
            "web".to_string(),
        );

        assert_eq!(log.timestamp, 1234567890);
        assert_eq!(log.message, "Test log message");
        assert_eq!(log.container_name, "web");
    }

    #[test]
    fn test_log_entries_sorting() {
        let mut logs = [
            LogEntry::new(3000, "third".to_string(), "web".to_string()),
            LogEntry::new(1000, "first".to_string(), "web".to_string()),
            LogEntry::new(2000, "second".to_string(), "web".to_string()),
        ];

        logs.sort_by_key(|log| log.timestamp);

        assert_eq!(logs[0].message, "first");
        assert_eq!(logs[1].message, "second");
        assert_eq!(logs[2].message, "third");
        assert_eq!(logs[0].timestamp, 1000);
        assert_eq!(logs[1].timestamp, 2000);
        assert_eq!(logs[2].timestamp, 3000);
    }

    #[test]
    fn test_log_entries_with_same_timestamp() {
        let mut logs = [
            LogEntry::new(1000, "log A".to_string(), "container1".to_string()),
            LogEntry::new(1000, "log B".to_string(), "container2".to_string()),
        ];

        logs.sort_by_key(|log| log.timestamp);

        // Both should have same timestamp
        assert_eq!(logs[0].timestamp, 1000);
        assert_eq!(logs[1].timestamp, 1000);
    }

    // Test TimeRange functionality
    #[test]
    fn test_time_range_minutes() {
        assert_eq!(TimeRange::OneHour.minutes(), 60);
        assert_eq!(TimeRange::SixHours.minutes(), 360);
        assert_eq!(TimeRange::OneDay.minutes(), 1440);
        assert_eq!(TimeRange::SevenDays.minutes(), 10080);
    }

    #[test]
    fn test_time_range_label() {
        assert_eq!(TimeRange::OneHour.label(), "1h");
        assert_eq!(TimeRange::SixHours.label(), "6h");
        assert_eq!(TimeRange::OneDay.label(), "24h");
        assert_eq!(TimeRange::SevenDays.label(), "7d");
    }

    #[test]
    fn test_time_range_next_cycle() {
        assert_eq!(TimeRange::OneHour.next(), TimeRange::SixHours);
        assert_eq!(TimeRange::SixHours.next(), TimeRange::OneDay);
        assert_eq!(TimeRange::OneDay.next(), TimeRange::SevenDays);
        assert_eq!(TimeRange::SevenDays.next(), TimeRange::OneHour);
    }

    #[test]
    fn test_time_range_from_minutes() {
        assert_eq!(TimeRange::from_minutes(30), TimeRange::OneHour);
        assert_eq!(TimeRange::from_minutes(60), TimeRange::OneHour);
        assert_eq!(TimeRange::from_minutes(120), TimeRange::SixHours);
        assert_eq!(TimeRange::from_minutes(360), TimeRange::SixHours);
        assert_eq!(TimeRange::from_minutes(720), TimeRange::OneDay);
        assert_eq!(TimeRange::from_minutes(1440), TimeRange::OneDay);
        assert_eq!(TimeRange::from_minutes(5000), TimeRange::SevenDays);
        assert_eq!(TimeRange::from_minutes(10080), TimeRange::SevenDays);
    }

    #[test]
    fn test_time_range_equality() {
        assert_eq!(TimeRange::OneHour, TimeRange::OneHour);
        assert_ne!(TimeRange::OneHour, TimeRange::SixHours);
        assert_eq!(TimeRange::SevenDays, TimeRange::SevenDays);
    }

    // Test MetricDatapoint structure
    #[test]
    fn test_metric_datapoint_creation() {
        let dp = MetricDatapoint {
            timestamp: 1234567890,
            average: Some(45.5),
            maximum: Some(75.0),
            minimum: Some(20.0),
            sum: Some(500.0),
            sample_count: Some(10.0),
        };

        assert_eq!(dp.timestamp, 1234567890);
        assert_eq!(dp.average, Some(45.5));
        assert_eq!(dp.maximum, Some(75.0));
        assert_eq!(dp.minimum, Some(20.0));
        assert_eq!(dp.sum, Some(500.0));
        assert_eq!(dp.sample_count, Some(10.0));
    }

    #[test]
    fn test_metric_datapoint_with_none_values() {
        let dp = MetricDatapoint {
            timestamp: 1234567890,
            average: None,
            maximum: None,
            minimum: None,
            sum: None,
            sample_count: None,
        };

        assert_eq!(dp.timestamp, 1234567890);
        assert!(dp.average.is_none());
        assert!(dp.maximum.is_none());
        assert!(dp.minimum.is_none());
        assert!(dp.sum.is_none());
        assert!(dp.sample_count.is_none());
    }

    #[test]
    fn test_metric_datapoint_sorting_by_timestamp() {
        let mut datapoints = vec![
            MetricDatapoint {
                timestamp: 3000,
                average: Some(50.0),
                maximum: Some(60.0),
                minimum: Some(40.0),
                sum: Some(100.0),
                sample_count: Some(2.0),
            },
            MetricDatapoint {
                timestamp: 1000,
                average: Some(30.0),
                maximum: Some(35.0),
                minimum: Some(25.0),
                sum: Some(60.0),
                sample_count: Some(2.0),
            },
            MetricDatapoint {
                timestamp: 2000,
                average: Some(40.0),
                maximum: Some(45.0),
                minimum: Some(35.0),
                sum: Some(80.0),
                sample_count: Some(2.0),
            },
        ];

        datapoints.sort_by_key(|dp| dp.timestamp);

        assert_eq!(datapoints[0].timestamp, 1000);
        assert_eq!(datapoints[1].timestamp, 2000);
        assert_eq!(datapoints[2].timestamp, 3000);
        assert_eq!(datapoints[0].average, Some(30.0));
        assert_eq!(datapoints[1].average, Some(40.0));
        assert_eq!(datapoints[2].average, Some(50.0));
    }

    // Test CloudWatchAlarm structure
    #[test]
    fn test_cloudwatch_alarm_creation() {
        let alarm = CloudWatchAlarm {
            name: "HighCPUAlarm".to_string(),
            description: Some("CPU usage above 80%".to_string()),
            state: "ALARM".to_string(),
            state_reason: Some("Threshold crossed".to_string()),
            metric_name: "CPUUtilization".to_string(),
        };

        assert_eq!(alarm.name, "HighCPUAlarm");
        assert_eq!(alarm.description, Some("CPU usage above 80%".to_string()));
        assert_eq!(alarm.state, "ALARM");
        assert_eq!(alarm.state_reason, Some("Threshold crossed".to_string()));
        assert_eq!(alarm.metric_name, "CPUUtilization");
    }

    #[test]
    fn test_cloudwatch_alarm_ok_state() {
        let alarm = CloudWatchAlarm {
            name: "TestAlarm".to_string(),
            description: None,
            state: "OK".to_string(),
            state_reason: None,
            metric_name: "MemoryUtilization".to_string(),
        };

        assert_eq!(alarm.state, "OK");
        assert!(alarm.description.is_none());
        assert!(alarm.state_reason.is_none());
    }

    #[test]
    fn test_cloudwatch_alarm_insufficient_data_state() {
        let alarm = CloudWatchAlarm {
            name: "NewAlarm".to_string(),
            description: Some("New alarm without data".to_string()),
            state: "INSUFFICIENT_DATA".to_string(),
            state_reason: Some("Not enough data points".to_string()),
            metric_name: "NetworkIn".to_string(),
        };

        assert_eq!(alarm.state, "INSUFFICIENT_DATA");
    }

    // Test Metrics structure
    #[test]
    fn test_metrics_creation() {
        let metrics = Metrics {
            cpu_datapoints: vec![],
            memory_datapoints: vec![],
            alarms: vec![],
            time_range: TimeRange::OneHour,
            cluster_name: "test-cluster".to_string(),
            service_name: "test-service".to_string(),
        };

        assert!(metrics.cpu_datapoints.is_empty());
        assert!(metrics.memory_datapoints.is_empty());
        assert!(metrics.alarms.is_empty());
        assert_eq!(metrics.time_range, TimeRange::OneHour);
        assert_eq!(metrics.cluster_name, "test-cluster");
        assert_eq!(metrics.service_name, "test-service");
    }

    #[test]
    fn test_metrics_with_data() {
        let cpu_dp = MetricDatapoint {
            timestamp: 1000,
            average: Some(50.0),
            maximum: Some(60.0),
            minimum: Some(40.0),
            sum: Some(100.0),
            sample_count: Some(2.0),
        };

        let mem_dp = MetricDatapoint {
            timestamp: 1000,
            average: Some(70.0),
            maximum: Some(80.0),
            minimum: Some(60.0),
            sum: Some(140.0),
            sample_count: Some(2.0),
        };

        let alarm = CloudWatchAlarm {
            name: "TestAlarm".to_string(),
            description: None,
            state: "OK".to_string(),
            state_reason: None,
            metric_name: "CPUUtilization".to_string(),
        };

        let metrics = Metrics {
            cpu_datapoints: vec![cpu_dp],
            memory_datapoints: vec![mem_dp],
            alarms: vec![alarm],
            time_range: TimeRange::SixHours,
            cluster_name: "prod-cluster".to_string(),
            service_name: "web-service".to_string(),
        };

        assert_eq!(metrics.cpu_datapoints.len(), 1);
        assert_eq!(metrics.memory_datapoints.len(), 1);
        assert_eq!(metrics.alarms.len(), 1);
        assert_eq!(metrics.time_range, TimeRange::SixHours);
        assert_eq!(metrics.cpu_datapoints[0].average, Some(50.0));
        assert_eq!(metrics.memory_datapoints[0].average, Some(70.0));
        assert_eq!(metrics.alarms[0].state, "OK");
    }

    #[test]
    fn test_metrics_clone() {
        let metrics = Metrics {
            cpu_datapoints: vec![],
            memory_datapoints: vec![],
            alarms: vec![],
            time_range: TimeRange::OneDay,
            cluster_name: "cluster-1".to_string(),
            service_name: "service-1".to_string(),
        };

        let cloned = metrics.clone();
        assert_eq!(cloned.cluster_name, metrics.cluster_name);
        assert_eq!(cloned.service_name, metrics.service_name);
        assert_eq!(cloned.time_range, metrics.time_range);
    }

    // Test edge cases
    #[test]
    fn test_metrics_with_multiple_datapoints() {
        let datapoints: Vec<MetricDatapoint> = (0..100)
            .map(|i| MetricDatapoint {
                timestamp: i as i64 * 300, // 5-minute intervals
                average: Some(50.0 + (i as f64 % 20.0)),
                maximum: Some(60.0 + (i as f64 % 20.0)),
                minimum: Some(40.0 + (i as f64 % 20.0)),
                sum: Some(100.0),
                sample_count: Some(2.0),
            })
            .collect();

        assert_eq!(datapoints.len(), 100);
        assert_eq!(datapoints[0].timestamp, 0);
        assert_eq!(datapoints[99].timestamp, 29700);
    }

    #[test]
    fn test_metrics_with_multiple_alarms() {
        let alarms = vec![
            CloudWatchAlarm {
                name: "CPUAlarm".to_string(),
                description: Some("High CPU".to_string()),
                state: "ALARM".to_string(),
                state_reason: Some("CPU > 80%".to_string()),
                metric_name: "CPUUtilization".to_string(),
            },
            CloudWatchAlarm {
                name: "MemoryAlarm".to_string(),
                description: Some("High Memory".to_string()),
                state: "OK".to_string(),
                state_reason: None,
                metric_name: "MemoryUtilization".to_string(),
            },
            CloudWatchAlarm {
                name: "NetworkAlarm".to_string(),
                description: None,
                state: "INSUFFICIENT_DATA".to_string(),
                state_reason: Some("New alarm".to_string()),
                metric_name: "NetworkIn".to_string(),
            },
        ];

        assert_eq!(alarms.len(), 3);
        assert_eq!(alarms[0].state, "ALARM");
        assert_eq!(alarms[1].state, "OK");
        assert_eq!(alarms[2].state, "INSUFFICIENT_DATA");
    }

    #[test]
    fn test_time_range_cycle_completeness() {
        // Verify that cycling through all ranges returns to start
        let start = TimeRange::OneHour;
        let cycle1 = start.next();
        let cycle2 = cycle1.next();
        let cycle3 = cycle2.next();
        let cycle4 = cycle3.next();

        assert_eq!(cycle1, TimeRange::SixHours);
        assert_eq!(cycle2, TimeRange::OneDay);
        assert_eq!(cycle3, TimeRange::SevenDays);
        assert_eq!(cycle4, TimeRange::OneHour); // Full cycle complete
    }

    // Test default value handling
    #[test]
    fn test_service_info_with_defaults() {
        let service = ServiceInfo {
            name: "unknown".to_string(),
            status: "unknown".to_string(),
            desired_count: 0,
            running_count: 0,
            pending_count: 0,
            launch_type: "unknown".to_string(),
        };

        assert_eq!(service.name, "unknown");
        assert_eq!(service.status, "unknown");
        assert_eq!(service.launch_type, "unknown");
    }

    #[test]
    fn test_task_info_with_none_container_instance() {
        let task = TaskInfo {
            task_arn: "arn:test".to_string(),
            task_id: "test-id".to_string(),
            status: "RUNNING".to_string(),
            desired_status: "RUNNING".to_string(),
            container_instance: "none".to_string(),
            cpu: "unknown".to_string(),
            memory: "unknown".to_string(),
        };

        assert_eq!(task.container_instance, "none");
        assert_eq!(task.cpu, "unknown");
        assert_eq!(task.memory, "unknown");
    }

    // Test string formatting for describe methods
    #[test]
    fn test_service_description_format() {
        let cluster = "my-cluster";
        let service_name = "my-service";
        let status = "ACTIVE";
        let desired = 5;
        let running = 5;
        let pending = 0;

        let description = format!(
            "Cluster: {cluster}\n\nService Name: {service_name}\nStatus: {status}\nDesired Count: {desired}\nRunning Count: {running}\nPending Count: {pending}\n"
        );

        assert!(description.contains("Cluster: my-cluster"));
        assert!(description.contains("Service Name: my-service"));
        assert!(description.contains("Status: ACTIVE"));
        assert!(description.contains("Desired Count: 5"));
        assert!(description.contains("Running Count: 5"));
    }

    #[test]
    fn test_task_description_format() {
        let cluster = "my-cluster";
        let task_arn = "arn:aws:ecs:region:account:task/task-id";
        let status = "RUNNING";

        let description =
            format!("Cluster: {cluster}\n\nTask ARN: {task_arn}\nLast Status: {status}\n");

        assert!(description.contains("Cluster: my-cluster"));
        assert!(description.contains("Task ARN: arn:aws:ecs"));
        assert!(description.contains("Last Status: RUNNING"));
    }

    // Test edge cases
    #[test]
    fn test_empty_cluster_list() {
        let clusters: Vec<String> = vec![];
        assert_eq!(clusters.len(), 0);
        assert!(clusters.is_empty());
    }

    #[test]
    fn test_empty_services_list() {
        let services: Vec<ServiceInfo> = vec![];
        assert_eq!(services.len(), 0);
        assert!(services.is_empty());
    }

    #[test]
    fn test_empty_tasks_list() {
        let tasks: Vec<TaskInfo> = vec![];
        assert_eq!(tasks.len(), 0);
        assert!(tasks.is_empty());
    }

    #[test]
    fn test_empty_logs_list() {
        let logs: Vec<LogEntry> = vec![];
        assert_eq!(logs.len(), 0);
        assert!(logs.is_empty());
    }

    // Test multi-container scenario
    #[test]
    fn test_multiple_container_logs() {
        let logs = [
            LogEntry::new(1000, "Web server started".to_string(), "web".to_string()),
            LogEntry::new(2000, "Database connected".to_string(), "db".to_string()),
            LogEntry::new(3000, "Cache initialized".to_string(), "redis".to_string()),
        ];

        assert_eq!(logs.len(), 3);
        assert_eq!(logs[0].container_name, "web");
        assert_eq!(logs[1].container_name, "db");
        assert_eq!(logs[2].container_name, "redis");
    }

    // Test string conversions and handling
    #[test]
    fn test_service_name_extraction() {
        let service_arn = "arn:aws:ecs:us-east-1:123456789012:service/cluster-name/service-name";
        let service_name = service_arn.split('/').next_back().unwrap_or("unknown");
        assert_eq!(service_name, "service-name");
    }

    #[test]
    fn test_task_definition_arn_format() {
        let task_def_arn = "arn:aws:ecs:us-east-1:123456789012:task-definition/family:revision";
        assert!(task_def_arn.contains("task-definition"));
        assert!(task_def_arn.contains(":"));
    }

    // Test data integrity
    #[test]
    fn test_service_count_consistency() {
        let service = ServiceInfo {
            name: "test".to_string(),
            status: "ACTIVE".to_string(),
            desired_count: 10,
            running_count: 8,
            pending_count: 2,
            launch_type: "FARGATE".to_string(),
        };

        // Running + Pending should ideally equal Desired (though in practice may vary)
        assert!(service.desired_count >= service.running_count);
        assert!(service.pending_count >= 0);
    }

    #[test]
    fn test_log_timestamp_ordering() {
        let log1 = LogEntry::new(1000, "first".to_string(), "web".to_string());
        let log2 = LogEntry::new(2000, "second".to_string(), "web".to_string());

        assert!(log1.timestamp < log2.timestamp);
    }

    // Test clone trait implementations
    #[test]
    fn test_service_info_debug() {
        let service = ServiceInfo {
            name: "test".to_string(),
            status: "ACTIVE".to_string(),
            desired_count: 1,
            running_count: 1,
            pending_count: 0,
            launch_type: "EC2".to_string(),
        };

        let debug_string = format!("{service:?}");
        assert!(debug_string.contains("test"));
        assert!(debug_string.contains("ACTIVE"));
    }

    #[test]
    fn test_task_info_debug() {
        let task = TaskInfo {
            task_arn: "arn:test".to_string(),
            task_id: "id".to_string(),
            status: "RUNNING".to_string(),
            desired_status: "RUNNING".to_string(),
            container_instance: "instance".to_string(),
            cpu: "256".to_string(),
            memory: "512".to_string(),
        };

        let debug_string = format!("{task:?}");
        assert!(debug_string.contains("arn:test"));
        assert!(debug_string.contains("RUNNING"));
    }

    #[test]
    fn test_log_entry_debug() {
        let log = LogEntry::new(123, "test message".to_string(), "web".to_string());

        let debug_string = format!("{log:?}");
        assert!(debug_string.contains("test message"));
        assert!(debug_string.contains("123"));
    }
}
