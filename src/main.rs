//! ECS Voyager - A Terminal User Interface for AWS ECS Management
//!
//! This application provides an interactive terminal interface for managing AWS ECS clusters,
//! services, tasks, and viewing CloudWatch logs. It uses the ratatui framework for rendering
//! and the AWS SDK for Rust for cloud integration.

mod app;
mod ui;
mod aws;
mod config;

use anyhow::Result;
use app::{App, AppState};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
};
use std::io;

/// Application entry point.
///
/// Initializes the terminal, creates the app instance, runs the main event loop,
/// and ensures proper cleanup on exit. The terminal is restored to its original
/// state even if an error occurs.
///
/// # Returns
/// Returns `Ok(())` on successful execution or an error if terminal initialization,
/// app creation, or cleanup fails.
///
/// # Errors
/// This function will return an error if:
/// - Terminal initialization fails (raw mode, alternate screen)
/// - AWS client initialization fails
/// - Terminal restoration fails on cleanup
#[tokio::main]
async fn main() -> Result<()> {
    // Setup terminal
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Create app
    let mut app = App::new().await?;

    // Run app
    let res = run_app(&mut terminal, &mut app).await;

    // Restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    if let Err(err) = res {
        eprintln!("Error: {:?}", err);
    }

    Ok(())
}

/// Runs the main application event loop.
///
/// Handles terminal rendering, keyboard input processing, and periodic data refresh.
/// The loop continues until the user presses 'q' to quit. Input handling differs
/// based on whether search mode is active.
///
/// # Arguments
/// * `terminal` - Mutable reference to the terminal backend for rendering
/// * `app` - Mutable reference to the application state
///
/// # Returns
/// Returns `Ok(())` when the user quits normally, or an error if rendering,
/// event reading, or data operations fail.
///
/// # Errors
/// This function will return an error if:
/// - Terminal drawing fails
/// - Event polling or reading fails
/// - AWS API calls during refresh/select/describe operations fail
/// - CloudWatch logs retrieval fails
///
/// # Event Handling
/// - In search mode: Handles character input, backspace, enter, and escape
/// - In normal mode: Handles navigation (↑↓/jk), selection (Enter), view switching (1-3),
///   refresh (r), describe (d), logs (l), actions (x), and help (?)
async fn run_app<B: ratatui::backend::Backend>(
    terminal: &mut Terminal<B>,
    app: &mut App,
) -> Result<()> {
    loop {
        terminal.draw(|f| ui::draw(f, app))?;

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    // Handle search mode input separately
                    if app.search_mode {
                        match key.code {
                            KeyCode::Char(c) => app.update_search(c),
                            KeyCode::Backspace => app.delete_search_char(),
                            KeyCode::Enter => app.exit_search_mode(),
                            KeyCode::Esc => app.clear_search(),
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Char('q') => return Ok(()),
                            KeyCode::Char('/') => {
                                // Only enable search in list views
                                match app.state {
                                    AppState::Clusters | AppState::Services | AppState::Tasks => {
                                        app.enter_search_mode();
                                    }
                                    _ => {}
                                }
                            }
                            KeyCode::Char('?') => app.toggle_help(),
                            KeyCode::Char('1') => app.set_view(AppState::Clusters),
                            KeyCode::Char('2') => app.set_view(AppState::Services),
                            KeyCode::Char('3') => app.set_view(AppState::Tasks),
                            KeyCode::Up | KeyCode::Char('k') => app.previous(),
                            KeyCode::Down | KeyCode::Char('j') => app.next(),
                            KeyCode::Enter => app.select().await?,
                            KeyCode::Esc | KeyCode::Char('h') => {
                                if !app.search_query.is_empty() {
                                    app.clear_search();
                                } else {
                                    app.back();
                                }
                            }
                            KeyCode::Char('r') => app.refresh().await?,
                            KeyCode::Char('d') => app.describe().await?,
                            KeyCode::Char('l') => app.view_logs().await?,
                            KeyCode::Char('t') => app.toggle_auto_tail(),
                            KeyCode::Char('x') => app.execute_action().await?,
                            _ => {}
                        }
                    }
                }
            }
        }

        // Auto-refresh data periodically
        if app.should_refresh() {
            app.refresh().await?;
        }
    }
}
