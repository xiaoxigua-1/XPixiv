use std::{
    fs::{create_dir_all, File},
    io::Write,
    path::PathBuf,
};

use futures_util::StreamExt;

pub async fn downloader<F, FT>(
    path: PathBuf,
    url: String,
    progress: F,
    total: FT,
) -> reqwest::Result<()>
where
    F: Fn(u64, u64),
    FT: Fn(u64),
{
    create_dir_all(path.parent().unwrap()).unwrap();

    let client = reqwest::Client::new();
    let mut file = File::create(path);
    let response = client
        .get(url)
        .header("referer", "https://www.pixiv.net/")
        .send()
        .await?;
    let Some(total_size) = response.content_length() else {
        return Ok(());
    };
    total(total_size);
    if let Ok(file) = &mut file {
        let mut byte_stream = response.bytes_stream();
        let mut now_size: u64 = 0;
        while let Some(byte) = byte_stream.next().await {
            let byte = byte?;
            now_size += byte.len() as u64;
            progress(now_size, total_size);
            file.write(&byte[..]).unwrap();
        }
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::downloader;

    #[tokio::test]
    async fn downloader_test() -> reqwest::Result<()> {
        downloader(
            PathBuf::new(),
            "https://i.pximg.net/img-original/img/2023/03/23/00/05/02/106465672_p0.png".to_string(),
            |_, _| {},
            |_| {},
        )
        .await?;
        Ok(())
    }
}
