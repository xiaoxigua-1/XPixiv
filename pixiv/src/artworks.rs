use scraper::{Html, Selector};

use crate::data::{Api, ArtworkPagesData, ArtworksData, Illust};
use std::collections::HashMap;

pub async fn get_artworks_data(id: usize) -> reqwest::Result<ArtworksData> {
    let mut images = get_artworks_image_data(id).await?;
    let html = reqwest::get(format!("https://www.pixiv.net/artworks/{}", id))
        .await?
        .error_for_status()?
        .text()
        .await?;
    let parser = Html::parse_document(&html);
    let selector = Selector::parse("#meta-preload-data").unwrap();
    let element = parser.select(&selector).next().unwrap();
    let json_str = element.value().attr("content").unwrap();
    let data: Illust<HashMap<String, ArtworksData>> = serde_json::from_str(json_str).unwrap();

    let mut artworks_data = data.illust.get(&id.to_string()).unwrap().clone();
    artworks_data.images.append(&mut images);

    Ok(artworks_data)
}

pub async fn get_artworks_image_data(id: usize) -> reqwest::Result<Vec<String>> {
    let mut data = reqwest::get(format!("https://www.pixiv.net/ajax/illust/{}/pages", id))
        .await?
        .error_for_status()?
        .json::<Api<Vec<ArtworkPagesData>>>()
        .await?;
    let images = data
        .body
        .iter_mut()
        .map(|image| image.urls.get("original").unwrap().to_string())
        .collect();
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
