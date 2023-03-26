use clap::Parser;
use cli::{Cli, Commands, rank_downloader};

mod cli;

#[tokio::main]
async fn main() -> pixiv::Result<()> {
    if std::env::args().len() == 1 {
        tui(); 
    } else {
        let cli = Cli::parse();
        match &cli.command {
            Commands::Rank(args) => rank_downloader(args).await?, 
            _ => {}
        }
    }

    Ok(())
}

fn tui() {
    println!("Starting TUI");
}
