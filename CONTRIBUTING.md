# ECS Voyager Development Guidelines

## Pre-Push Checklist ✅

**ALWAYS run these commands before committing and pushing code:**

```bash
# 1. Format code with rustfmt
cargo fmt

# 2. Run clippy with warnings as errors
cargo clippy -- -D warnings

# 3. Run all tests
cargo test

# 4. Build in release mode (catches optimization issues)
cargo build --release
```

### Why These Checks Matter

- **`cargo fmt`** - Ensures consistent code formatting across the project
- **`cargo clippy`** - Catches common mistakes and enforces Rust best practices
- **`cargo test`** - Verifies all 224+ unit tests pass
- **`cargo build --release`** - Catches issues that only appear with optimizations

### Quick Pre-Push Script

Create this as `scripts/pre-push.sh`:

```bash
#!/bin/bash
set -e

echo "🔍 Running pre-push checks..."

echo "1️⃣  Formatting code..."
cargo fmt

echo "2️⃣  Running clippy..."
cargo clippy -- -D warnings

echo "3️⃣  Running tests..."
cargo test

echo "4️⃣  Building release..."
cargo build --release

echo "✅ All checks passed! Safe to push."
```

Make it executable: `chmod +x scripts/pre-push.sh`

## Common Clippy Fixes

### Uninlined Format Args
❌ **Bad:**
```rust
format!("Error: {}", error_msg)
```

✅ **Good:**
```rust
format!("Error: {error_msg}")
```

### Unused Variables
❌ **Bad:**
```rust
let result = some_function();
```

✅ **Good:**
```rust
let _result = some_function();  // Prefix with _ if intentionally unused
// OR
let _ = some_function();         // If you don't need the value at all
```

## Test Coverage Standards

- All new features must include unit tests
- Target: >70% code coverage
- Test edge cases, error conditions, and happy paths
- Use descriptive test names: `test_feature_scenario_expected_behavior`

## Documentation Standards

- All public functions must have rustdoc comments
- Include:
  - Brief description of what the function does
  - `# Arguments` section for parameters
  - `# Returns` section for return values
  - `# Errors` section for potential errors
  - `# Examples` if helpful

Example:
```rust
/// Checks if a task has ECS Exec enabled.
///
/// # Arguments
/// * `cluster` - The ECS cluster name
/// * `task_arn` - The full ARN of the task
///
/// # Returns
/// Returns `Ok(true)` if exec is enabled, `Ok(false)` otherwise
///
/// # Errors
/// Returns an error if the task doesn't exist or AWS API call fails
pub async fn check_task_exec_enabled(&self, cluster: &str, task_arn: &str) -> Result<bool> {
    // ...
}
```

## Git Workflow

1. Create feature branch: `git checkout -b feature/feature-name`
2. Make changes and test locally
3. Run pre-push checks (see above)
4. Commit with descriptive messages
5. Push and create PR
6. Ensure CI passes before merging

## Commit Message Format

Use conventional commits:

```
feat: add ECS Exec support for interactive shell access
fix: improve error visibility when exec fails
docs: update README with ECS Exec requirements
test: add unit tests for Session struct
refactor: extract session handling into separate module
```

Types: `feat`, `fix`, `docs`, `test`, `refactor`, `perf`, `chore`

## Performance Considerations

- Use `async/await` for all AWS API calls
- Avoid blocking the event loop
- Cache results when appropriate (with TTL)
- Profile with `cargo flamegraph` if performance issues arise

## Security Guidelines

- Never commit AWS credentials
- Use environment variables or AWS config for credentials
- Validate all user input before using in AWS API calls
- Use `anyhow::Context` to add context to errors without exposing sensitive data
