use serde::{Serialize, Deserialize};

#[derive(Clone)]
pub struct DownloadInfo {
    pub title: String,
    pub progress: u64,
}

#[derive(Serialize, Deserialize)]
pub enum GroupType {
    Artwork,
    Author,
}

#[derive(Serialize, Deserialize)]
pub struct ConfigData {
    pub output: String,
    pub group_type: Option<GroupType>,
}

impl DownloadInfo {
    pub fn new(title: String) -> Self {
        Self { title, progress: 0 }
    }
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            output: "./images".to_string(),
            group_type: None
        }
    }
}
