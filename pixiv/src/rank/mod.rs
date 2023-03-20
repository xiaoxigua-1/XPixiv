use std::ops::Range;

mod rank_list;

const RANK_URI: &str = "https://www.pixiv.net/ranking.php";

enum RankType {
    Day,
    Week,
    Month,
    Rookie,
    Original,
    DailyAI,
    Male,
    Femal
}

struct Rank {
    rank_type: RankType,
    is_r18: bool,
    download_range: Range<usize>
}

impl Rank {
    fn new(rank_type: RankType, is_r18: bool, download_range: Range<usize>) -> Self {
        Self {
            rank_type,
            is_r18,
            download_range
        }
    }

    fn get_page_uri(&self) {
        
    }
}
