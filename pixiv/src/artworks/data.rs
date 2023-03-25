use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct Artworks {
    pub illust: HashMap<String, ArtworksData>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ArtworksData {
    #[serde(skip)]
    #[serde(default)]
    pub images: Vec<String>,
    pub title: String,
    pub description: String,
    #[serde(rename = "userName")]
    pub user_name: String
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArtworkPages {
    pub body: Vec<ArtworkPagesData>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ArtworkPagesData {
    pub urls: HashMap<String, String>
}
