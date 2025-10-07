//! ASCII chart rendering module for metrics visualization.
//!
//! This module provides functions to render time-series data as ASCII charts
//! suitable for terminal display. Supports line charts with configurable dimensions,
//! labels, and value ranges.

use ratatui::{
    style::{Color, Modifier, Style},
    text::{Line, Span},
};

/// Represents a single datapoint for charting.
#[derive(Debug, Clone)]
pub struct ChartDatapoint {
    /// Timestamp of the datapoint (Unix timestamp in seconds, for future use)
    #[allow(dead_code)]
    pub timestamp: i64,
    /// Value to plot
    pub value: f64,
}

/// Configuration for rendering an ASCII chart.
pub struct ChartConfig {
    /// Width of the chart in characters
    pub width: usize,
    /// Height of the chart in characters (excluding labels)
    pub height: usize,
    /// Minimum value for Y-axis (auto-calculated if None)
    pub min_value: Option<f64>,
    /// Maximum value for Y-axis (auto-calculated if None)
    pub max_value: Option<f64>,
    /// Color for the chart line
    pub line_color: Color,
    /// Show Y-axis labels
    pub show_y_labels: bool,
}

impl Default for ChartConfig {
    fn default() -> Self {
        Self {
            width: 60,
            height: 10,
            min_value: None,
            max_value: None,
            line_color: Color::Cyan,
            show_y_labels: true,
        }
    }
}

/// Renders time-series data as ASCII chart lines for display in ratatui.
///
/// Creates a sparkline-style chart using Unicode block characters to show
/// trends in metric data over time. Handles empty data gracefully.
///
/// # Arguments
/// * `datapoints` - Vector of datapoints to plot (sorted by timestamp)
/// * `config` - Chart configuration options
/// * `title` - Title to display above the chart
///
/// # Returns
/// Vector of ratatui `Line` objects ready to render
///
/// # Examples
/// ```
/// let datapoints = vec![
///     ChartDatapoint { timestamp: 1000, value: 45.2 },
///     ChartDatapoint { timestamp: 2000, value: 52.1 },
///     ChartDatapoint { timestamp: 3000, value: 48.7 },
/// ];
/// let config = ChartConfig::default();
/// let lines = render_chart(&datapoints, &config, "CPU Usage");
/// ```
pub fn render_chart(
    datapoints: &[ChartDatapoint],
    config: &ChartConfig,
    title: &str,
) -> Vec<Line<'static>> {
    let mut lines = Vec::new();

    // Add title
    lines.push(Line::from(vec![Span::styled(
        format!("  {title}"),
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
    )]));

    if datapoints.is_empty() {
        lines.push(Line::from(Span::styled(
            "    No data available",
            Style::default().fg(Color::DarkGray),
        )));
        return lines;
    }

    // Calculate value range
    let values: Vec<f64> = datapoints.iter().map(|dp| dp.value).collect();
    let min_val = config
        .min_value
        .unwrap_or_else(|| values.iter().fold(f64::INFINITY, |a, &b| a.min(b)).floor());
    let max_val = config.max_value.unwrap_or_else(|| {
        values
            .iter()
            .fold(f64::NEG_INFINITY, |a, &b| a.max(b))
            .ceil()
    });

    // Avoid division by zero
    let range = if (max_val - min_val).abs() < 0.001 {
        1.0
    } else {
        max_val - min_val
    };

    // Sample datapoints to fit chart width
    let sampled_values = sample_datapoints(&values, config.width);

    // Render chart rows from top to bottom
    for row in 0..config.height {
        // Threshold represents the TOP of this row (for Y-axis label)
        let row_top = max_val - (row as f64 * range / config.height as f64);
        // But we check against the BOTTOM of this row for filled bar charts
        let row_bottom = max_val - ((row + 1) as f64 * range / config.height as f64);

        let mut row_chars = String::new();

        // Add Y-axis label (show the top of this row)
        if config.show_y_labels {
            row_chars.push_str(&format!("  {row_top:5.1}│ "));
        } else {
            row_chars.push_str("  ");
        }

        // Render chart points - draw if value reaches the bottom of this row
        for &value in &sampled_values {
            let char = if value >= row_bottom { '█' } else { ' ' };
            row_chars.push(char);
        }

        lines.push(Line::from(Span::styled(
            row_chars,
            Style::default().fg(config.line_color),
        )));
    }

    // Add X-axis
    if config.show_y_labels {
        let axis_line = format!("       └{}", "─".repeat(config.width));
        lines.push(Line::from(Span::styled(
            axis_line,
            Style::default().fg(Color::DarkGray),
        )));
    }

    lines
}

/// Samples datapoints to fit the target width using averaging.
///
/// If there are more datapoints than width, averages groups of points.
/// If there are fewer datapoints, interpolates or repeats values.
///
/// # Arguments
/// * `values` - Vector of values to sample
/// * `target_width` - Desired number of output values
///
/// # Returns
/// Vector of sampled values with length equal to target_width
fn sample_datapoints(values: &[f64], target_width: usize) -> Vec<f64> {
    if values.is_empty() {
        return vec![0.0; target_width];
    }

    if values.len() <= target_width {
        // If we have fewer points than width, repeat last value
        let mut result = values.to_vec();
        while result.len() < target_width {
            result.push(*values.last().unwrap_or(&0.0));
        }
        result
    } else {
        // Sample by averaging buckets
        let bucket_size = values.len() as f64 / target_width as f64;
        (0..target_width)
            .map(|i| {
                let start = (i as f64 * bucket_size) as usize;
                let end = ((i + 1) as f64 * bucket_size) as usize;
                let bucket = &values[start..end.min(values.len())];
                if bucket.is_empty() {
                    0.0
                } else {
                    bucket.iter().sum::<f64>() / bucket.len() as f64
                }
            })
            .collect()
    }
}

/// Renders a simple sparkline chart (single-line visualization).
///
/// Creates a compact one-line chart using Unicode characters to show
/// trends without taking up much vertical space.
///
/// # Arguments
/// * `values` - Vector of values to plot
/// * `width` - Width of the sparkline in characters
/// * `color` - Color for the sparkline
///
/// # Returns
/// A single ratatui `Line` containing the sparkline
#[allow(dead_code)]
pub fn render_sparkline(values: &[f64], width: usize, color: Color) -> Line<'static> {
    if values.is_empty() {
        return Line::from(Span::styled(" ".repeat(width), Style::default().fg(color)));
    }

    let min_val = values.iter().fold(f64::INFINITY, |a, &b| a.min(b));
    let max_val = values.iter().fold(f64::NEG_INFINITY, |a, &b| a.max(b));

    let range = if (max_val - min_val).abs() < 0.001 {
        1.0
    } else {
        max_val - min_val
    };

    let sampled = sample_datapoints(values, width);
    let chars = "▁▂▃▄▅▆▇█";

    let sparkline: String = sampled
        .iter()
        .map(|&val| {
            let normalized = ((val - min_val) / range * 7.0).round() as usize;
            chars.chars().nth(normalized.min(7)).unwrap_or('▁')
        })
        .collect();

    Line::from(Span::styled(sparkline, Style::default().fg(color)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sample_datapoints_exact_fit() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let sampled = sample_datapoints(&values, 5);
        assert_eq!(sampled.len(), 5);
        assert_eq!(sampled, values);
    }

    #[test]
    fn test_sample_datapoints_downsample() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0];
        let sampled = sample_datapoints(&values, 3);
        assert_eq!(sampled.len(), 3);
        // Should average pairs: (1+2)/2=1.5, (3+4)/2=3.5, (5+6)/2=5.5
        assert!((sampled[0] - 1.5).abs() < 0.01);
        assert!((sampled[1] - 3.5).abs() < 0.01);
        assert!((sampled[2] - 5.5).abs() < 0.01);
    }

    #[test]
    fn test_sample_datapoints_upsample() {
        let values = vec![1.0, 2.0];
        let sampled = sample_datapoints(&values, 5);
        assert_eq!(sampled.len(), 5);
        // Should repeat last value
        assert_eq!(sampled, vec![1.0, 2.0, 2.0, 2.0, 2.0]);
    }

    #[test]
    fn test_sample_datapoints_empty() {
        let values: Vec<f64> = vec![];
        let sampled = sample_datapoints(&values, 10);
        assert_eq!(sampled.len(), 10);
        assert!(sampled.iter().all(|&v| v == 0.0));
    }

    #[test]
    fn test_render_chart_empty_datapoints() {
        let datapoints: Vec<ChartDatapoint> = vec![];
        let config = ChartConfig::default();
        let lines = render_chart(&datapoints, &config, "Test Chart");

        assert!(!lines.is_empty());
        // Should have title + "No data available" message
        assert!(lines.len() >= 2);
    }

    #[test]
    fn test_render_chart_with_data() {
        let datapoints = vec![
            ChartDatapoint {
                timestamp: 1000,
                value: 10.0,
            },
            ChartDatapoint {
                timestamp: 2000,
                value: 20.0,
            },
            ChartDatapoint {
                timestamp: 3000,
                value: 15.0,
            },
        ];
        let config = ChartConfig {
            width: 10,
            height: 5,
            ..Default::default()
        };
        let lines = render_chart(&datapoints, &config, "Test");

        // Should have title + chart rows + axis
        assert!(lines.len() > 5);
    }

    #[test]
    fn test_render_sparkline_empty() {
        let values: Vec<f64> = vec![];
        let line = render_sparkline(&values, 10, Color::Cyan);
        // Should return a line with spaces
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_render_sparkline_with_values() {
        let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let line = render_sparkline(&values, 5, Color::Green);
        assert!(!line.spans.is_empty());
    }

    #[test]
    fn test_chart_config_default() {
        let config = ChartConfig::default();
        assert_eq!(config.width, 60);
        assert_eq!(config.height, 10);
        assert!(config.min_value.is_none());
        assert!(config.max_value.is_none());
        assert!(config.show_y_labels);
    }

    #[test]
    fn test_chart_datapoint_creation() {
        let dp = ChartDatapoint {
            timestamp: 123456,
            value: 42.5,
        };
        assert_eq!(dp.timestamp, 123456);
        assert!((dp.value - 42.5).abs() < 0.001);
    }

    #[test]
    fn test_sample_datapoints_single_value() {
        let values = vec![5.0];
        let sampled = sample_datapoints(&values, 5);
        assert_eq!(sampled.len(), 5);
        assert!(sampled.iter().all(|&v| (v - 5.0).abs() < 0.001));
    }

    #[test]
    fn test_render_chart_min_max_override() {
        let datapoints = vec![ChartDatapoint {
            timestamp: 1000,
            value: 50.0,
        }];
        let config = ChartConfig {
            width: 10,
            height: 5,
            min_value: Some(0.0),
            max_value: Some(100.0),
            ..Default::default()
        };
        let lines = render_chart(&datapoints, &config, "Test");
        assert!(!lines.is_empty());
    }

    #[test]
    fn test_sample_datapoints_large_dataset() {
        let values: Vec<f64> = (0..1000).map(|i| i as f64).collect();
        let sampled = sample_datapoints(&values, 50);
        assert_eq!(sampled.len(), 50);
        // First value should be close to average of first bucket
        assert!(sampled[0] < 20.0);
        // Last value should be close to average of last bucket
        assert!(sampled[49] > 980.0);
    }
}
