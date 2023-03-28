use std::{
    io::Stdout,
    sync::{Arc, RwLock},
};

use crossterm::event::{KeyCode, KeyEvent};
use pixiv::rank::rank_list::Content;
use tokio::task::JoinHandle;
use tui::{
    backend::{Backend, CrosstermBackend},
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Tabs},
    Frame,
};

use crate::cli::parse_agrs_type;

pub trait Compose {
    fn render(
        &mut self,
        f: &mut Frame<CrosstermBackend<Stdout>>,
        app_state: &mut AppState,
        area: Rect,
    );

    fn update(&mut self, key_event: KeyEvent);
}

pub struct AppState<'a> {
    menu: Vec<ListItem<'a>>,
    menu_state: ListState,
    pub focus: bool,
}

pub struct RankState<'a> {
    tabs_index: usize,
    rank_list_state: ListState,
    rank_list: Arc<RwLock<Vec<Content>>>,
    tabs: Vec<&'a str>,
    qu: Vec<JoinHandle<()>>,
}

impl<'a> AppState<'a> {
    pub fn new(menu: Vec<ListItem<'a>>) -> Self {
        Self {
            menu,
            menu_state: ListState::default(),
            focus: true,
        }
    }

    pub fn current(&self) -> usize {
        self.menu_state.selected().unwrap()
    }

    pub fn init(&mut self) {
        self.menu_state.select(Some(0));
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
}

impl<'a> RankState<'a> {
    pub fn new(tabs: Vec<&'a str>) -> Self {
        Self {
            tabs_index: 0,
            rank_list_state: ListState::default(),
            rank_list: Arc::new(RwLock::new(vec![])),
            tabs,
            qu: vec![],
        }
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

    pub fn get_data(&mut self) {
        self.rank_list_state.select(Some(0));
        for task in &self.qu {
            task.abort();
        }

        let rank_list_clone = self.rank_list.clone();
        let rank_type = parse_agrs_type(self.tabs[self.tabs_index]);

        let task = tokio::spawn(async move {
            rank_list_clone.write().unwrap().clear();
            let mut rank = pixiv::rank::Rank::new(rank_type, false, 1..500);
            loop {
                if let Some(content) = rank.next().await.unwrap() {
                    rank_list_clone.write().unwrap().push(content);
                } else {
                    break;
                }
            }
        });

        self.qu.push(task);
    }
}

pub fn ui<B: Backend>(f: &mut Frame<B>, state: &mut AppState, area: Rect) {
    let border_style = Style::default();
    let border_style = if state.focus {
        border_style.fg(Color::White)
    } else {
        border_style
    };

    // Menu
    let list = List::new(state.menu.clone())
        .block(
            Block::default()
                .title("Menu")
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
    f.render_stateful_widget(list, area, &mut state.menu_state);
}

impl<'a> Compose for RankState<'a> {
    fn render(
        &mut self,
        f: &mut Frame<CrosstermBackend<Stdout>>,
        app_state: &mut AppState,
        area: Rect,
    ) {
        let border_style = Style::default();
        let border_style = if !app_state.focus {
            border_style.fg(Color::White)
        } else {
            border_style
        };
        let check = Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Min(0)].as_ref())
            .split(area);
        let tabs = Tabs::new(
            self.tabs
                .iter()
                .map(|tab| Spans::from(tab.clone()))
                .collect(),
        )
        .select(self.tabs_index)
        .style(Style::default())
        .block(
            Block::default()
                .title(format!("{} rank list", self.tabs[self.tabs_index]))
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
                .map(|(index, content)| {
                    ListItem::new(format!(
                        "{: <3} |{} https://www.pixiv.net/artworks/{}",
                        index + 1,
                        content.title,
                        content.illust_id
                    ))
                })
                .collect::<Vec<ListItem>>(),
        )
        .block(
            Block::default()
                .borders(Borders::ALL)
                .border_style(border_style)
                .border_type(BorderType::Rounded),
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

    fn update(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Tab => {
                self.get_data();
                self.tabs_next();
            }
            KeyCode::BackTab => {
                self.get_data();
                self.tabs_prev();
            },
            KeyCode::Enter => todo!(),
            KeyCode::Down => self.list_next(),
            KeyCode::Up => self.list_prev(),
            _ => {}
        }
    }
}

pub fn update(state: &mut AppState, keycode: KeyCode) {
    match keycode {
        KeyCode::Down => state.next(),
        KeyCode::Up => state.prev(),
        _ => {}
    }
}
