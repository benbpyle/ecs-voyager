# ECS Voyager - Feature Requirements

## Project Vision

A terminal user interface (TUI) for exploring and managing AWS ECS resources, inspired by k9s for Kubernetes. The tool should provide an intuitive, keyboard-driven experience for DevOps engineers and developers working with ECS.

## Competitive Analysis (Updated 2025-01-12)

### Main Competitors
1. **e1s** (keidarcy/e1s) - Most feature-complete competitor
2. **ecsplorer** (masaushi/ecsplorer) - Simpler, early-stage tool

### Our Competitive Advantages ✅
- **Native AWS SDK** - Direct SDK integration (no AWS CLI dependency)
- **Advanced Search/Filtering** - Regex support, multi-criteria filtering
- **Well-tested** - 219 unit tests, >70% code coverage
- **Rust-based** - Superior performance and error handling
- **CloudWatch Logs** - Full log viewer with auto-tail
- **Metrics Integration** - CPU/Memory with ASCII charts
- **Comprehensive Documentation** - Full rustdoc comments

### Critical Feature Gaps vs. e1s ❌
1. **ECS Exec** - Interactive shell into containers (PRIORITY 1)
2. **Port Forwarding** - Access private subnet services locally (PRIORITY 5)
3. **Task Definition Management** - View/update/compare (PRIORITY 3)
4. **Service Editing** - Update desired count, task def on-the-fly (PRIORITY 2)
5. **Read-only Mode** - Safety feature to prevent accidents (PRIORITY 6)
6. **Configuration File** - TOML config support (PRIORITY 4)

### Strategic Focus
To compete effectively with e1s, we must prioritize features that developers use daily for debugging and operations, particularly **ECS Exec** and **service management capabilities**.

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
- [x] / - Search/filter
- [x] M - Toggle regex mode for search
- [x] F - Cycle status filters (Services/Tasks)
- [x] L - Cycle launch type filter (Services)
- [x] C - Clear all active filters
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
- [x] Support multiple color themes
- [x] Responsive layout for different terminal sizes
- [x] Minimum terminal size validation

### 5. Search & Filtering

#### 5.1 Search
- [x] Press '/' to enter search mode
- [x] Filter current view by search term
- [ ] Highlight matching items (partial: filters work, text highlighting not yet implemented)
- [x] Clear search with Esc
- [x] Regex pattern support (toggle with 'M' key)

#### 5.2 Filtering
- [x] Filter by service status (ACTIVE, DRAINING) - cycle with 'F' key
- [x] Filter by task status (RUNNING, STOPPED, PENDING) - cycle with 'F' key
- [x] Filter by launch type (EC2, FARGATE) - cycle with 'L' key
- [ ] Save filter presets (deferred for future release)
- [x] Multi-criteria filtering (search + filters work together)

### 6. Logs & Monitoring

#### 6.1 CloudWatch Logs
- [x] View task container logs
- [x] Tail logs in real-time
- [x] Search logs
- [x] Filter by log level
- [x] Export logs to file
- [x] Multiple log stream support

#### 6.2 Metrics
- [x] Display service CPU/Memory metrics
- [ ] Display task CPU/Memory metrics
- [x] Show metrics graphs (ASCII charts)
- [x] Configurable time ranges
- [x] CloudWatch alarms status

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

#### 8.5 Port Forwarding (NEW - PRIORITY 5)
- [ ] List available port forwarding targets from tasks view
- [ ] Initiate port forwarding session to container
- [ ] Support custom local port selection
- [ ] Support custom remote port/target selection
- [ ] Display active port forwarding sessions
- [ ] Terminate port forwarding sessions
- [ ] Show connection status and statistics
- [ ] Support multiple simultaneous forwards
- [ ] Use SSM Session Manager for secure tunneling
- [ ] Handle port conflicts gracefully
- [ ] Persist port forwarding preferences
- [ ] Auto-reconnect on connection drop

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

#### 9.3 Read-only Mode (NEW - PRIORITY 6)
- [ ] Toggle read-only mode on/off (keybinding: 'R' or command-line flag)
- [ ] Display read-only indicator in status bar
- [ ] Disable destructive operations when enabled:
  - Service restarts (force new deployment)
  - Task stops
  - Service updates (desired count, task definition)
  - Task definition registration
  - Any write operations to AWS
- [ ] Show warning when user attempts destructive action
- [ ] Allow read-only mode by default (opt-in for write operations)
- [ ] Configuration file option for default mode
- [ ] Environment variable support (ECS_VOYAGER_READONLY=true)
- [ ] Audit log for attempted operations in read-only mode

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

## Priority Levels (Updated Based on Competitive Analysis)

### P0 - MVP (COMPLETED ✅)
- [x] Basic navigation (clusters → services → tasks)
- [x] View resource details
- [x] Manual refresh
- [x] Service restart
- [x] Task stop
- [x] Help screen

### P1 - Essential (COMPLETED ✅)
- [x] Work off of the AWS SDK not the AWS CLI
- [x] Search/filter functionality (regex + multi-criteria)
- [x] CloudWatch logs viewer
- [x] Loading indicators
- [x] Comprehensive documentation comments
- [x] Unit tests (219 tests, >70% coverage)
- [x] Metrics integration (CPU/Memory with ASCII charts)

### P2 - Competitive Parity (HIGH PRIORITY - Match e1s features)
**These features are critical to compete with e1s. Implement in this order:**

1. **PRIORITY 1: ECS Exec Support** (Section 8.1) ⭐⭐⭐
   - [ ] Check if task supports ECS Exec
   - [ ] Launch interactive shell into containers
   - [ ] Execute one-off commands
   - [ ] Session logging
   - **Impact:** Most requested debugging feature, e1s's killer feature

2. **PRIORITY 2: Service Management Enhancements** (Section 3.1) ⭐⭐⭐
   - [ ] Update service desired count
   - [ ] Update service task definition
   - [ ] Confirmation prompts for destructive actions
   - [ ] View service events/deployments
   - **Impact:** Daily operations, match e1s's service editing

3. **PRIORITY 3: Task Definition Management** (Section 8.2) ⭐⭐
   - [ ] View task definition details
   - [ ] Compare task definition versions
   - [ ] Register new task definition
   - [ ] Export task definition to JSON/YAML
   - **Impact:** Essential for understanding and managing deployments

4. **PRIORITY 4: Configuration File** (Section 7) ⭐⭐
   - [ ] Support ~/.ecs-voyager/config.toml
   - [ ] Configure default AWS region/profile
   - [ ] Configure refresh interval
   - [ ] Configure keybindings
   - [ ] Configure default view
   - **Impact:** Foundation for customization, e1s already has this

5. **PRIORITY 5: Port Forwarding** (Section 8.5) ⭐
   - [ ] Initiate port forwarding to containers
   - [ ] Support custom local/remote ports
   - [ ] Display active sessions
   - [ ] Use SSM Session Manager for tunneling
   - **Impact:** Critical for debugging private subnet services

6. **PRIORITY 6: Read-only Mode** (Section 9.3) ⭐
   - [ ] Toggle read-only mode
   - [ ] Disable destructive operations
   - [ ] Display mode indicator
   - **Impact:** Safety feature to prevent accidents

### P3 - Enhanced
- [ ] Multi-region support (Section 8.3)
- [ ] Export functionality (JSON/CSV/YAML) (Section 8.4)
- [ ] Cluster management (Section 3.3)
- [ ] Performance optimization (caching, pagination) (Section 10)

### P4 - Nice to Have
- [ ] Custom themes
- [ ] Session recording
- [ ] Plugin system
- [ ] API for automation

## UI Requirements

### Layout & Structure
- [x] Three-panel layout (header, content, footer)
- [x] Header shows current view and context
- [x] Content area displays lists/tables/details
- [x] Footer shows keybindings and status
- [x] Responsive layout adapts to terminal size
- [x] Minimum terminal size: 80x24
- [x] Split-pane view for side-by-side comparison (infrastructure ready, not yet integrated into views)

### Color Scheme
- [x] Cyan for selected items and highlights
- [x] Yellow for keybinding hints
- [x] Green for success messages
- [x] White/Gray for normal text
- [x] Red for errors and warnings
- [x] Custom theme support (dark/light/custom)
- [x] Color configuration in config file

### Visual Feedback
- [x] Bold text for selected items
- [x] Background highlight for active row
- [x] Loading spinner during API calls
- [x] Progress bar for long operations
- [x] Success/error toast notifications
- [x] Blinking cursor in input mode
- [x] Dimmed/disabled items for unavailable actions

### Typography & Formatting
- [x] Monospace font (terminal default)
- [x] Table alignment (left/right)
- [x] Column headers with separators
- [x] Unicode box-drawing characters
- [x] Truncate long text with ellipsis
- [x] Word wrap in details view
- [x] Line numbers in log viewer

### Interactive Elements
- [x] Selectable list items
- [x] Scrollable content areas
- [x] Text input fields
- [x] Dropdown menus
- [x] Confirmation dialogs
- [x] Multi-select checkboxes
- [ ] Context menus (right-click/menu key) - Optional feature, not implemented

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

## Implementation Roadmap (Competitive-Driven)

This roadmap prioritizes features that close the gap with e1s while leveraging our unique advantages (native AWS SDK, advanced search, strong testing).

### Phase 1: ECS Exec & Core Operations (Q1 2025)
**Goal:** Match e1s's most critical debugging feature

**Features:**
- ✅ Section 8.1: ECS Exec Support (PRIORITY 1)
  - Interactive shell into containers
  - One-off command execution
  - ECS Exec capability checking
  - Session logging and history

**Dependencies:**
- AWS SSM Session Manager integration
- Terminal PTY handling for interactive sessions
- Proper IAM permission checking

**Estimated Effort:** 3-4 weeks
**Key Deliverable:** `ecs-voyager exec` command for container access

---

### Phase 2: Service Management & Safety (Q1 2025)
**Goal:** Enable daily operations and prevent accidents

**Features:**
- ✅ Section 3.1: Service Management Enhancements (PRIORITY 2)
  - Update service desired count with validation
  - Update service task definition
  - View service events and deployment history
  - Rich deployment status display

- ✅ Section 9.3: Read-only Mode (PRIORITY 6)
  - Toggle read-only mode (default: on)
  - Confirmation prompts for all destructive actions
  - Clear visual indicators

**Dependencies:**
- ECS UpdateService API integration
- ECS DescribeServices events parsing
- Input validation and error handling

**Estimated Effort:** 2-3 weeks
**Key Deliverable:** Safe, auditable service modifications

---

### Phase 3: Task Definition Management (Q2 2025)
**Goal:** Complete view/edit/compare workflow for task definitions

**Features:**
- ✅ Section 8.2: Task Definition Management (PRIORITY 3)
  - View task definition JSON (syntax highlighted)
  - Compare two task definition revisions (diff view)
  - Register new task definition from file/editor
  - Export task definition to JSON/YAML
  - Task definition revision history browser

**Dependencies:**
- JSON/YAML parsing and formatting
- Diff algorithm for revision comparison
- File I/O for export/import

**Estimated Effort:** 2-3 weeks
**Key Deliverable:** Complete task definition workflow in TUI

---

### Phase 4: Configuration System (Q2 2025)
**Goal:** Enable user customization and persistence

**Features:**
- ✅ Section 7: Configuration File (PRIORITY 4)
  - TOML config file at `~/.ecs-voyager/config.toml`
  - AWS region/profile configuration
  - Auto-refresh interval settings
  - Keybinding customization
  - Default view selection
  - Color theme preferences
  - Config validation and migration

**Dependencies:**
- TOML parsing library (serde integration)
- Config file watching for hot-reload
- Schema validation

**Estimated Effort:** 2 weeks
**Key Deliverable:** Fully customizable user experience

---

### Phase 5: Port Forwarding (Q2 2025)
**Goal:** Enable access to private subnet services

**Features:**
- ✅ Section 8.5: Port Forwarding (PRIORITY 5)
  - SSM Session Manager port forwarding
  - Local/remote port configuration
  - Active session management
  - Connection status monitoring
  - Multiple simultaneous forwards

**Dependencies:**
- AWS SSM StartSession API
- Local port binding and forwarding
- Session lifecycle management

**Estimated Effort:** 2-3 weeks
**Key Deliverable:** Secure tunneling to ECS tasks

---

### Phase 6: Polish & Performance (Q3 2025)
**Goal:** Production-ready quality

**Features:**
- ✅ Section 10: Performance Optimization
  - Caching for cluster/service/task lists
  - Pagination for large result sets (100+ items)
  - Parallel data fetching
  - Request cancellation

- ✅ Section 8.3: Multi-region Support
  - Region selector UI
  - Switch region at runtime
  - Cross-region cluster listing

- ✅ Section 8.4: Export & Sharing
  - Export to JSON/CSV/YAML
  - Copy ARNs to clipboard
  - Generate AWS CLI commands

**Dependencies:**
- Async runtime optimization
- Memory-efficient data structures
- Cross-region AWS client handling

**Estimated Effort:** 3-4 weeks
**Key Deliverable:** Production-grade performance and UX

---

### Testing Strategy (Continuous)
- **Unit tests:** >80% coverage target for all new features
- **Integration tests:** Mock AWS SDK calls for all operations
- **Manual testing:** Test with real ECS clusters (dev/staging)
- **Performance testing:** Benchmark with 100+ services/tasks
- **Security testing:** IAM permission validation, audit logging

### Success Metrics
- **Feature Parity:** Match e1s on top 6 priority features by Q2 2025
- **Performance:** Sub-second response for all operations
- **Reliability:** Zero crashes, graceful error handling
- **Testing:** >80% code coverage maintained
- **Adoption:** 50+ GitHub stars, 10+ active contributors by Q3 2025

## Success Criteria

1. **Usability**: Users can navigate ECS resources faster than AWS Console
2. **Reliability**: No crashes, graceful error handling
3. **Performance**: Sub-second response for most operations
4. **Completeness**: Cover 80% of common ECS management tasks
5. **Adoption**: Positive feedback from at least 10 active users
