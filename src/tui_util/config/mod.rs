mod output;

use self::output::OutputConfig;

use super::data::ConfigData;

use std::{io::Stdout, sync::{Arc, Mutex}};
use crossterm::event::Event;
use tui::{
    Frame,
    backend::CrosstermBackend,
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

    fn render(&self, f: &mut Frame<CrosstermBackend<Stdout>>) {

    }

    fn update(&mut self, event: Event) {
        let config_items = self.config_items.clone();

        config_items.lock().unwrap()[self.state.selected()].update(&mut self.config_data, event);
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
