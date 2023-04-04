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
use crate::tui_util::data::DownloadInfo;
use crossterm::event::KeyCode;

pub struct ArtworkState {
    download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>,
    input: String,
}

impl ArtworkState {
    pub fn new() -> Self {
        Self {
            download_queue: Arc::new(Mutex::new(HashMap::new())),
            input: String::new(),
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
                .border_style(focus_style),
        );

        f.set_cursor(check[0].x + self.input.len() as u16 + 1, check[0].y + 1);
        f.render_widget(text, check[0]);
    }

    fn update(&mut self, event: &crossterm::event::Event) {
        if let Event::Key(code) = event {
            match code.code {
                KeyCode::Char(c) => {
                    self.input.push(c);
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                _ => {}
            }
        }
    }
}
