use std::{
    io::Stdout,
    sync::{Arc, RwLock},
};

use crossterm::event::{KeyCode, Event, MouseEventKind};
use pixiv::rank::rank_list::Content;
use tokio::task::JoinHandle;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect, Direction},
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
        focus: bool,
        area: Rect,
    );

    fn update(&mut self, event: &Event);

    fn init(&mut self);
}

pub struct AppState<'a> {
    menu: Vec<ListItem<'a>>,
    menu_state: ListState,
    pub contents: Vec<Box<dyn Compose>>,
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
    pub fn new(menu: Vec<ListItem<'a>>, contents: Vec<Box<dyn Compose>>) -> Self {
        Self {
            menu,
            menu_state: ListState::default(),
            focus: true,
            contents
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
        if self.focus {
            if let Event::Key(key_event) = event {
                match key_event.code {
                    KeyCode::Down => self.next(),
                    KeyCode::Up => self.prev(),
                    _ => {}
                }
            }
        } else {
            if let Some(content) = self.contents.get_mut(self.menu_state.selected().unwrap()) {
                content.update(event);
            }
        }
    }

    pub fn ui(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        let chunks = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(15), Constraint::Percentage(80)].as_ref())
            .split(f.size());
        let border_style = Style::default();
        let border_style = if self.focus {
            border_style.fg(Color::White)
        } else {
            border_style
        };

        // Menu
        let list = List::new(self.menu.clone())
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
        f.render_stateful_widget(list, chunks[0], &mut self.menu_state);

        let index = self.menu_state.selected().unwrap(); 
        if let Some(content) = self.contents.get_mut(index) {
            content.render(f, self.focus, chunks[1]);
        }
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


impl<'a> Compose for RankState<'a> {
    fn render(
        &mut self,
        f: &mut Frame<CrosstermBackend<Stdout>>,
        focus: bool,
        area: Rect,
    ) {
        let border_style = Style::default();
        let border_style = if !focus {
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

    fn update(&mut self, event: &Event) {
        match event {
            Event::Key(key_event) => {
                match key_event.code {
                    KeyCode::Tab => {
                        self.get_data();
                        self.tabs_next();
                    }
                    KeyCode::BackTab => {
                        self.get_data();
                        self.tabs_prev();
                    },
                    KeyCode::Enter => todo!("Download"),
                    KeyCode::Down => self.list_next(),
                    KeyCode::Up => self.list_prev(),
                    _ => {}
                }
            }
            Event::Mouse(mouse_event) => {
                match mouse_event.kind {
                    MouseEventKind::ScrollUp => self.list_prev(),
                    MouseEventKind::ScrollDown => self.list_next(),
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn init(&mut self) {
        self.get_data();
    }
}

