use std::path::PathBuf;

pub async fn downloader(path: PathBuf, download_list: Vec<String>) -> reqwest::Result<()> {
    let client = reqwest::Client::new();

    for url in download_list {
        let response = client.get(url).header("referer", "https://www.pixiv.net/").send().await?;
        println!("{:?}", response.bytes().await?); 
    }
    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::downloader;

    #[tokio::test]
    async fn downloader_test() -> reqwest::Result<()> {
        downloader(PathBuf::new(), vec!["https://i.pximg.net/img-original/img/2023/03/23/00/05/02/106465672_p0.png".to_string()]).await?;
        Ok(())
    }
}
