use crossterm::event::Event;
use std::sync::{Arc, Mutex, RwLock};
use std::{collections::HashMap, io::Stdout};
use tui::widgets::{List, ListItem, ListState};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, BorderType, Borders, Paragraph},
    Frame,
};
use uuid::Uuid;
use x_pixiv_lib::user::User;

use super::compose::Compose;
use super::util::download;
use crate::tui_util::data::DownloadInfo;
use crossterm::event::KeyCode;

pub struct UserDownloaderState {
    input: String,
    artworks: Arc<RwLock<Vec<usize>>>,
    artowrks_state: ListState,
}

impl UserDownloaderState {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            input: String::new(),
            artworks: Arc::new(RwLock::new(vec![])),
            artowrks_state: ListState::default(),
        })
    }

    fn get_user_all_artwork(&mut self) {
        let Ok(id) = self.input.parse::<usize>() else {
            return;
        };
        let user = User::new(id);
        let clone_user_artworks = self.artworks.clone();

        tokio::spawn(async move {
            let Ok(mut ids) = user.get_artworks().await else {
                return;
            };
            let mut write = clone_user_artworks.write().unwrap();

            write.clear();
            write.append(&mut ids);
        });
    }

    fn next(&mut self) {
        let i = match self.artowrks_state.selected() {
            Some(i) => {
                if i >= self.artworks.read().unwrap().len() - 1 {
                    None
                } else {
                    Some(i + 1)
                }
            }
            None => {
                if self.artworks.read().unwrap().len() == 0 {
                    None
                } else {
                    Some(0)
                }
            }
        };

        self.artowrks_state.select(i);
    }

    fn prev(&mut self) {
        let i = match self.artowrks_state.selected() {
            Some(i) => {
                if i == 0 {
                    None
                } else {
                    Some(i - 1)
                }
            }
            None => {
                if self.artworks.read().unwrap().len() == 0 {
                    None
                } else {
                    Some(self.artworks.read().unwrap().len() - 1)
                }
            }
        };

        self.artowrks_state.select(i);
    }
}

impl Compose for UserDownloaderState {
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
                .title("input user id (Enter confirm)"),
        );

        let list = List::new(
            self.artworks
                .read()
                .unwrap()
                .iter()
                .map(|item| ListItem::new(format!("https://www.pixiv.net/artworks/{}", item)))
                .collect::<Vec<_>>(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(focus_style)
                .title("(Enter download selected) (`a` download all)"),
        )
        .highlight_style(Style::default().bg(Color::Gray));

        if self.artowrks_state.selected().is_none() {
            f.set_cursor(check[0].x + self.input.len() as u16 + 1, check[0].y + 1);
        }
        // input
        f.render_widget(text, check[0]);

        f.render_stateful_widget(list, check[1], &mut self.artowrks_state);
    }

    fn update(&mut self, event: &Event, download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>) {
        if let Event::Key(code) = event {
            match code.code {
                KeyCode::Char(c) => {
                    if ('0'..'9').contains(&c) && self.artowrks_state.selected().is_none() {
                        self.input.push(c);
                    } else if c == 'a' {
                        let artworks = self.artworks.clone();
                        let len = self.artworks.read().unwrap().len().clone();
                        tokio::spawn(async move {
                            for i in 0..len {
                                let id = artworks.read().unwrap()[i];
                                download(id, download_queue.clone()).await;
                            }
                        });
                    }
                }
                KeyCode::Backspace => {
                    self.input.pop();
                }
                KeyCode::Enter => {
                    if let Some(i) = self.artowrks_state.selected() {
                        let id = self.artworks.read().unwrap()[i];
                        tokio::spawn(download(id, download_queue));
                    } else {
                        self.get_user_all_artwork();
                    }
                }
                KeyCode::Up => {
                    self.prev();
                }
                KeyCode::Down => {
                    self.next();
                }
                _ => {}
            }
        }
    }
}