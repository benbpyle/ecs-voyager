# ECS Voyager

A terminal user interface (TUI) for exploring and managing AWS ECS resources, inspired by [k9s](https://k9scli.io/) for Kubernetes.

## Features

- 🚀 **Fast Navigation** - Browse ECS clusters, services, and tasks with vim-style keybindings
- 📊 **Real-time Monitoring** - Auto-refresh every 30 seconds to keep data current
- 🔍 **Detailed Views** - Describe services and tasks to view full AWS API responses
- ⚡ **Management Actions** - Restart services and stop tasks directly from the TUI
- 🎨 **Beautiful Interface** - Clean, intuitive UI built with Ratatui

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
- AWS CLI configured with credentials
- Valid AWS credentials with ECS permissions

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
- `r` - Refresh current view
- `d` - Describe selected item (show full details)
- `x` - Execute action:
  - On services: Force new deployment (restart)
  - On tasks: Stop task
- `?` - Toggle help screen
- `q` - Quit application

### Workflow Example

1. Start at **Clusters** view
2. Press `Enter` on a cluster to view its **Services**
3. Press `Enter` on a service to view its **Tasks**
4. Press `d` to see detailed JSON description of a task
5. Press `x` to stop a task
6. Press `Esc` to navigate back up the hierarchy

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
        "ecs:UpdateService",
        "ecs:StopTask"
      ],
      "Resource": "*"
    }
  ]
}
```

## Configuration

### AWS Credentials

ECS Voyager uses the standard AWS SDK credential chain:

1. Environment variables (`AWS_ACCESS_KEY_ID`, `AWS_SECRET_ACCESS_KEY`)
2. AWS credentials file (`~/.aws/credentials`)
3. IAM role (if running on EC2/ECS)

### AWS Region

Set your region using:
```bash
export AWS_REGION=us-east-1
# or
export AWS_DEFAULT_REGION=us-east-1
```

## Development

### Project Structure

```
src/
├── main.rs      # Application entry point and event loop
├── app.rs       # Application state and business logic
├── ui.rs        # UI rendering with Ratatui
└── aws.rs       # AWS ECS client wrapper
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
cargo test
```

## Roadmap

- [ ] Support for ECS Exec (interactive shell into containers)
- [ ] Container logs viewer
- [ ] Service/task filtering and search
- [ ] Multiple cluster selection
- [ ] Task definition viewer and comparison
- [ ] CloudWatch metrics integration
- [ ] Export data to JSON/YAML
- [ ] Custom themes and color schemes
- [ ] Configuration file support

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

MIT License - see LICENSE file for details

## Acknowledgments

- Inspired by [k9s](https://k9scli.io/) - The amazing Kubernetes CLI
- Built with [Ratatui](https://ratatui.rs/) - Terminal UI framework
- Powered by [AWS SDK for Rust](https://aws.amazon.com/sdk-for-rust/)
