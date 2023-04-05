#[cfg(feature = "cli")]
use clap::Parser;
#[cfg(feature = "cli")]
use cli::{artwork_download, rank_downloader, user_download, Cli, Commands};
#[cfg(feature = "tui")]
use crossterm::{
    event::{read, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
#[cfg(feature = "tui")]
use std::{io, time::Duration};
#[cfg(feature = "tui")]
use tui::{backend::CrosstermBackend, widgets::ListItem, Terminal};
#[cfg(feature = "tui")]
use tui_util::AppState;

#[cfg(feature = "cli")]
mod cli;
#[cfg(feature = "tui")]
mod tui_util;

#[tokio::main]
async fn main() -> x_pixiv_lib::Result<()> {
    match std::env::args().len() {
        1 => {
            #[cfg(feature = "tui")]
            tui().unwrap();
        }
        _ =>
        {
            #[cfg(feature = "cli")]
            cli().await?
        }
    };

    Ok(())
}

#[cfg(feature = "cli")]
async fn cli() -> x_pixiv_lib::Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Rank(args) => rank_downloader(args).await?,
        Commands::Artwork(args) => artwork_download(args).await?,
        Commands::User(args) => user_download(args).await?,
    }

    Ok(())
}

#[cfg(feature = "tui")]
fn tui() -> Result<(), io::Error> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;
    let mut app_state = AppState::new(vec![
        ListItem::new("Rank Downloader"),
        ListItem::new("Artworks Downloader"),
        ListItem::new("User Downloader"),
    ]);

    app_state.init();

    loop {
        if crossterm::event::poll(Duration::from_millis(100))? {
            let event = read()?;

            app_state.update(&event);

            if let Event::Key(key) = event {
                match key.code {
                    KeyCode::Char('q') => break,
                    _ => match key.code {
                        code if (code == KeyCode::Left || code == KeyCode::Right)
                            && !app_state.config_open =>
                        {
                            app_state.focus = !app_state.focus
                        }
                        KeyCode::Esc => app_state.config_open = !app_state.config_open,
                        _ => {}
                    },
                }
            }
        }

        terminal.draw(|f| {
            app_state.ui(f);
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
