use clap::Parser;
use cli::{rank_downloader, Cli, Commands};
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{io, time::Duration};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    widgets::ListItem,
    Terminal,
};
use ui_util::{AppState, RankState};

mod cli;
mod ui_util;

#[tokio::main]
async fn main() -> pixiv::Result<()> {
    match std::env::args().len() {
        1 => {
            tui().unwrap();
        }
        _ => cli().await?,
    };

    Ok(())
}

async fn cli() -> pixiv::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Rank(args) => rank_downloader(args).await?,
        _ => {}
    }

    Ok(())
}

fn tui() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app_state = AppState::new(vec![
        ListItem::new("Rank Downloader"),
        ListItem::new("Artworks Downloader"),
    ]);
    let mut rank_downloader_state = RankState::new(vec![
        "daily", "weekly", "monthly", "rookie", "original", "daily_ai", "male", "femal",
    ]);

    app_state.init();

    loop {
        if crossterm::event::poll(Duration::from_millis(100))? {
            if let Event::Key(key) = read()? {
                match key.code {
                    KeyCode::Char('q') => break,
                    _ => {
                        match key.code {
                            KeyCode::Left | KeyCode::Right => app_state.focus = !app_state.focus,
                            _ => {}
                        }

                        if app_state.focus {
                            ui_util::update(&mut app_state, key.code);
                        } else {
                            match app_state.current() {
                                0 => {
                                    ui_util::rank_downloader_update(
                                        &mut rank_downloader_state,
                                        key,
                                    );
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([Constraint::Percentage(15), Constraint::Percentage(80)].as_ref())
                .split(f.size());
            ui_util::ui(f, &mut app_state, chunks[0]);

            match app_state.current() {
                0 => {
                    ui_util::rank_downloader_ui(
                        f,
                        &mut app_state,
                        &mut rank_downloader_state,
                        chunks[1],
                    );
                }
                _ => {}
            }
        })?;
    }

    // restore terminal
    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}
