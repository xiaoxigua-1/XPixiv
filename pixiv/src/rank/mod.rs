use std::{fmt::Display, ops::Range};

use self::rank_list::{Content, RankList};

pub mod rank_list;

const RANK_URI: &str = "https://www.pixiv.net/ranking.php";

pub enum RankType {
    Daily,
    Weekly,
    Monthly,
    Rookie,
    Original,
    DailyAI,
    Male,
    Female,
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
                Female => "female",
            }
        )
    }
}

pub struct Rank {
    rank_type: RankType,
    is_r18: bool,
    download_range: Range<usize>,
    queue: Vec<Content>,
    current: usize,
}

impl Rank {
    pub fn new(rank_type: RankType, is_r18: bool, download_range: Range<usize>) -> Self {
        let start = download_range.start;
        Self {
            rank_type,
            is_r18,
            download_range,
            queue: vec![],
            current: start,
        }
    }

    fn get_url(&self, page: usize) -> String {
        let is_r18 = if self.is_r18 { "_r18" } else { "" };
        format!(
            "{}?mode={}{}&format=json&p={}",
            RANK_URI, self.rank_type, is_r18, page
        )
    }

    pub async fn next(&mut self) -> reqwest::Result<Option<Content>> {
        self.current += 1;
        if self.current - 1 > self.download_range.end {
            Ok(None)
        } else if self.queue.len() == 0 {
            let response = reqwest::get(self.get_url((self.current / 50) + 1)).await?;
            if response.status() == 200 {
                let data = response.json::<RankList>().await?;
                let list = &mut if self.download_range.start > self.current {
                    data.contents[(self.download_range.start - self.current)..].to_vec()
                } else {
                    data.contents
                };
                let current_id = list.remove(0);
                self.queue.append(list);
                Ok(Some(current_id))
            } else {
                Ok(None)
            }
        } else {
            Ok(Some(self.queue.remove(0)))
        }
    }
}

#[cfg(test)]
mod rank_test {
    use super::Rank;

    #[tokio::test]
    async fn test() {
        let mut rank = Rank::new(super::RankType::Daily, false, 44..66);
        let mut index = 0;
        loop {
            if let Some(_) = rank.next().await.unwrap() {
                index += 1;
            } else {
                break;
            }
        }
        println!("{} {}", index, 23);
    }
}
