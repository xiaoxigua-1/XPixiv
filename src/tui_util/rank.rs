use crate::cli::parse_agrs_type;
use crate::tui_util::compose::Compose;
use crate::tui_util::data::DownloadInfo;
use crossterm::event::{Event, KeyCode, MouseEventKind};
use std::{
    collections::HashMap,
    io::Stdout,
    path::PathBuf,
    sync::{Arc, Mutex, RwLock},
};
use tokio::task::JoinHandle;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, BorderType, Borders, Gauge, List, ListItem, ListState, Tabs},
    Frame,
};
use uuid::Uuid;
use x_pixiv_lib::data::Content;
use x_pixiv_lib::downloader::downloader;

pub struct RankState<'a> {
    tabs_index: usize,
    rank_list_state: ListState,
    rank_list: Arc<RwLock<Vec<Content>>>,
    tabs: Vec<&'a str>,
    queue: Vec<JoinHandle<()>>,
    download_queue: Arc<Mutex<HashMap<Uuid, DownloadInfo>>>,
}

impl DownloadInfo {
    fn new(title: String) -> Self {
        Self { title, progress: 0 }
    }
}

impl<'a> RankState<'a> {
    pub fn new(tabs: Vec<&'a str>) -> Self {
        Self {
            tabs_index: 0,
            rank_list_state: ListState::default(),
            rank_list: Arc::new(RwLock::new(vec![])),
            tabs,
            queue: vec![],
            download_queue: Arc::new(Mutex::new(HashMap::new())),
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
            loop {
                if let Some(content) = rank.next().await.unwrap() {
                    rank_list_clone.write().unwrap().push(content);
                } else {
                    break;
                }
            }
        });

        self.queue.push(task);
    }

    fn download(&mut self, index: usize) -> JoinHandle<()> {
        let download_id = self.rank_list.read().unwrap()[index].clone().illust_id;
        let clone_download_queue = self.download_queue.clone();

        tokio::spawn(async move {
            let images = x_pixiv_lib::artworks::get_artworks_data(download_id.clone())
                .await
                .unwrap();
            let mut queue = HashMap::new();
            for (index, url) in images.images.iter().enumerate() {
                let update_download_progress = clone_download_queue.clone();
                let title = format!("{}-{}", images.title, index + 1);
                let info = DownloadInfo::new(title.clone());
                let id = Uuid::new_v4();

                clone_download_queue
                    .lock()
                    .unwrap()
                    .insert(id.clone(), info);

                let task = tokio::spawn(downloader(
                    PathBuf::from(format!("./images/{}.{}", title, &url[url.len() - 3..])),
                    url.clone(),
                    move |now_size, total_size| {
                        let mut write_update = update_download_progress.lock().unwrap();
                        let mut info = write_update[&id].clone();
                        info.progress = ((now_size as f64 / total_size as f64) * 100.0) as u64;
                        write_update.insert(id, info);
                    },
                    |_| {},
                ));
                queue.insert(id, task);
            }

            for (id, task) in queue {
                task.await.unwrap().unwrap();
                clone_download_queue.lock().unwrap().remove(&id);
            }
        })
    }
}

impl<'a> Compose for RankState<'a> {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, focus: bool, area: Rect) {
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
                .title(format!("{} rank list(Tab)", self.tabs[self.tabs_index]))
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
    }

    fn update(&mut self, event: &Event) {
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
                    self.download(self.rank_list_state.selected().unwrap());
                }
                KeyCode::Down => self.list_next(),
                KeyCode::Up => self.list_prev(),
                KeyCode::Char('a') => {
                    let clone_len = self.rank_list.read().unwrap().len().clone();
                    for i in 0..clone_len {
                        self.download(i);
                    }
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
