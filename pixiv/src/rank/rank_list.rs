use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct RankList {
    pub contents: Vec<Content>,
    pub date: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Content {
    pub illust_id: usize,
    pub url: String,
    pub user_name: String,
    pub tags: Vec<String>,
}
