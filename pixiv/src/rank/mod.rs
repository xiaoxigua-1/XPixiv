use std::{ops::Range, fmt::Display};

use crate::artworks::get_artworks_data;

use self::rank_list::RankList;

mod rank_list;

const RANK_URI: &str = "https://www.pixiv.net/ranking.php";

pub enum RankType {
    Daily,
    Weekly,
    Monthly,
    Rookie,
    Original,
    DailyAI,
    Male,
    Femal
}

impl Display for RankType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use RankType::*;

        write!(
            f,
            "{}",
            match self {
                Daily => "daily",
                Weekly => "weekly",
                Monthly => "monthly",
                Rookie => "rookie",
                Original => "original",
                DailyAI => "daily_ai",
                Male => "male",
                Femal => "femal"
            }
        )   
    }
}

pub struct Rank {
    rank_type: RankType,
    is_r18: bool,
    download_range: Range<usize>,
    download_list: Vec<usize>,
}

impl Rank {
    pub fn new(rank_type: RankType, is_r18: bool, download_range: Range<usize>) -> Self {
        Self {
            rank_type,
            is_r18,
            download_range,
            download_list: vec![],
        }
    }

    fn get_url(&self, page: usize) -> String {
        let is_r18 = if self.is_r18 { "_r18" } else { "" };
        format!("{}?mode={}{}&format=json&p={}", RANK_URI, self.rank_type, is_r18, page) 
    }

    pub async fn get_download_list(&mut self) -> reqwest::Result<Vec<String>> {
        let mut download_image_list: Vec<String> = vec![];

        for p in (self.download_range.start / 50)..(self.download_range.end / 50) {
            let response = reqwest::get(self.get_url(p + 1)).await?;
            if response.status() == 200 {
                let mut data = response.json::<RankList>().await?;
                let mut download_list: Vec<usize> = data.contents.iter_mut().map(|content| { content.illust_id }).collect();
                self.download_list.append(&mut download_list);
            } else {
                break;
            }
        }

        for id in &self.download_list {
            download_image_list.append(&mut get_artworks_data(id.clone()).await?);
        }

        Ok(download_image_list)
    }
}

#[cfg(test)]
mod rank_test {
    use super::Rank;

    #[tokio::test]
    async fn test() {
        let mut rank = Rank::new(super::RankType::Daily, false, 0..50);
        let list = rank.get_download_list().await.unwrap();

        for i in list {
            println!("{}", &i);
        }
    }
}
