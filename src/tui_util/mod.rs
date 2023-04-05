mod artwork;
mod compose;
mod config;
mod data;
mod rank;
mod user;
mod util;

use std::collections::HashMap;
use std::fs::File;
use std::io::{Read, Stdout};
use std::sync::{Arc, Mutex};

use crate::tui_util::compose::Compose;
use crossterm::event::{Event, KeyCode};
use data::DownloadInfo;
use rank::RankState;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, ListState},
    Frame,
};
use uuid::Uuid;

use self::artwork::ArtworkDownloaderState;
use self::config::Config;
use self::data::ConfigData;
use self::user::UserDownloaderState;

pub struct AppState<'a> {
    menu: Vec<ListItem<'a>>,
    menu_state: ListState,
    pub contents: Vec<Box<dyn Compose>>,
    pub focus: bool,
    download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>,
    config: Config,
    pub config_open: bool,
}

impl<'a> AppState<'a> {
    pub fn new(menu: Vec<ListItem<'a>>) -> Self {
        let rank_downloader_state = RankState::new(vec![
            "daily", "weekly", "monthly", "rookie", "original", "daily_ai", "male", "female",
        ]);
        let artwork_state = ArtworkDownloaderState::new();
        let user_state = UserDownloaderState::new();
        let config_data = if let Ok(mut file) = File::open("./config.toml") {
            let mut content = String::new();
            file.read_to_string(&mut content).unwrap();
            toml::from_str::<ConfigData>(&content).unwrap()
        } else {
            ConfigData::default().save()
        };
        Self {
            menu,
            menu_state: ListState::default(),
            focus: true,
            contents: vec![rank_downloader_state, artwork_state, user_state],
            download_queue: Arc::new(Mutex::new(HashMap::new())),
            config: Config::new(config_data),
            config_open: false,
        }
    }

    pub fn init(&mut self) {
        self.menu_state.select(Some(0));
        self.contents.iter_mut().for_each(|content| {
            content.init();
        });
    }

    fn next(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i >= self.menu.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };

        self.menu_state.select(Some(i));
    }

    fn prev(&mut self) {
        let i = match self.menu_state.selected() {
            Some(i) => {
                if i == 0 {
                    self.menu.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };

        self.menu_state.select(Some(i));
    }

    pub fn update(&mut self, event: &Event) {
        if self.config_open {
            self.config.update(event);
        } else if self.focus {
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Down => self.next(),
                    KeyCode::Up => self.prev(),
                    _ => {}
                }
            }
        } else if let Some(content) = self.contents.get_mut(self.menu_state.selected().unwrap()) {
            content.update(
                event,
                self.download_queue.clone(),
                self.config.config_data.clone(),
            );
        }
    }

    pub fn ui(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(80)].as_ref())
            .split(f.size());
        let border_style = if self.focus {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        // Menu
        let list = List::new(self.menu.clone())
            .block(
                Block::default()
                    .title("Menu ⇦⇧⇩⇨ ")
                    .borders(Borders::ALL)
                    .border_style(border_style)
                    .border_type(BorderType::Rounded),
            )
            .style(Style::default().add_modifier(Modifier::BOLD))
            .highlight_style(
                Style::default()
                    .add_modifier(Modifier::BOLD)
                    .bg(Color::Gray),
            )
            .highlight_symbol("> ");
        f.render_stateful_widget(list, chunks[0], &mut self.menu_state);

        let index = self.menu_state.selected().unwrap();
        if let Some(content) = self.contents.get_mut(index) {
            content.render(f, self.focus, chunks[1]);
        }

        let size = f.size();
        for (index, progress) in self.download_queue.lock().unwrap().values().enumerate() {
            if size.width < 20 || size.height < ((index + 1) * 4) as u16 {
                break;
            } else {
                let x = size.width - 25;
                let y = size.height - ((index + 1) * 4) as u16;

                f.render_widget(
                    Gauge::default()
                        .block(
                            Block::default()
                                .borders(Borders::ALL)
                                .title(progress.title.clone()),
                        )
                        .percent(progress.progress as u16)
                        .gauge_style(
                            Style::default()
                                .fg(Color::White)
                                .bg(Color::Black)
                                .add_modifier(Modifier::ITALIC),
                        ),
                    Rect::new(x, y, 20, 3),
                );
            }
        }
        if self.config_open {
            self.config.render(f);
        }
    }
}
