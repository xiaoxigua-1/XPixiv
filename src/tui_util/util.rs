use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, Mutex},
};
use uuid::Uuid;
use x_pixiv_lib::{artworks::get_artworks_data, downloader::downloader};

use super::data::{ConfigData, DownloadInfo, GroupType};

pub async fn download(
    download_id: usize,
    download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>,
    config: ConfigData,
) -> x_pixiv_lib::Result<()> {
    let data = get_artworks_data(download_id).await?;
    let mut queue = HashMap::new();

    for (index, url) in data.images.iter().enumerate() {
        let update_download_progress = download_queue.clone();
        let file_name = format!("{}-{}.{}", data.title, index, &url[url.len() - 3..]);
        let mut path = PathBuf::from(config.output.clone());
        let info = DownloadInfo::new(data.title.clone());
        let id = Uuid::new_v4();

        if let Some(group) = &config.group_type {
            let group = match group {
                GroupType::Artwork => format!("{}-{}/", data.title, download_id),
                GroupType::Author => format!("{}/", data.user_name),
            };

            path.push(group);
        }

        download_queue.lock().unwrap().insert(id, info);

        let task = tokio::spawn(downloader(
            path.join(file_name),
            url.clone(),
            move |now_size, total_size| {
                let mut write_update = update_download_progress.lock().unwrap();
                let mut info = write_update[&id].clone();
                info.progress = ((now_size as f64 / total_size as f64) * 100.0) as u64;
                write_update.insert(id, info);
            },
            |_| {},
        ));

        queue.insert(id, task);
    }

    for (id, task) in queue {
        task.await.unwrap()?;
        download_queue.lock().unwrap().remove(&id);
    }

    Ok(())
}
