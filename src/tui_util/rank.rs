use super::{util::download, data::ConfigData};
use crate::cli::parse_agrs_type;
use crate::tui_util::compose::Compose;
use crate::tui_util::data::DownloadInfo;
use crossterm::event::{Event, KeyCode, MouseEventKind};
use std::{
    collections::HashMap,
    io::Stdout,
    sync::{Arc, Mutex, RwLock},
};
use tokio::task::JoinHandle;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Tabs},
    Frame,
};
use uuid::Uuid;
use x_pixiv_lib::data::Content;

pub struct RankState<'a> {
    tabs_index: usize,
    rank_list_state: ListState,
    rank_list: Arc<RwLock<Vec<ArtworkInfo>>>,
    tabs: Vec<&'a str>,
    queue: Vec<JoinHandle<()>>,
}

struct ArtworkInfo {
    content: Content,
    error: bool,
    downloading: bool,
}

impl ArtworkInfo {
    fn new(content: Content) -> Self {
        Self {
            content,
            error: false,
            downloading: false,
        }
    }
}

impl<'a> RankState<'a> {
    pub fn new(tabs: Vec<&'a str>) -> Box<Self> {
        Box::new(Self {
            tabs_index: 0,
            rank_list_state: ListState::default(),
            rank_list: Arc::new(RwLock::new(vec![])),
            tabs,
            queue: vec![],
        })
    }

    fn tabs_next(&mut self) {
        self.tabs_index = if self.tabs_index >= self.tabs.len() - 1 {
            0
        } else {
            self.tabs_index + 1
        };
    }

    fn tabs_prev(&mut self) {
        self.tabs_index = if self.tabs_index == 0 {
            self.tabs.len() - 1
        } else {
            self.tabs_index - 1
        };
    }

    fn list_next(&mut self) {
        let i = match self.rank_list_state.selected() {
            Some(i) => Some(if i >= self.rank_list.read().unwrap().len() - 1 {
                0
            } else {
                i + 1
            }),
            None => {
                if self.rank_list.read().unwrap().len() == 0 {
                    None
                } else {
                    Some(0)
                }
            }
        };

        self.rank_list_state.select(i);
    }

    fn list_prev(&mut self) {
        let i = match self.rank_list_state.selected() {
            Some(i) => Some(if i == 0 {
                self.rank_list.read().unwrap().len() - 1
            } else {
                i - 1
            }),
            None => {
                if self.rank_list.read().unwrap().len() == 0 {
                    None
                } else {
                    Some(0)
                }
            }
        };

        self.rank_list_state.select(i);
    }

    fn get_data(&mut self) {
        self.rank_list_state.select(Some(0));
        for task in &self.queue {
            task.abort();
        }

        let rank_list_clone = self.rank_list.clone();
        let rank_type = parse_agrs_type(self.tabs[self.tabs_index]);

        let task = tokio::spawn(async move {
            rank_list_clone.write().unwrap().clear();
            let mut rank = x_pixiv_lib::rank::Rank::new(rank_type, false, 1..500);
            while let Some(content) = rank.next().await.unwrap() {
                rank_list_clone
                    .write()
                    .unwrap()
                    .push(ArtworkInfo::new(content));
            }
        });

        self.queue.push(task);
    }
}

impl<'a> Compose for RankState<'a> {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, focus: bool, area: Rect) {
        let border_style = if !focus {
            Style::default().fg(Color::White)
        } else {
            Style::default().fg(Color::DarkGray)
        };
        let red_style = Style::default().fg(Color::Red);
        let check = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);
        let tabs = Tabs::new(
            self.tabs
                .iter()
                .map(|tab| Spans::from(<&str>::clone(tab)))
                .collect(),
        )
        .select(self.tabs_index)
        .style(Style::default())
        .block(
            Block::default()
                .title(Spans::from(vec![
                    Span::raw(format!("{} rank list (", self.tabs[self.tabs_index])),
                    Span::styled("Tab", red_style),
                    Span::raw(")"),
                ]))
                .borders(Borders::ALL)
                .border_style(border_style)
                .border_type(BorderType::Rounded),
        )
        .highlight_style(
            Style::default()
                .bg(Color::Gray)
                .add_modifier(Modifier::BOLD),
        );

        let list = List::new(
            self.rank_list
                .read()
                .unwrap()
                .iter()
                .enumerate()
                .map(|(index, info)| {
                    ListItem::new(format!(
                        "{: <3} |{} https://www.pixiv.net/artworks/{}",
                        index + 1,
                        info.content.title,
                        info.content.illust_id
                    ))
                    .style(Style::default().bg(if info.error {
                        Color::Red
                    } else if info.downloading {
                        Color::LightGreen
                    } else {
                        Color::Reset
                    }))
                })
                .collect::<Vec<ListItem>>(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .border_type(BorderType::Rounded)
                .title(Spans::from(vec![
                    Span::styled("Enter", red_style),
                    Span::raw(" download selected | "),
                    Span::styled("A", red_style),
                    Span::raw("ll "),
                    Span::raw("download"),
                ])),
        )
        .style(Style::default().add_modifier(Modifier::BOLD))
        .highlight_style(
            Style::default()
                .add_modifier(Modifier::BOLD)
                .bg(Color::Gray),
        );

        f.render_stateful_widget(list, check[1], &mut self.rank_list_state);
        f.render_widget(tabs, check[0]);
    }

    fn update(&mut self, event: &Event, download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>, config: ConfigData) {  
        match event {
            Event::Key(key_event) => match key_event.code {
                KeyCode::Tab => {
                    self.get_data();
                    self.tabs_next();
                }
                KeyCode::BackTab => {
                    self.get_data();
                    self.tabs_prev();
                }
                KeyCode::Enter => {
                    let index = self.rank_list_state.selected().unwrap();
                    let rank_list = self.rank_list.clone();
                    let id = rank_list.read().unwrap()[index].content.illust_id;

                    rank_list.write().unwrap()[index].downloading = true;

                    tokio::spawn(async move {
                        if (download(id, download_queue, config).await).is_err() {
                            rank_list.write().unwrap()[index].error = true;
                        };
                        rank_list.write().unwrap()[index].downloading = false;
                    });
                }
                KeyCode::Down => self.list_next(),
                KeyCode::Up => self.list_prev(),
                KeyCode::Char('a') => {
                    let clone_len = self.rank_list.read().unwrap().len();
                    let rank_list = self.rank_list.clone();

                    tokio::spawn(async move {
                        for i in 0..clone_len {
                            rank_list.write().unwrap()[i].downloading = true;
                            let id = rank_list.read().unwrap()[i].content.illust_id;
                            if (download(id, download_queue.clone(), config.clone()).await).is_err() {
                                rank_list.write().unwrap()[i].error = true;
                            };
                            rank_list.write().unwrap()[i].downloading = false;
                        }
                    });
                }
                _ => {}
            },
            Event::Mouse(mouse_event) => match mouse_event.kind {
                MouseEventKind::ScrollUp => self.list_prev(),
                MouseEventKind::ScrollDown => self.list_next(),
                _ => {}
            },
            _ => {}
        }
    }

    fn init(&mut self) {
        self.get_data();
    }
}
