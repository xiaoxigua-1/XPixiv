use crate::tui_util::data::DownloadInfo;
use crossterm::event::Event;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, io::Stdout};
use tui::{backend::CrosstermBackend, layout::Rect, Frame};
use uuid::Uuid;

use super::data::ConfigData;

pub trait Compose {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, focus: bool, area: Rect);

    fn update(
        &mut self,
        event: &Event,
        download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>,
        config: ConfigData,
    );

    fn init(&mut self);
}
