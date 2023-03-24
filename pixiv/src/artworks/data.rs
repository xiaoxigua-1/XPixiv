use std::collections::HashMap;

use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
pub struct ArtworkPages {
    pub body: Vec<ArtworkPagesData>
}

#[derive(Serialize, Deserialize)]
pub struct ArtworkPagesData {
    pub urls: HashMap<String, String>
}
