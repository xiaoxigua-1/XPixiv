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
    /// rank start at index
    #[arg(default_value_t = 0, short = 's', long)]
    start: usize,

    /// rank end at index
    #[arg(default_value_t = 500, short = 'e', long)]
    end: usize,

    /// output path
    #[arg(default_value_t = String::from("./"), short = 'p', long)]
    path: String,

    /// rank type
    #[arg(default_value_t = String::from("daily"), short = 't', long)]
    rank_type: String,

    /// output folder group
    #[arg(short = 'g', long)]
    path_group: Option<String>,
}

pub fn parse_agrs_type(s: &str) -> RankType {
    use RankType::*;

    match s {
        "daily" => Daily,
        "weekly" => Weekly,
        "monthly" => Monthly,
        "rookie" => Rookie,
        "original" => Original,
        "daily_ai" => DailyAI,
        "male" => Male,
        "female" => Female,
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
            let images = get_artworks_data(id.illust_id).await?;
            let mut path = PathBuf::from(&args.path);
            if let Some(group) = &args.path_group {
                match group.as_str() {
                    "author" => path.push(&format!("{}/", images.user_name)),
                    _ => {}
                }
            }

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
                let title = images.title.clone();
                tokio::spawn(async move {
                    let progress_fn = |now_size, total_size| {
                        let progress: f64 = now_size as f64 / total_size as f64;
                        print!("\u{001b}[1000D\u{001b}[2K");
                        print!("{}-{} Downloading |", title, index);
                        for i in 0..10 {
                            let c = if (i as f64) > 10 as f64 * progress {
                                ' '
                            } else {
                                '#'
                            };
                            print!("{}", c);
                        }
                        print!("| {}%", (progress * 100.0) as u64);
                        std::io::stdout().flush().unwrap();
                    };
                    downloader(path_clone.join(&image_name), url_clone, progress_fn)
                        .await
                        .unwrap();
                })
                .await
                .unwrap();
            }
        } else {
            break;
        }
    }

    Ok(())
}
