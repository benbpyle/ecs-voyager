//! Configuration management for ECS Voyager.
//!
//! This module handles loading and managing application configuration from a TOML file
//! located at `~/.ecs-voyager/config.toml`. Configuration includes AWS settings,
//! application behavior, and UI preferences.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

/// Main configuration structure for ECS Voyager.
///
/// All configuration options are optional and will fall back to sensible defaults
/// if not specified in the configuration file.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// AWS-specific configuration options
    #[serde(default)]
    pub aws: AwsConfig,

    /// Application behavior configuration
    #[serde(default)]
    pub behavior: BehaviorConfig,

    /// UI and display configuration
    #[serde(default)]
    pub ui: UiConfig,
}

/// AWS SDK configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsConfig {
    /// Default AWS region (e.g., "us-east-1")
    /// If not specified, will use AWS SDK's default resolution (env vars, profile, etc.)
    pub region: Option<String>,

    /// AWS profile name to use from ~/.aws/credentials
    /// If not specified, will use the default profile
    pub profile: Option<String>,
}

/// Application behavior configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BehaviorConfig {
    /// Whether to automatically refresh data periodically
    #[serde(default = "default_auto_refresh")]
    pub auto_refresh: bool,

    /// Interval in seconds between automatic refreshes
    #[serde(default = "default_refresh_interval")]
    pub refresh_interval: u64,

    /// Default view to show on startup (e.g., "clusters", "services", "tasks")
    #[serde(default = "default_view")]
    pub default_view: String,
}

/// UI configuration options.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UiConfig {
    /// Color theme for the UI (for future use)
    /// Options: "dark", "light", "custom"
    #[serde(default = "default_theme")]
    pub theme: String,
}

// Default value functions for serde
fn default_auto_refresh() -> bool {
    true
}

fn default_refresh_interval() -> u64 {
    30
}

fn default_view() -> String {
    "clusters".to_string()
}

fn default_theme() -> String {
    "dark".to_string()
}

// Implement Default trait for all config structs
impl Default for Config {
    fn default() -> Self {
        Self {
            aws: AwsConfig::default(),
            behavior: BehaviorConfig::default(),
            ui: UiConfig::default(),
        }
    }
}

impl Default for AwsConfig {
    fn default() -> Self {
        Self {
            region: None,
            profile: None,
        }
    }
}

impl Default for BehaviorConfig {
    fn default() -> Self {
        Self {
            auto_refresh: default_auto_refresh(),
            refresh_interval: default_refresh_interval(),
            default_view: default_view(),
        }
    }
}

impl Default for UiConfig {
    fn default() -> Self {
        Self {
            theme: default_theme(),
        }
    }
}

impl Config {
    /// Returns the path to the configuration directory (~/.ecs-voyager/)
    pub fn config_dir() -> Result<PathBuf> {
        let home_dir = dirs::home_dir()
            .context("Failed to determine home directory")?;
        Ok(home_dir.join(".ecs-voyager"))
    }

    /// Returns the path to the configuration file (~/.ecs-voyager/config.toml)
    pub fn config_file_path() -> Result<PathBuf> {
        Ok(Self::config_dir()?.join("config.toml"))
    }

    /// Loads configuration from the config file, creating a default if it doesn't exist.
    ///
    /// # Behavior
    /// 1. If the config file exists, parse and return it
    /// 2. If the config file doesn't exist, create default config file and return defaults
    /// 3. If parsing fails, return error with context
    ///
    /// # Returns
    /// Returns the loaded configuration or an error if file operations fail
    ///
    /// # Errors
    /// This function will return an error if:
    /// - Home directory cannot be determined
    /// - File I/O operations fail
    /// - TOML parsing fails
    pub fn load() -> Result<Self> {
        let config_path = Self::config_file_path()?;

        if config_path.exists() {
            let contents = fs::read_to_string(&config_path)
                .with_context(|| format!("Failed to read config file: {:?}", config_path))?;

            let config: Config = toml::from_str(&contents)
                .with_context(|| format!("Failed to parse config file: {:?}", config_path))?;

            Ok(config)
        } else {
            // Create default config file
            let default_config = Config::default();
            default_config.create_default_config()?;
            Ok(default_config)
        }
    }

    /// Creates a default configuration file at ~/.ecs-voyager/config.toml
    ///
    /// This function will create the config directory if it doesn't exist, then
    /// write a default configuration file with helpful comments.
    ///
    /// # Returns
    /// Returns `Ok(())` if successful, or an error if file operations fail
    ///
    /// # Errors
    /// This function will return an error if:
    /// - Directory creation fails
    /// - File write operations fail
    pub fn create_default_config(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;
        let config_path = Self::config_file_path()?;

        // Create directory if it doesn't exist
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir)
                .with_context(|| format!("Failed to create config directory: {:?}", config_dir))?;
        }

        // Generate default config with comments
        let default_toml = r#"# ECS Voyager Configuration File
# This file is automatically generated with default values.
# You can edit this file to customize ECS Voyager's behavior.

[aws]
# Default AWS region to use (optional)
# If not specified, uses AWS SDK's default resolution (env vars, ~/.aws/config, etc.)
# region = "us-east-1"

# AWS profile to use from ~/.aws/credentials (optional)
# If not specified, uses the default profile
# profile = "default"

[behavior]
# Enable automatic refresh of data
auto_refresh = true

# Interval in seconds between automatic refreshes
refresh_interval = 30

# Default view to show on startup
# Options: "clusters", "services", "tasks"
default_view = "clusters"

[ui]
# Color theme (for future use)
# Options: "dark", "light"
theme = "dark"
"#;

        fs::write(&config_path, default_toml)
            .with_context(|| format!("Failed to write config file: {:?}", config_path))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let config = Config::default();
        assert_eq!(config.behavior.auto_refresh, true);
        assert_eq!(config.behavior.refresh_interval, 30);
        assert_eq!(config.behavior.default_view, "clusters");
        assert_eq!(config.ui.theme, "dark");
        assert!(config.aws.region.is_none());
        assert!(config.aws.profile.is_none());
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = r#"
[aws]
region = "us-west-2"
profile = "production"

[behavior]
auto_refresh = false
refresh_interval = 60
default_view = "services"

[ui]
theme = "light"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.aws.region, Some("us-west-2".to_string()));
        assert_eq!(config.aws.profile, Some("production".to_string()));
        assert_eq!(config.behavior.auto_refresh, false);
        assert_eq!(config.behavior.refresh_interval, 60);
        assert_eq!(config.behavior.default_view, "services");
        assert_eq!(config.ui.theme, "light");
    }

    #[test]
    fn test_partial_config() {
        let toml_str = r#"
[aws]
region = "eu-west-1"
"#;

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.aws.region, Some("eu-west-1".to_string()));
        assert_eq!(config.aws.profile, None);
        // Should use defaults for other fields
        assert_eq!(config.behavior.auto_refresh, true);
        assert_eq!(config.behavior.refresh_interval, 30);
    }
}
