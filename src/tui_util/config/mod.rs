mod output;

use self::output::OutputConfig;

use super::data::ConfigData;

use std::{io::Stdout, sync::{Arc, Mutex}};
use crossterm::event::{Event, KeyCode};
use tui::{
    Frame,
    backend::CrosstermBackend, layout::{Rect, Margin}, widgets::{Block, Borders, BorderType, Clear}, style::{Style, Color, Modifier}, text::Span,
};

pub struct Config {
    config_data: ConfigData,
    state: ConfigState,
    config_items: Arc<Mutex<Vec<Box<dyn ConfigItem>>>> 
}

impl Config {
    pub fn new(config_data: ConfigData) -> Self {
        Self {
            config_data,
            state: ConfigState::new(),
            config_items: Arc::new(Mutex::new(vec![
                OutputConfig::new() 
            ])) 
        }
    }

    pub fn render(&self, f: &mut Frame<CrosstermBackend<Stdout>>) {
        let size = f.size();
        let width = size.width / 2;
        let height = size.height / 2;
        let x = (size.width - width) / 2;
        let y = (size.height - height) / 2;
        let rect = Rect { width, height, x, y };
        let content_rect = rect.inner(&Margin { vertical: 1, horizontal: 1 }); 

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(Color::Cyan))
            .title(Span::styled("Config", Style::default().fg(Color::LightCyan)));
        
        f.render_widget(Clear, content_rect); 
        f.render_widget(block, rect);

        
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

        config_items.lock().unwrap()[self.state.selected()].update(&mut self.config_data, event.clone());
    }

    fn next(&mut self) {
        self.state.select(if self.state.selected() + 1 >= self.config_items.lock().unwrap().len() {
            0
        } else {
            self.state.selected() + 1
        });
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
        Config::new(ConfigData { output: "./images".to_string(), group_type: None }) 
    }
}

trait ConfigItem {
    fn render(&self);

    fn update(&mut self, config: &mut ConfigData, event: Event);
}

pub struct ConfigState {
    index: usize,
}

impl ConfigState {
    pub fn new() -> Self {
        Self {
            index: 0,
        }
    }

    pub fn select(&mut self, index: usize) {
        self.index = index;
    }

    pub fn selected(&self) -> usize {
        self.index
    }
}
