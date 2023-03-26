use std::{fmt::Display, ops::Range};

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
    Femal,
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
                Femal => "femal",
            }
        )
    }
}

pub struct Rank {
    rank_type: RankType,
    is_r18: bool,
    download_range: Range<usize>,
    queue: Vec<usize>,
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
            current: (start / 50) * 50 + 1,
        }
    }

    fn get_url(&self, page: usize) -> String {
        let is_r18 = if self.is_r18 { "_r18" } else { "" };
        format!(
            "{}?mode={}{}&format=json&p={}",
            RANK_URI, self.rank_type, is_r18, page
        )
    }

    pub async fn next(&mut self) -> reqwest::Result<Option<usize>> {
        if self.current > self.download_range.end {
            Ok(None)
        } else if self.queue.len() == 0 {
            let response = reqwest::get(self.get_url((self.current / 50) + 1)).await?;
            if response.status() == 200 {
                self.current = self.download_range.start;
                let mut data = response.json::<RankList>().await?;
                let artworks_list: Vec<usize> = data
                    .contents
                    .iter_mut()
                    .map(|content| content.illust_id)
                    .collect();
                let list =
                    &mut artworks_list[(self.download_range.start - self.current)..].to_vec();
                let current_id = list.remove(0);
                self.queue.append(list);
                Ok(Some(current_id))
            } else {
                Ok(None)
            }
        } else {
            self.current += 1;
            Ok(Some(self.queue.remove(0)))
        }
    }
}

#[cfg(test)]
mod rank_test {
    use super::Rank;

    #[tokio::test]
    async fn test() {
        let mut rank = Rank::new(super::RankType::Daily, false, 44..50);
        loop {
            if let Some(next) = rank.next().await.unwrap() {
                println!("{}", next);
            } else {
                break;
            }
        }
    }
}
