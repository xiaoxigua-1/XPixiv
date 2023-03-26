use std::path::PathBuf;

use clap::{Subcommand, Parser, Args};
use pixiv::{rank::{Rank, RankType}, artworks::get_artworks_data, downloader::downloader};

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
    path_group: Option<String>
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
        _ => Daily
    }
}

pub async fn rank_downloader(args: &RankArgs) -> pixiv::Result<()> {
    let mut rank = Rank::new(parse_agrs_type(&args.rank_type), false, args.start..args.end);
    loop {
        let next = rank.next().await?;
        if let Some(id) = next {
            let images = get_artworks_data(id).await?;
            let mut path = PathBuf::from(&args.path);
            if let Some(group) = &args.path_group {
                match group.as_str() {
                    "author" => path.push(&format!("{}/", images.user_name)),
                    _ => {}
                }
            }
            for (index, url) in images.images.iter().enumerate() {
                let image_name = format!("{}-{}-{}.{}", images.title, id, index, &url[url.len() - 3..]);
                downloader(path.join(&image_name), url.clone()).await?;
            }
        } else {
            break;
        }
    }

    Ok(())
}
