use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use pixiv::rank::rank_list::Content;
use tui::{
    backend::Backend,
    layout::Rect,
    style::{Color, Modifier, Style},
    text::Spans,
    widgets::{Block, BorderType, Borders, List, ListItem, ListState, Tabs},
    Frame,
};

pub struct AppState<'a> {
    menu: Vec<ListItem<'a>>,
    menu_state: ListState,
    pub focus: bool,
}

pub struct RankState<'a> {
    tabs_index: usize,
    rank_index: usize,
    rank_list: Vec<Content>,
    tabs: Vec<&'a str>,
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
            rank_index: 0,
            rank_list: vec![],
            tabs,
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
                .add_modifier(Modifier::ITALIC)
                .bg(Color::Gray),
        )
        .highlight_symbol("> ");
    f.render_stateful_widget(list, area, &mut state.menu_state);
}

pub fn rank_downloader_ui<B: Backend>(
    f: &mut Frame<B>,
    app_state: &mut AppState,
    state: &mut RankState,
    area: Rect,
) {
    let border_style = Style::default();
    let border_style = if !app_state.focus {
        border_style.fg(Color::White)
    } else {
        border_style
    };
    let tabs = Tabs::new(
        state
            .tabs
            .iter()
            .map(|tab| Spans::from(tab.clone()))
            .collect(),
    )
    .select(state.tabs_index)
    .style(Style::default())
    .block(
        Block::default()
            .title(format!("{} rank list", state.tabs[state.tabs_index]))
            .borders(Borders::ALL)
            .border_style(border_style)
            .border_type(BorderType::Rounded),
    )
    .highlight_style(
        Style::default()
            .bg(Color::Gray)
            .add_modifier(Modifier::BOLD),
    );
    f.render_widget(tabs, area);
}

pub fn rank_downloader_update(state: &mut RankState, key_event: KeyEvent) {
    match key_event.code {
        KeyCode::Tab => state.tabs_next(),
        KeyCode::BackTab => state.tabs_prev(),
        _ => {}
    }
}

pub fn update(state: &mut AppState, keycode: KeyCode) {
    match keycode {
        KeyCode::Down => state.next(),
        KeyCode::Up => state.prev(),
        _ => {}
    }
}
