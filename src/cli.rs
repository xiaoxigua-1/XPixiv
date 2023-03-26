use std::{io::Write, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use pixiv::{
    artworks::get_artworks_data,
    downloader::downloader,
    rank::{Rank, RankType},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Rank(RankArgs),
    Artwork,
}

#[derive(Args, Debug)]
pub struct RankArgs {
    #[arg(default_value_t = 0, short = 's', long)]
    start: usize,

    #[arg(default_value_t = 500, short = 'e', long)]
    end: usize,

    #[arg(default_value_t = String::from("./"), short = 'p', long)]
    path: String,

    #[arg(default_value_t = String::from("daily"), short = 't', long)]
    rank_type: String,

    #[arg(short = 'g', long)]
    path_group: Option<String>,
}

fn parse_agrs_type(s: &str) -> RankType {
    use RankType::*;

    match s {
        "daily" => Daily,
        "weekly" => Weekly,
        "monthly" => Monthly,
        "rookie" => Rookie,
        "original" => Original,
        "daily_ai" => DailyAI,
        "male" => Male,
        "femal" => Femal,
        _ => Daily,
    }
}

pub async fn rank_downloader(args: &RankArgs) -> pixiv::Result<()> {
    let mut rank = Rank::new(
        parse_agrs_type(&args.rank_type),
        false,
        args.start..args.end,
    );
    loop {
        if let Some(id) = rank.next().await? {
            let mut qu: Vec<tokio::task::JoinHandle<()>> = vec![];
            let images = get_artworks_data(id.illust_id).await?;
            let mut path = PathBuf::from(&args.path);
            if let Some(group) = &args.path_group {
                match group.as_str() {
                    "author" => path.push(&format!("{}/", images.user_name)),
                    _ => {}
                }
            }

            print!("\r{} Downloading", images.title);

            std::io::stdout().flush().unwrap();
            for (index, url) in images.images.iter().enumerate() {
                let path_clone = path.clone();
                let image_name = format!(
                    "{}-{}-{}.{}",
                    images.title,
                    id.illust_id,
                    index,
                    &url[url.len() - 3..]
                );
                let url_clone = url.clone();
                let task = tokio::spawn(async move {
                    downloader(path_clone.join(&image_name), url_clone)
                        .await
                        .unwrap();
                });
                qu.push(task);
            }

            for task in qu {
                task.await.unwrap();
            }
        } else {
            break;
        }
    }

    Ok(())
}
