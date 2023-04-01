use std::{
    path::PathBuf,
    sync::{Arc, Mutex},
    time::Duration,
};

use clap::{Args, Parser, Subcommand};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::time::sleep;
use x_pixiv_lib::{
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
    #[arg(default_value_t = 1, short = 's', long)]
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

pub async fn rank_downloader(args: &RankArgs) -> x_pixiv_lib::Result<()> {
    let mut rank = Rank::new(
        parse_agrs_type(&args.rank_type),
        false,
        args.start..args.end,
    );
    let progress_manager = MultiProgress::new();
    progress_manager.set_alignment(indicatif::MultiProgressAlignment::Bottom);
    let progress_manager = Arc::new(Mutex::new(progress_manager));
    let progress_style = ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] {bar:40.cyan/blue} {pos:>7}/{len:7} ({eta})",
    )
    .unwrap()
    .progress_chars("##-");
    let total_progress = progress_manager
        .lock()
        .unwrap()
        .add(ProgressBar::new((args.end - args.start) as u64 + 1));
    total_progress.set_style(progress_style.clone());
    total_progress.enable_steady_tick(Duration::from_millis(100));

    loop {
        total_progress.inc(1);
        if let Some(id) = rank.next().await? {
            let images = get_artworks_data(id.illust_id).await?;
            let mut path = PathBuf::from(&args.path);
            let mut download_qu = vec![];
            if let Some(group) = &args.path_group {
                match group.as_str() {
                    "author" => path.push(&format!("{}/", images.user_name)),
                    "title" => path.push(&format!("{}/", images.title)),
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
                let clone_progress_manager = progress_manager.clone();
                let task = tokio::spawn(async move {
                    let task_progress: Arc<Mutex<ProgressBar>> =
                        Arc::new(Mutex::new(ProgressBar::hidden()));
                    let clone_progress = task_progress.clone();
                    let clone_two_p = task_progress.clone();
                    let progress_fn = |now_size, _| {
                        clone_two_p.lock().unwrap().set_position(now_size);
                    };

                    downloader(path_clone.join(&image_name), url_clone, progress_fn, |total_size| {
                        let progress = ProgressBar::new(total_size);
                        *clone_progress.lock().unwrap() = clone_progress_manager.lock().unwrap().add(progress);
                        clone_progress.lock().unwrap().set_style(ProgressStyle::with_template("{spinner:.green} [{msg}] [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));
                        clone_progress.lock().unwrap().set_message(format!("{}-{}", title, index));
                    })
                        .await
                        .unwrap();
                    task_progress.lock().unwrap().finish_and_clear();
                });
                download_qu.push(task);
                sleep(Duration::from_millis(10)).await;
            }

            for task in download_qu {
                if let Err(_) = task.await {};
            }
        } else {
            break;
        }
    }

    total_progress.finish_with_message("Deno");

    Ok(())
}
