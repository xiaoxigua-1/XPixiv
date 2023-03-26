use std::{path::PathBuf, fs::{File, create_dir_all}, io::Write};

pub async fn downloader(path: PathBuf, url: String) -> reqwest::Result<()> {
    create_dir_all(path.parent().unwrap()).unwrap();
   
    let client = reqwest::Client::new();
    let mut file = File::create(path);
    let response = client.get(url).header("referer", "https://www.pixiv.net/").send().await?;
    
    if let Ok(file) = &mut file {
        let bytes = &response.bytes().await?[..]; 
        file.write(bytes).unwrap();
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::path::PathBuf;

    use super::downloader;

    #[tokio::test]
    async fn downloader_test() -> reqwest::Result<()> {
        downloader(PathBuf::new(), "https://i.pximg.net/img-original/img/2023/03/23/00/05/02/106465672_p0.png".to_string()).await?;
        Ok(())
    }
}
