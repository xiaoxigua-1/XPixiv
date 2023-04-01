use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize)]
pub struct Api<T> {
    pub body: T,
}

#[derive(Serialize, Deserialize)]
pub struct Illust<T> {
    pub illust: T,
}

/// https://www.pixiv.net/ajax/illust/{id}/pages
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArtworksData {
    #[serde(skip)]
    #[serde(default)]
    pub images: Vec<String>,
    pub title: String,
    pub description: String,
    #[serde(rename = "userName")]
    pub user_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArtworkPagesData {
    pub urls: HashMap<String, String>,
}

/// https://www.pixiv.net/ranking.php??mode={}&format=json&p={}
#[derive(Serialize, Deserialize, Debug)]
pub struct RankList {
    pub contents: Vec<Content>,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub title: String,
    pub illust_id: usize,
    pub url: String,
    pub user_name: String,
    pub tags: Vec<String>,
}

// https://www.pixiv.net/ajax/user/3115085/profile/illusts?ids%5B%5D={id}&work_category=illustManga&is_first_page=1
#[derive(Serialize, Deserialize)]
pub struct ImagesInfo {}
