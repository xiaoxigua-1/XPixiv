use crossterm::event::Event;
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
use super::util::download;
use crate::tui_util::data::DownloadInfo;
use crossterm::event::KeyCode;

pub struct ArtworkDownloaderState {
    input: String,
}

impl ArtworkDownloaderState {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            input: String::new(),
        })
    }
}

impl Compose for ArtworkDownloaderState {
    fn init(&mut self) {}

    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, focus: bool, area: Rect) {
        let check = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Max(2)])
            .split(area);
        let focus_style = if !focus {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
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

    fn update(
        &mut self,
        event: &crossterm::event::Event,
        download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>,
    ) {
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
                    let Ok(id) = self.input.parse::<usize>() else {
                        return;
                    };
                    tokio::spawn(download(id, download_queue));
                }
                _ => {}
            }
        }
    }
}
