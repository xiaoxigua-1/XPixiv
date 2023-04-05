use std::{fs::File, io::Write};

use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct DownloadInfo {
    pub title: String,
    pub progress: u64,
}

#[derive(Serialize, Deserialize, Clone)]
pub enum GroupType {
    Artwork,
    Author,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConfigData {
    pub output: String,
    pub group_type: Option<GroupType>,
}

impl DownloadInfo {
    pub fn new(title: String) -> Self {
        Self { title, progress: 0 }
    }
}

impl ConfigData {
    pub fn save(&self) -> Self {
        let mut file = File::create("./config.toml").unwrap();
        let toml_str = toml::to_string(self).unwrap();
        file.write_all(toml_str.as_bytes()).unwrap();
        self.clone()
    }
}

impl Default for ConfigData {
    fn default() -> Self {
        Self {
            output: "./images".to_string(),
            group_type: None,
        }
    }
}
