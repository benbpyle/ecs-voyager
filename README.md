# ECS Voyager

A terminal user interface (TUI) for exploring and managing AWS ECS resources, inspired by [k9s](https://k9scli.io/) for Kubernetes.

## Features

- 🚀 **Fast Navigation** - Browse ECS clusters, services, and tasks with vim-style keybindings
- 📊 **Real-time Monitoring** - Auto-refresh (configurable interval) to keep data current
- 🔍 **Search & Filter** - Quickly find clusters, services, and tasks with live filtering
- 📝 **CloudWatch Logs** - View container logs with auto-tail support
- ⚡ **Management Actions** - Restart services and stop tasks directly from the TUI
- 🎨 **Beautiful Interface** - Clean, intuitive UI with loading indicators
- ⚙️ **Configuration** - TOML config file support for customization
- 🔐 **AWS SDK Native** - Direct AWS SDK integration (no AWS CLI required)
- 🧪 **Well Tested** - 84 unit tests with >70% code coverage

## Screenshots

```
┌ ECS Voyager - Clusters ─────────────────────────────┐
│ prod-cluster                                         │
│ staging-cluster                                      │
│ dev-cluster                                          │
└──────────────────────────────────────────────────────┘
```

## Installation

### Prerequisites

- Rust 1.70+ ([Install Rust](https://rustup.rs/))
- Valid AWS credentials configured (AWS CLI not required - uses AWS SDK directly)

### Build from Source

```bash
git clone https://github.com/yourusername/ecs-voyager.git
cd ecs-voyager
cargo build --release
```

The binary will be available at `target/release/ecs-voyager`

### Install Locally

```bash
cargo install --path .
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
- `/` - Search/filter current view (case-insensitive)
- `r` - Refresh current view
- `d` - Describe selected item (show full details)
- `l` - View CloudWatch logs (from Tasks view)
- `t` - Toggle auto-tail (in Logs view)
- `x` - Execute action:
  - On services: Force new deployment (restart)
  - On tasks: Stop task
- `?` - Toggle help screen
- `q` - Quit application

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

## AWS Permissions Required

The application requires the following IAM permissions:

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
        "ecs:UpdateService",
        "ecs:StopTask",
        "logs:GetLogEvents"
      ],
      "Resource": "*"
    }
  ]
}
```

**Note**: `logs:GetLogEvents` permission is required for viewing CloudWatch Logs.

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
├── main.rs      # Application entry point and event loop
├── app.rs       # Application state and business logic
├── ui.rs        # UI rendering with Ratatui
├── aws.rs       # AWS ECS and CloudWatch Logs client wrapper
└── config.rs    # Configuration file handling
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
# Run all 84 unit tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run tests for specific module
cargo test app::tests
cargo test aws::tests
cargo test config::tests
```

### Documentation

```bash
# Generate and view documentation
cargo doc --open --no-deps
```

All functions and methods include comprehensive documentation comments.

## Roadmap

### Completed ✅
- [x] Search/filter functionality
- [x] CloudWatch Logs viewer
- [x] Configuration file support
- [x] Loading indicators
- [x] Comprehensive unit tests (84 tests, >70% coverage)
- [x] Full documentation comments

### Planned 📋
- [ ] Better error handling with user-friendly messages
- [ ] Support for ECS Exec (interactive shell into containers)
- [ ] Task definition viewer and comparison
- [ ] CloudWatch metrics integration
- [ ] Export data to JSON/YAML
- [ ] Custom themes and color schemes
- [ ] Multi-region support
- [ ] Service/task filtering by multiple criteria

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Inspired by [k9s](https://k9scli.io/) - The amazing Kubernetes CLI
- Built with [Ratatui](https://ratatui.rs/) - Terminal UI framework
- Powered by [AWS SDK for Rust](https://aws.amazon.com/sdk-for-rust/)
