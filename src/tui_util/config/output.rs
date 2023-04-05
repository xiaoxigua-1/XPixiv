use crossterm::event::{Event, KeyCode};

use crate::tui_util::data::ConfigData;

use super::ConfigItem;

pub struct OutputConfig {
    input: String,
    edit: bool
}

impl OutputConfig {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            input: String::new(),
            edit: false
        })
    }
}

impl ConfigItem for OutputConfig {
    fn render(&self) {
        
    }

    fn update(&mut self, config: &mut ConfigData, event: Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(c) => {
                    self.input.push(c);
                },
                KeyCode::Backspace => {
                    self.input.pop();
                },
                KeyCode::Enter => {
                    if self.edit {
                        config.output = self.input.clone();
                        self.edit = false;
                    } else {
                        self.edit = true;
                    }
                }
                _ => {}
            }
        } 
    }
}
