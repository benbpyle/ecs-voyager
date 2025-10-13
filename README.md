# ECS Voyager

A terminal user interface (TUI) for exploring and managing AWS ECS resources, inspired by [k9s](https://k9scli.io/) for Kubernetes.

## Features

### 🚀 Core Functionality
- **Fast Navigation** - Browse ECS clusters, services, and tasks with vim-style keybindings (↑↓/jk)
- **Real-time Monitoring** - Auto-refresh with configurable intervals to keep data current
- **Resource Details** - View comprehensive details for services and tasks with JSON/formatted toggle
- **AWS SDK Native** - Direct AWS SDK for Rust integration (no AWS CLI required)
- **Multi-Profile/Region** - Switch between AWS profiles and regions on-the-fly

### 🔍 Search & Filtering
- **Live Search** - Quickly find resources with instant case-insensitive filtering
- **Regex Support** - Toggle between literal and regex pattern matching (press `M`)
- **Advanced Filters** - Filter services by status (ACTIVE/DRAINING) and launch type (FARGATE/EC2/EXTERNAL)
- **Task Filters** - Filter tasks by status (RUNNING/PENDING/STOPPED)
- **Multi-Criteria** - Combine search queries with status and type filters

### 📊 Observability
- **CloudWatch Logs** - View container logs with auto-tail, search, and log level filtering
- **CloudWatch Metrics** - Service CPU/Memory metrics with ASCII charts and multiple time ranges
- **CloudWatch Alarms** - View alarm status and state reasons for services
- **Log Export** - Export logs to timestamped files for analysis

### ⚡ Management & Actions
- **ECS Exec** - Interactive shell access to running containers (Fargate & EC2)
- **Service Management** - Restart services with force new deployment
- **Task Management** - Stop tasks with interactive confirmation
- **Profile/Region Switching** - Change AWS context without restarting

### 🎨 User Experience
- **Beautiful Interface** - Clean, intuitive TUI with loading indicators and spinners
- **Context-Aware Help** - Built-in help screen with all keybindings (press `?`)
- **Configuration** - TOML config file support for customization
- **Responsive Layout** - Adapts to different terminal sizes with minimum size validation

### 🧪 Quality & Testing
- **Well Tested** - 224 unit tests with >70% code coverage
- **Comprehensive Docs** - Full rustdoc documentation for all functions and methods
- **Error Handling** - User-friendly error messages with actionable guidance

## Screenshots

```
┌ ECS Voyager - Clusters ─────────────────────────────┐
│ prod-cluster                                         │
│ staging-cluster                                      │
│ dev-cluster                                          │
└──────────────────────────────────────────────────────┘
```

## Installation

### Quick Install (All Platforms)

```bash
curl -sSL https://raw.githubusercontent.com/benbpyle/ecs-voyager/main/install.sh | bash
```

This script automatically detects your platform and installs the appropriate package.

### Platform-Specific Instructions

<details>
<summary><b>macOS (Homebrew)</b></summary>

```bash
# Tap the repository and install
brew tap benbpyle/ecs-voyager
brew install ecs-voyager

# Upgrade to latest version
brew upgrade ecs-voyager
```

Or install directly without tapping:
```bash
brew install benbpyle/ecs-voyager/ecs-voyager
```
</details>

<details>
<summary><b>Windows (Chocolatey)</b></summary>

```powershell
# Install using Chocolatey
choco install ecs-voyager

# Upgrade to latest version
choco upgrade ecs-voyager
```

**Requirements:**
- [Chocolatey](https://chocolatey.org/install) package manager
- Windows 10/11 or Windows Server 2016+

</details>

<details>
<summary><b>Debian/Ubuntu (.deb)</b></summary>

```bash
# Download and install .deb package
curl -sLO https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager_0.2.7_amd64.deb
sudo dpkg -i ecs-voyager_0.2.7_amd64.deb

# Install dependencies if needed
sudo apt-get install -f
```

**Supported:**
- Ubuntu 20.04+
- Debian 11+
- Linux Mint 20+

</details>

<details>
<summary><b>RedHat/Fedora/CentOS (.rpm)</b></summary>

```bash
# Using dnf (Fedora/RHEL 8+)
sudo dnf install https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager-0.2.7-1.x86_64.rpm

# Using yum (RHEL/CentOS 7)
sudo yum install https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager-0.2.7-1.x86_64.rpm
```

**Supported:**
- Fedora 36+
- RHEL/CentOS 7+
- Rocky Linux 8+
- AlmaLinux 8+

</details>

<details>
<summary><b>Arch Linux (AUR)</b></summary>

```bash
# Using yay
yay -S ecs-voyager

# Using paru
paru -S ecs-voyager

# Manual build
git clone https://aur.archlinux.org/ecs-voyager.git
cd ecs-voyager
makepkg -si
```
</details>

<details>
<summary><b>Generic Linux (Binary)</b></summary>

```bash
# Download and extract
curl -sL https://github.com/benbpyle/ecs-voyager/releases/download/v0.2.7/ecs-voyager-v0.2.7-x86_64-unknown-linux-gnu.tar.gz | tar -xz

# Install to /usr/local/bin
sudo install -m 755 ecs-voyager /usr/local/bin/
```
</details>

<details>
<summary><b>Cargo (All Platforms)</b></summary>

If you have Rust installed:

```bash
cargo install --git https://github.com/benbpyle/ecs-voyager.git
```
</details>

<details>
<summary><b>Build from Source</b></summary>

Requirements:
- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Valid AWS credentials configured (AWS CLI not required - uses AWS SDK directly)

```bash
git clone https://github.com/benbpyle/ecs-voyager.git
cd ecs-voyager
cargo build --release
```

The binary will be available at `target/release/ecs-voyager`

Install locally:
```bash
cargo install --path .
```
</details>

### Verify Installation

```bash
ecs-voyager --version
```

## Usage

### Starting the Application

```bash
ecs-voyager
```

The application will automatically:
1. Load AWS credentials from your environment/config
2. Fetch the list of ECS clusters
3. Display the clusters view

### Key Bindings

#### Navigation
- `↑` / `k` - Move up
- `↓` / `j` - Move down
- `Enter` - Select item / drill down
- `Esc` / `h` - Go back to previous view

#### Views
- `1` - Switch to Clusters view
- `2` - Switch to Services view
- `3` - Switch to Tasks view

#### Actions
- `r` - Refresh current view
- `P` - Switch AWS profile
- `R` - Switch AWS region
- `d` - Describe selected item (show full details)
- `J` - Toggle JSON view (in Details view)
- `e` - Context-aware action:
  - On tasks: **ECS Exec** - Interactive shell into container
  - In logs: Export logs to file
- `l` - View CloudWatch logs (from Tasks view)
- `m` - View CloudWatch metrics (from Services view)
- `T` - Cycle time range (in Metrics view: 1h/6h/24h/7d)
- `t` - Toggle auto-tail (in Logs view)
- `x` - Execute action:
  - On services: Force new deployment (restart)
  - On tasks: Stop task
- `?` - Toggle help screen
- `q` - Quit application

#### Search & Filters
- `/` - Enter search mode
- `M` - Toggle regex mode for search
- `F` - Cycle status filter (Services: ACTIVE/DRAINING, Tasks: RUNNING/PENDING/STOPPED)
- `L` - Cycle launch type filter (Services: FARGATE/EC2/EXTERNAL)
- `C` - Clear all active filters
- `f` - Cycle log level filter (in Logs view: DEBUG/INFO/WARN/ERROR)
- `Esc` - Clear search or go back

### Workflow Example

1. Start at **Clusters** view
2. Press `/` to search for a specific cluster, type to filter
3. Press `Enter` on a cluster to view its **Services**
4. Press `/` again to filter services by name, status, or launch type
5. Press `Enter` on a service to view its **Tasks**
6. Press `l` to view **CloudWatch Logs** for a task (auto-tail enabled)
7. Press `t` to toggle auto-tail on/off
8. Press `Esc` to go back to tasks, then `d` to see detailed task description
9. Press `x` to stop a task or restart a service
10. Press `Esc` to navigate back up the hierarchy

## Requirements

### System Requirements
- **Rust**: 1.70+ (for building from source)
- **Terminal**: Minimum 80x24 characters
- **AWS Credentials**: Valid AWS credentials configured

### Feature-Specific Requirements

#### ECS Exec (Interactive Shell)
To use the ECS Exec feature (`e` key in Tasks view):

**Local Requirements:**
- `session-manager-plugin` must be installed:
  ```bash
  # macOS
  brew install --cask session-manager-plugin

  # Linux (Amazon Linux 2/RHEL/CentOS)
  curl "https://s3.amazonaws.com/session-manager-downloads/plugin/latest/linux_64bit/session-manager-plugin.rpm" -o "session-manager-plugin.rpm"
  sudo yum install -y session-manager-plugin.rpm

  # Ubuntu/Debian
  curl "https://s3.amazonaws.com/session-manager-downloads/plugin/latest/ubuntu_64bit/session-manager-plugin.deb" -o "session-manager-plugin.deb"
  sudo dpkg -i session-manager-plugin.deb
  ```

**AWS Task Requirements:**
- Task must be **RUNNING**
- Task definition must have `"enableExecuteCommand": true`
- Service or standalone task must be launched with `--enable-execute-command` flag
- Works with **both Fargate and EC2**:
  - **Fargate**: Platform version 1.4.0 or later
  - **EC2**: ECS container agent version 1.50.2 or later

**Example Task Definition (JSON):**
```json
{
  "family": "my-task",
  "executionRoleArn": "arn:aws:iam::123456789012:role/ecsTaskExecutionRole",
  "taskRoleArn": "arn:aws:iam::123456789012:role/ecsTaskRole",
  "containerDefinitions": [...],
  "requiresCompatibilities": ["FARGATE"],
  "networkMode": "awsvpc",
  "cpu": "256",
  "memory": "512",
  "enableExecuteCommand": true  // <-- Required for ECS Exec
}
```

**Enable on Service:**
```bash
# New service
aws ecs create-service \
  --cluster my-cluster \
  --service-name my-service \
  --task-definition my-task \
  --enable-execute-command  # <-- Required

# Existing service
aws ecs update-service \
  --cluster my-cluster \
  --service my-service \
  --enable-execute-command
```

#### CloudWatch Logs
- Tasks must be configured to send logs to CloudWatch Logs
- Log group and stream must exist
- Supports `awslogs` log driver in task definition

#### CloudWatch Metrics
- Service must be running and generating metrics
- Metrics are available for services (not individual tasks)
- Requires CloudWatch to be enabled in the region

## AWS Permissions Required

### Basic Permissions (Read-Only)
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "ecs:ListClusters",
        "ecs:ListServices",
        "ecs:ListTasks",
        "ecs:DescribeServices",
        "ecs:DescribeTasks",
        "ecs:DescribeTaskDefinition",
        "ecs:DescribeClusters",
        "logs:GetLogEvents",
        "logs:DescribeLogStreams",
        "cloudwatch:GetMetricStatistics",
        "cloudwatch:DescribeAlarms"
      ],
      "Resource": "*"
    }
  ]
}
```

### Management Permissions (Write Operations)
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "ecs:UpdateService",
        "ecs:StopTask"
      ],
      "Resource": "*"
    }
  ]
}
```

### ECS Exec Permissions
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Sid": "AllowECSExec",
      "Effect": "Allow",
      "Action": [
        "ecs:ExecuteCommand",
        "ecs:DescribeTasks"
      ],
      "Resource": "*"
    },
    {
      "Sid": "AllowSSMStartSession",
      "Effect": "Allow",
      "Action": [
        "ssm:StartSession"
      ],
      "Resource": [
        "arn:aws:ecs:*:*:task/*",
        "arn:aws:ssm:*:*:document/AmazonECS-ExecuteInteractiveCommand"
      ]
    }
  ]
}
```

**Task Role Requirements (for ECS Exec):**
The ECS task's task role (not execution role) must have these permissions:
```json
{
  "Version": "2012-10-17",
  "Statement": [
    {
      "Effect": "Allow",
      "Action": [
        "ssmmessages:CreateControlChannel",
        "ssmmessages:CreateDataChannel",
        "ssmmessages:OpenControlChannel",
        "ssmmessages:OpenDataChannel"
      ],
      "Resource": "*"
    }
  ]
}
```

## Configuration

### Configuration File

ECS Voyager supports a TOML configuration file at `~/.ecs-voyager/config.toml`. On first run, a default config file is created automatically.

```toml
[aws]
region = "us-east-1"      # Optional: Default AWS region
profile = "default"        # Optional: AWS profile from ~/.aws/credentials

[behavior]
auto_refresh = true        # Enable/disable automatic refresh
refresh_interval = 30      # Seconds between refreshes
default_view = "clusters"  # Initial view: "clusters", "services", or "tasks"

[ui]
theme = "dark"            # Color theme (for future use)
```

### AWS Credentials

ECS Voyager uses the AWS SDK for Rust (no AWS CLI required) with the standard credential chain:

1. Configuration file (`~/.ecs-voyager/config.toml` - region and profile)
2. Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`, `AWS_REGION`)
3. AWS credentials file (`~/.aws/credentials`)
4. IAM role (if running on EC2/ECS/Lambda)

### Configuration Priority

Settings are resolved in this order (highest to lowest):
1. Environment variables
2. Configuration file (`~/.ecs-voyager/config.toml`)
3. AWS SDK defaults

## Development

### Project Structure

```
src/
├── main.rs         # Application entry point and event loop
├── app.rs          # Application state and business logic
├── aws.rs          # AWS SDK client wrapper (ECS, CloudWatch Logs, CloudWatch Metrics)
├── config.rs       # TOML configuration file handling
├── charts.rs       # ASCII chart rendering for metrics
├── ui/
│   ├── mod.rs      # UI module exports
│   ├── render.rs   # View rendering (clusters, services, tasks, logs, metrics, help)
│   ├── theme.rs    # Color themes and styling
│   ├── utils.rs    # Layout helpers and utilities
│   └── widgets.rs  # Reusable UI components
└── tests/          # Integration tests
```

### Building

```bash
cargo build
```

### Running in Development

```bash
cargo run
```

### Testing

```bash
# Run all 224 unit tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests for specific module
cargo test app::tests
cargo test aws::tests
cargo test config::tests
cargo test ui::tests
cargo test charts::tests
```

### Documentation

```bash
# Generate and view documentation
cargo doc --open --no-deps
```

All functions and methods include comprehensive documentation comments.

## Roadmap

### Completed ✅
- [x] **Core Navigation** - Browse clusters, services, tasks with vim-style keys
- [x] **Search & Filter** - Live search with regex support and multi-criteria filtering
- [x] **CloudWatch Logs** - Viewer with auto-tail, search, log level filtering, and export
- [x] **CloudWatch Metrics** - Service CPU/Memory metrics with ASCII charts and alarms
- [x] **ECS Exec** - Interactive shell access to containers (Fargate & EC2)
- [x] **Port Forwarding** - Forward local ports to container ports using SSM
- [x] **Multi-Profile/Region** - Switch AWS profiles and regions on-the-fly
- [x] **Service Management** - Restart services with force new deployment
- [x] **Task Management** - Stop tasks interactively
- [x] **Service Editor** - Update desired count and task definition from TUI
- [x] **Configuration** - TOML config file support with defaults
- [x] **Resource Details** - Full service/task details with JSON/formatted toggle
- [x] **Task Definition Viewer** - Browse task definition families
- [x] **Testing** - 234 comprehensive unit tests with >70% coverage
- [x] **Documentation** - Complete rustdoc for all functions and methods
- [x] **Multi-Platform Packaging** - Homebrew, Chocolatey, .deb, .rpm packages

### In Progress 🚧
- [ ] **Enhanced Cluster/Service/Task Headers** - Overview information display
- [ ] **Enhanced Error Handling** - More user-friendly error messages with recovery suggestions

### Planned 📋
- [ ] **Task Definition Details** - View full task definition with revision history
- [ ] **Read-Only Mode** - Safety flag to prevent accidental modifications
- [ ] **Export Functionality** - Export current view to JSON/YAML/CSV
- [ ] **Custom Themes** - User-defined color schemes beyond dark/light
- [ ] **Container Instance View** - Browse and manage EC2 container instances
- [ ] **Auto-Scaling Policies** - View and manage service auto-scaling
- [ ] **Session Recording** - Record and replay TUI sessions for debugging
- [ ] **Batch Operations** - Multi-select and bulk actions

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Inspired by [k9s](https://k9scli.io/) - The amazing Kubernetes CLI
- Built with [Ratatui](https://ratatui.rs/) - Terminal UI framework
- Powered by [AWS SDK for Rust](https://aws.amazon.com/sdk-for-rust/)
