mod data;

use self::data::ArtworkPages;

pub async fn get_artworks_data(id: usize) -> reqwest::Result<Vec<String>> {
    let mut data = reqwest::get(format!("https://www.pixiv.net/ajax/illust/{}/pages", id)).await?.json::<ArtworkPages>().await?;
    let images = data.body.iter_mut().map(|image| { image.urls.get("original").unwrap().to_string() }).collect();
    Ok(images)
}

#[cfg(test)]
mod test {
    use super::get_artworks_data;

    #[tokio::test]
    async fn test_get_artworks_data() -> std::io::Result<()> {
        let s = get_artworks_data(106483793).await.unwrap();
        println!("{:?}", s);
        Ok(())
    }
}
