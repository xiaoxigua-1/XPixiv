use crate::data::Illust;

use std::collections::HashMap;

pub struct User {
    id: usize,
}

impl User {
    pub fn new(id: usize) -> Self {
        Self { id }
    }

    pub async fn get_artworks(&self) -> reqwest::Result<Vec<usize>> {
        let data = reqwest::get(format!(
            "https://www.pixiv.net/ajax/user/{}/profile/all",
            self.id
        ))
        .await?
        .json::<Illust<HashMap<usize, Option<bool>>>>()
        .await?;
        let images = data
            .illust
            .keys()
            .map(|k| k.clone())
            .collect::<Vec<usize>>();

        Ok(images)
    }
}
