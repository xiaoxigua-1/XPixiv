use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
struct RankList {
    content: String,
    contents: Vec<Content>,
    date: String,
    mode: String,
    next: Next,
    next_date: Next,
    page: usize,
    prev: usize,
    prev_date: String,
    rank_total: usize,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(untagged)]
enum Next {
    Str(String),
    Bool(bool)
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(deny_unknown_fields)]
struct Content {
    url: String,
    user_name: String,
    tags: Vec<String>
}
