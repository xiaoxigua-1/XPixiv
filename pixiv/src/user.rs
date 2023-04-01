use crate::data::{Api, Illusts};

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
        .json::<Api<Illusts<HashMap<usize, Option<bool>>>>>()
        .await?;
        let images = data
            .body
            .illusts
            .keys()
            .map(|k| k.clone())
            .collect::<Vec<usize>>();

        Ok(images)
    }
}

#[cfg(test)]
mod test {
    use super::User;

    #[tokio::test]
    async fn test() {
        let images = User::new(3115085).get_artworks().await.unwrap();
        println!("{:?}", images);
    }
}
