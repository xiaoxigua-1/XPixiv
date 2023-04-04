use crossterm::event::Event;
use x_pixiv_lib::artworks::get_artworks_data;
use x_pixiv_lib::downloader::downloader;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, io::Stdout};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use uuid::Uuid;

use super::compose::Compose;
use crate::tui_util::data::DownloadInfo;
use crossterm::event::KeyCode;

pub struct ArtworkState {
    input: String,
}

impl ArtworkState {
    pub fn new() -> Self {
        Self {
            input: String::new(),
        }
    }
    
    fn download(&mut self, download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>) {
        if let Ok(id) = self.input.clone().parse::<usize>() {
            tokio::spawn(async move {
                let data = get_artworks_data(id).await.unwrap();
                let mut queue = HashMap::new();
                let path = PathBuf::from("./images/");

                for (index, url) in data.images.iter().enumerate() {
                    let update_download_progress = download_queue.clone();
                    let id = Uuid::new_v4();
                    let info = DownloadInfo::new(data.title.clone());
                    let file_name = format!("{}-{}.{}", data.title, index, &url[url.len() - 3..]); 
                    download_queue.lock().unwrap().insert(id.clone(), info);
                    
                    let task = tokio::spawn(downloader(path.join(file_name), url.clone(), move |now_size, total_size| {
                        let mut write_update = update_download_progress.lock().unwrap();
                        let mut info = write_update[&id].clone();
                        info.progress = ((now_size as f64 / total_size as f64) * 100.0) as u64;
                        write_update.insert(id, info);
                    }, |_| {}));

                    queue.insert(id, task);
                }

                for (id, task) in queue {
                    task.await.unwrap().unwrap();

                    download_queue.lock().unwrap().remove(&id);
                }
            });
        }
    }
}

impl Compose for ArtworkState {
    fn init(&mut self) {}

    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, focus: bool, area: Rect) {
        let check = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Max(2)])
            .split(area);
        let focus_style = if !focus {
            Style::default().fg(Color::White)
        } else {
            Style::default()
        };

        let text = Paragraph::new(self.input.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(focus_style)
                .title("input artwork id (Enter download)"),
        );

        f.set_cursor(check[0].x + self.input.len() as u16 + 1, check[0].y + 1);
        f.render_widget(text, check[0]);
    }

    fn update(&mut self, event: &crossterm::event::Event, download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>) {
        if let Event::Key(code) = event {
            match code.code {
                KeyCode::Char(c) => {
                    if ('0'..'9').contains(&c) {
                        self.input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => {
                    self.download(download_queue);                
                }
                _ => {}
            }
        }
    }
}
