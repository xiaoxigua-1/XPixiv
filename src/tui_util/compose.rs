use crossterm::event::Event;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, io::Stdout};
use uuid::Uuid;
use crate::tui_util::data::DownloadInfo;
use tui::{backend::CrosstermBackend, layout::Rect, Frame};

pub trait Compose {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, focus: bool, area: Rect);

    fn update(&mut self, event: &Event, download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>);

    fn init(&mut self);
}
