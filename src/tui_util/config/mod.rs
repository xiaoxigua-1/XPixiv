mod output;
mod group;

use self::{output::OutputConfig, group::GroupConfig};

use super::data::ConfigData;

use crossterm::event::{Event, KeyCode};
use std::{
    io::Stdout,
    sync::{Arc, Mutex},
};
use tui::{
    backend::CrosstermBackend,
    layout::{Margin, Rect},
    style::{Color, Style},
    text::Span,
    widgets::{Block, BorderType, Borders, Clear},
    Frame,
};

pub struct Config {
    pub config_data: ConfigData,
    state: ConfigState,
    config_items: Arc<Mutex<Vec<Box<dyn ConfigItem>>>>,
}

impl Config {
    pub fn new(config_data: ConfigData) -> Self {
        let mut config_items: Vec<Box<dyn ConfigItem>> = vec![OutputConfig::new(), GroupConfig::new()];

        config_items.iter_mut().for_each(|item| {
            item.init(&config_data);
        });

        Self {
            config_data,
            state: ConfigState::new(),
            config_items: Arc::new(Mutex::new(config_items)),
        }
    }

    pub fn render(&self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        let size = f.size();
        let width = size.width / 2;
        let height = size.height / 2;
        let x = (size.width - width) / 2;
        let y = (size.height - height) / 2;
        let rect = Rect {
            width,
            height,
            x,
            y,
        };
        let content_rect = rect.inner(&Margin {
            vertical: 1,
            horizontal: 1,
        });

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled(
                "Config",
                Style::default().fg(Color::LightCyan),
            ));

        f.render_widget(Clear, content_rect);
        f.render_widget(block, rect);

        for (i, item) in self.config_items.lock().unwrap().iter().enumerate() {
            let item_rect = Rect {
                x: content_rect.x,
                y: content_rect.y + i as u16 * 3,
                width: content_rect.width,
                height: 3,
            };

            item.render(item_rect, f, self.state.selected() == i);
        }
    }

    pub fn update(&mut self, event: &Event) {
        let config_items = self.config_items.clone();

        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Up => self.prev(),
                KeyCode::Down => self.next(),
                _ => {}
            }
        }

        config_items.lock().unwrap()[self.state.selected()]
            .update(&mut self.config_data, event);
    }

    fn next(&mut self) {
        self.state.select(
            if self.state.selected() + 1 >= self.config_items.lock().unwrap().len() {
                0
            } else {
                self.state.selected() + 1
            },
        );
    }

    fn prev(&mut self) {
        self.state.select(if self.state.selected() == 0 {
            self.config_items.lock().unwrap().len() - 1
        } else {
            self.state.selected() - 1
        });
    }
}

impl Default for Config {
    fn default() -> Self {
        Config::new(ConfigData {
            output: "./images".to_string(),
            group_type: None,
        })
    }
}

trait ConfigItem {
    fn init(&mut self, config_data: &ConfigData);

    fn render(&self, area: Rect, f: &mut Frame<CrosstermBackend<Stdout>>, forcu: bool);

    fn update(&mut self, config: &mut ConfigData, event: &Event);
}

pub struct ConfigState {
    index: usize,
}

impl ConfigState {
    pub fn new() -> Self {
        Self { index: 0 }
    }

    pub fn select(&mut self, index: usize) {
        self.index = index;
    }

    pub fn selected(&self) -> usize {
        self.index
    }
}
