# ECS Voyager - Feature Requirements

## Project Vision

A terminal user interface (TUI) for exploring and managing AWS ECS resources, inspired by k9s for Kubernetes. The tool should provide an intuitive, keyboard-driven experience for DevOps engineers and developers working with ECS.

## Core Requirements

Context:  

* Always check off tasks as you finish.
* Always build unit tests
* Use subagents and MCP servers will at all possible
* Add comments to functions and methods so that help context shows up


### 1. Navigation & View Management

#### 1.1 Cluster View
- [x] List all ECS clusters in the current AWS region
- [x] Display cluster names (extracted from ARNs)
- [x] Support keyboard navigation (↑/↓, j/k)
- [x] Highlight selected cluster
- [x] Press Enter to drill down into services

#### 1.2 Services View
- [x] List all services in the selected cluster
- [x] Display service information in table format:
  - Service name
  - Status (ACTIVE, DRAINING, etc.)
  - Desired count
  - Running count
  - Pending count
  - Launch type (EC2, FARGATE, EXTERNAL)
- [x] Navigate back to clusters with Esc/h
- [x] Press Enter to view tasks for selected service

#### 1.3 Tasks View
- [x] List all tasks for the selected service
- [x] Display task information in table format:
  - Task ID (short form)
  - Current status
  - Desired status
  - Container instance
  - CPU allocation
  - Memory allocation
- [x] Navigate back to services with Esc/h
- [x] Press Enter to view task details

#### 1.4 Details View
- [x] Display formatted details for selected resource
- [x] Show service details (ARN, task definition, load balancers, etc.)
- [x] Show task details (containers, network info, etc.)
- [x] Navigate back with Esc/h
- [x] Support scrolling for long content

### 2. Data Management

#### 2.1 Data Fetching
- [x] Initialize AWS SDK with credentials from environment
- [x] List clusters using ECS API
- [x] List services for a cluster
- [x] List tasks for a service
- [x] Describe services for detailed information
- [x] Describe tasks for detailed information

#### 2.2 Auto-refresh
- [x] Implement 30-second auto-refresh for current view
- [x] Add configurable refresh interval
- [x] Show refresh indicator/timestamp
- [x] Pause auto-refresh on user interaction

#### 2.3 Manual Refresh
- [x] Press 'r' to manually refresh current view
- [x] Update status message on refresh
- [x] Show loading indicator during refresh
- [x] Handle refresh errors gracefully

### 3. Management Operations

#### 3.1 Service Management
- [x] Restart service (force new deployment)
- [ ] Update service desired count
- [ ] Update service task definition
- [ ] Enable/disable service auto-scaling
- [ ] View service events/deployments
- [ ] Confirmation prompts for destructive actions

#### 3.2 Task Management
- [x] Stop task with reason
- [ ] Execute command in running container (ECS Exec)
- [ ] View task stopped reason
- [ ] View task CloudWatch logs
- [ ] Copy task ARN to clipboard

#### 3.3 Cluster Management
- [ ] View cluster capacity providers
- [ ] View cluster statistics
- [ ] View container instances
- [ ] Drain container instances

### 4. User Interface

#### 4.1 Keybindings
- [x] q - Quit application
- [x] ? - Toggle help screen
- [x] 1/2/3 - Switch to Clusters/Services/Tasks view
- [x] ↑/↓ or j/k - Navigate up/down
- [x] Enter - Select/drill down
- [x] Esc/h - Go back
- [x] r - Refresh
- [x] d - Describe (show details)
- [x] x - Execute action (context-dependent)
- [ ] / - Search/filter
- [ ] : - Command mode
- [ ] y - Yank/copy to clipboard

#### 4.2 Help Screen
- [x] Toggle with '?'
- [x] Show all available keybindings
- [x] Categorize by function (Navigation, Views, Actions, General)
- [ ] Show context-sensitive help

#### 4.3 Status Bar
- [x] Display current view/context
- [x] Show status messages
- [x] Show selected cluster/service breadcrumb
- [ ] Show AWS region
- [ ] Show last refresh time
- [ ] Show error messages with severity

#### 4.4 Visual Design
- [x] Use consistent color scheme (Cyan for highlights)
- [x] Bold selected items
- [x] Clear section borders
- [ ] Support multiple color themes
- [ ] Responsive layout for different terminal sizes
- [ ] Minimum terminal size validation

### 5. Search & Filtering

#### 5.1 Search
- [ ] Press '/' to enter search mode
- [ ] Filter current view by search term
- [ ] Highlight matching items
- [ ] Clear search with Esc
- [ ] Regex pattern support

#### 5.2 Filtering
- [ ] Filter by service status (ACTIVE, DRAINING)
- [ ] Filter by task status (RUNNING, STOPPED, PENDING)
- [ ] Filter by launch type (EC2, FARGATE)
- [ ] Save filter presets
- [ ] Multi-criteria filtering

### 6. Logs & Monitoring

#### 6.1 CloudWatch Logs
- [ ] View task container logs
- [ ] Tail logs in real-time
- [ ] Search logs
- [ ] Filter by log level
- [ ] Export logs to file
- [ ] Multiple log stream support

#### 6.2 Metrics
- [ ] Display service CPU/Memory metrics
- [ ] Display task CPU/Memory metrics
- [ ] Show metrics graphs (ASCII charts)
- [ ] Configurable time ranges
- [ ] CloudWatch alarms status

### 7. Configuration

#### 7.1 Configuration File
- [ ] Support ~/.ecs-voyager/config.toml
- [ ] Configure default AWS region
- [ ] Configure refresh interval
- [ ] Configure color theme
- [ ] Configure keybindings
- [ ] Configure default view

#### 7.2 Profiles
- [ ] Support multiple AWS profiles
- [ ] Switch profiles at runtime
- [ ] Profile-specific settings
- [ ] Default profile selection

#### 7.3 Preferences
- [ ] Remember last viewed cluster/service
- [ ] Save window/pane layout
- [ ] Persist search/filter settings
- [ ] Auto-refresh on/off toggle

### 8. Advanced Features

#### 8.1 ECS Exec
- [ ] Check if task supports ECS Exec
- [ ] Launch interactive shell
- [ ] Execute one-off commands
- [ ] Session logging

#### 8.2 Task Definitions
- [ ] View task definition details
- [ ] Compare task definition versions
- [ ] Register new task definition
- [ ] Export task definition to JSON/YAML

#### 8.3 Multi-region Support
- [ ] List clusters across all regions
- [ ] Switch region at runtime
- [ ] Remember region per cluster
- [ ] Region selector UI

#### 8.4 Export & Sharing
- [ ] Export current view to JSON
- [ ] Export current view to CSV
- [ ] Export current view to YAML
- [ ] Copy resource ARNs
- [ ] Generate AWS CLI commands

### 9. Error Handling & Resilience

#### 9.1 Error Display
- [x] Show user-friendly error messages
- [ ] Display AWS API error details
- [ ] Retry failed API calls
- [ ] Offline mode with cached data
- [ ] Error log file

#### 9.2 Validation
- [ ] Validate AWS credentials on startup
- [ ] Check IAM permissions
- [ ] Validate terminal size
- [ ] Warn on missing optional features

### 10. Performance & Optimization

#### 10.1 Caching
- [ ] Cache cluster/service/task lists
- [ ] Invalidate cache on manual refresh
- [ ] Configurable cache TTL
- [ ] Persist cache to disk

#### 10.2 Pagination
- [ ] Support large result sets (100+ items)
- [ ] Lazy loading for services/tasks
- [ ] Virtual scrolling for long lists
- [ ] Batch API requests

#### 10.3 Async Operations
- [x] Non-blocking UI during API calls
- [x] Background refresh
- [ ] Parallel data fetching
- [ ] Request cancellation

### 11. Testing & Quality

#### 11.1 Unit Tests
- [ ] Test AWS client wrapper
- [ ] Test UI components
- [ ] Test application state management
- [ ] Mock AWS SDK calls

#### 11.2 Integration Tests
- [ ] Test with localstack
- [ ] Test key event handling
- [ ] Test navigation flows
- [ ] Test error scenarios

#### 11.3 Documentation
- [x] README with usage instructions
- [x] IAM permissions documentation
- [ ] Architecture documentation
- [ ] Contributing guide
- [ ] Changelog

### 12. Distribution & Deployment

#### 12.1 Packaging
- [ ] Publish to crates.io
- [x] Create GitHub releases (workflow ready, disabled)
- [x] Build multi-platform binaries (macOS x86_64, macOS ARM64, Linux x86_64, Linux ARM64, Windows)
- [ ] Docker container
- [x] Homebrew formula (created, ready for deployment)
- [ ] APT/YUM packages

#### 12.2 CI/CD
- [x] Automated testing in CI (part of release workflow)
- [x] Automated builds (multi-platform)
- [x] Automated releases (GitFlow workflow ready, disabled)
- [x] Security scanning (cargo-audit in workflow)
- [ ] Dependency updates (Dependabot)

## Priority Levels

### P0 - MVP (Complete)
- [x] Basic navigation (clusters → services → tasks)
- [x] View resource details
- [x] Manual refresh
- [x] Service restart
- [x] Task stop
- [x] Help screen

### P1 - Essential (COMPLETED ✅)
- [x] Work off of the AWS SDK not the AWS CLI
- [x] Search/filter functionality
- [x] CloudWatch logs viewer
- [x] Configuration file support
- [x] Loading indicators
- [x] Comprehensive documentation comments
- [x] Unit tests (84 tests, >70% coverage)

### P2 - Enhanced
- [ ] ECS Exec support
- [ ] Multi-region support
- [ ] Metrics/monitoring
- [ ] Task definition management
- [ ] Export functionality

### P3 - Nice to Have
- [ ] Custom themes
- [ ] Advanced filtering
- [ ] Session recording
- [ ] Plugin system
- [ ] API for automation

## UI Requirements

### Layout & Structure
- [x] Three-panel layout (header, content, footer)
- [x] Header shows current view and context
- [x] Content area displays lists/tables/details
- [x] Footer shows keybindings and status
- [ ] Responsive layout adapts to terminal size
- [ ] Minimum terminal size: 80x24
- [ ] Split-pane view for side-by-side comparison

### Color Scheme
- [x] Cyan for selected items and highlights
- [x] Yellow for keybinding hints
- [x] Green for success messages
- [x] White/Gray for normal text
- [ ] Red for errors and warnings
- [ ] Custom theme support (dark/light/custom)
- [ ] Color configuration in config file

### Visual Feedback
- [x] Bold text for selected items
- [x] Background highlight for active row
- [ ] Loading spinner during API calls
- [ ] Progress bar for long operations
- [ ] Success/error toast notifications
- [ ] Blinking cursor in input mode
- [ ] Dimmed/disabled items for unavailable actions

### Typography & Formatting
- [x] Monospace font (terminal default)
- [x] Table alignment (left/right)
- [x] Column headers with separators
- [ ] Unicode box-drawing characters
- [ ] Truncate long text with ellipsis
- [ ] Word wrap in details view
- [ ] Line numbers in log viewer

### Interactive Elements
- [x] Selectable list items
- [x] Scrollable content areas
- [ ] Text input fields
- [ ] Dropdown menus
- [ ] Confirmation dialogs
- [ ] Multi-select checkboxes
- [ ] Context menus (right-click/menu key)

### Accessibility
- [ ] High contrast mode
- [ ] Screen reader friendly output
- [ ] Configurable keybindings for different layouts
- [ ] Mouse support (optional)
- [ ] Audio feedback for actions

## Configuration Requirements

### Configuration File Format
- [ ] TOML format: `~/.ecs-voyager/config.toml`
- [ ] YAML alternative: `~/.ecs-voyager/config.yaml`
- [ ] JSON alternative: `~/.ecs-voyager/config.json`
- [ ] Auto-create default config on first run
- [ ] Schema validation for config files
- [ ] Config file hot-reload (watch for changes)

### AWS Configuration
- [ ] Default AWS region
- [ ] Default AWS profile
- [ ] Custom endpoint URL (for testing)
- [ ] AWS SDK retry configuration
- [ ] Request timeout settings
- [ ] Assume role configuration
- [ ] MFA token cache

### Application Settings
- [ ] Default view on startup (clusters/services/tasks)
- [ ] Auto-refresh interval (seconds)
- [ ] Auto-refresh enabled/disabled
- [ ] Max items per page
- [ ] Date/time format
- [ ] Log level (debug/info/warn/error)
- [ ] Log file location

### UI Preferences
- [ ] Color theme (dark/light/custom)
- [ ] Border style (single/double/rounded)
- [ ] Show/hide status bar
- [ ] Show/hide breadcrumbs
- [ ] Compact vs. detailed view mode
- [ ] Column visibility toggles
- [ ] Default sort order

### Keybinding Customization
- [ ] Custom key mappings
- [ ] Vim/Emacs mode presets
- [ ] Disable specific keybindings
- [ ] Multi-key sequences
- [ ] Leader key support
- [ ] Export/import keybinding profiles

### Filter & Search Presets
- [ ] Save named filters
- [ ] Default filter on startup
- [ ] Recent searches history
- [ ] Search case sensitivity
- [ ] Regex vs. literal search mode

### Cache Settings
- [ ] Cache directory location
- [ ] Cache TTL (time-to-live)
- [ ] Max cache size
- [ ] Cache cleanup policy
- [ ] Disable caching option

### Example Configuration File
```toml
[aws]
region = "us-east-1"
profile = "default"
timeout = 30

[ui]
theme = "dark"
border_style = "rounded"
show_breadcrumbs = true

[behavior]
auto_refresh = true
refresh_interval = 30
default_view = "clusters"

[keybindings]
quit = "q"
help = "?"
refresh = "r"
search = "/"
```

## Build and Deployment Requirements

### Build System
- [x] Cargo-based build system
- [x] Debug build: `cargo build`
- [x] Release build: `cargo build --release`
- [ ] Profile-guided optimization (PGO)
- [ ] Link-time optimization (LTO)
- [ ] Strip debug symbols in release
- [ ] Cross-compilation support
- [ ] Build reproducibility

### Dependency Management
- [x] Cargo.toml with pinned versions
- [ ] Cargo.lock committed to repository
- [ ] Regular dependency updates
- [ ] Security vulnerability scanning
- [ ] License compliance checking
- [ ] Minimal dependency tree
- [ ] Optional feature flags

### Platform Support
- [x] macOS (x86_64, ARM64)
- [ ] Linux (x86_64, ARM64, ARMv7)
- [ ] Windows (x86_64, WSL2)
- [ ] FreeBSD
- [ ] Platform-specific features/workarounds

### Binary Distribution
- [ ] GitHub Releases with binaries
- [ ] Standalone binary (static linking)
- [ ] Compressed archives (.tar.gz, .zip)
- [ ] Checksums (SHA256) for verification
- [ ] GPG signatures for security
- [ ] Version numbering (SemVer)
- [ ] Release notes/changelog

### Package Managers
- [ ] crates.io publication
- [ ] Homebrew formula (macOS/Linux)
- [ ] APT repository (Debian/Ubuntu)
- [ ] YUM/DNF repository (RHEL/Fedora)
- [ ] Chocolatey (Windows)
- [ ] Snap package (Linux)
- [ ] AUR package (Arch Linux)
- [ ] Docker Hub image

### Container Distribution
- [ ] Dockerfile for containerized usage
- [ ] Multi-stage build for small images
- [ ] Alpine-based image
- [ ] Support for AWS credentials pass-through
- [ ] Docker Compose example
- [ ] Kubernetes deployment example

### Installation Methods
- [ ] `cargo install ecs-voyager`
- [ ] `brew install ecs-voyager`
- [ ] `apt-get install ecs-voyager`
- [ ] `yum install ecs-voyager`
- [ ] `curl | sh` installer script
- [ ] Download binary from GitHub releases
- [ ] Build from source instructions

### CI/CD Pipeline
- [ ] GitHub Actions workflow
- [ ] Automated testing on push/PR
- [ ] Multi-platform build matrix
- [ ] Code coverage reporting
- [ ] Linting (clippy) in CI
- [ ] Formatting check (rustfmt) in CI
- [ ] Security audit (cargo-audit) in CI
- [ ] Automated dependency updates (Dependabot)

### Release Process
- [ ] Automated version bumping
- [ ] Automated changelog generation
- [ ] Tag-based release triggering
- [ ] Build all platform binaries
- [ ] Upload artifacts to GitHub Releases
- [ ] Publish to crates.io
- [ ] Update package manager formulas
- [ ] Announce on social media/forums

### Quality Gates
- [ ] All tests must pass
- [ ] Code coverage > 70%
- [ ] No clippy warnings
- [ ] No security vulnerabilities
- [ ] Documentation up-to-date
- [ ] Changelog entry required
- [ ] Semantic version validation

### Versioning Strategy
- [ ] SemVer (MAJOR.MINOR.PATCH)
- [ ] Pre-release versions (alpha, beta, rc)
- [ ] Development builds from main branch
- [ ] Git tag = cargo version
- [ ] Version displayed with --version flag
- [ ] Build metadata in binary

### Update Mechanism
- [ ] Check for updates on startup (optional)
- [ ] Notify user of new versions
- [ ] In-app update command
- [ ] Auto-update option (opt-in)
- [ ] Release channel selection (stable/beta)

## Success Criteria

1. **Usability**: Users can navigate ECS resources faster than AWS Console
2. **Reliability**: No crashes, graceful error handling
3. **Performance**: Sub-second response for most operations
4. **Completeness**: Cover 80% of common ECS management tasks
5. **Adoption**: Positive feedback from at least 10 active users
