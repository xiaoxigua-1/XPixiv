#[derive(Clone)]
pub struct DownloadInfo {
    pub title: String,
    pub progress: u64,
}

impl DownloadInfo {
    pub fn new(title: String) -> Self {
        Self { title, progress: 0 }
    }
}
