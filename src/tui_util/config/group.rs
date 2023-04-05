use crossterm::event::KeyCode;
use tui::{
    backend::CrosstermBackend,
    layout::{
        Rect,
        Layout,
        Constraint,
        Direction,
        Margin, Alignment
    },
    style::{Color, Style},
    widgets::Paragraph,
    Frame
};
use crossterm::event::Event;
use std::io::Stdout;

use crate::tui_util::data::GroupType;
use crate::tui_util::data::ConfigData;

use super::ConfigItem;


pub struct GroupConfig {
    group_type: Option<GroupType>,
    groups: Vec<Option<GroupType>>,
    index: usize
}

impl GroupConfig {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            index: 0,
            group_type: None,
            groups: vec![None, Some(GroupType::Author), Some(GroupType::Artwork)]
        })
    }

    fn next(&mut self) {
        self.index = if self.index + 1 >= self.groups.len() {
            0
        } else {
            self.index + 1
        };

        self.group_type = self.groups[self.index].clone();
    }

    fn prev(&mut self) {
        self.index = if self.index == 0 {
            self.groups.len() - 1
        } else {
            self.index - 1
        };

        self.group_type = self.groups[self.index].clone();
    }
}

impl ConfigItem for GroupConfig {
    fn init(&mut self, config_data: &ConfigData) {
        self.group_type = config_data.group_type.clone(); 
    }

    fn render(&self, area: Rect, f: &mut Frame<CrosstermBackend<Stdout>>, forcu: bool) {
        let check = Layout::default()
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .direction(Direction::Horizontal)
            .split(area);
        let forcu_style = Style::default().fg(if forcu { Color::White } else { Color::DarkGray });
        let config_name = Paragraph::new("Folder Group").style(forcu_style);
        let group_str = if let Some(group) = &self.group_type { group.to_string() } else { "None".to_string() };
        let config_value = Paragraph::new(format!("◀ {} ▶", group_str)).alignment(Alignment::Center);

        f.render_widget(config_name, check[0].inner(&Margin { horizontal: 1, vertical: 1 }));
        f.render_widget(config_value, check[1].inner(&Margin { horizontal: 5, vertical: 1 }));
    }

    fn update(&mut self, config: &mut ConfigData, event: &Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Left => self.prev(),
                KeyCode::Right => self.next(),
                _ => {}
            }

            config.group_type = self.group_type.clone();
            config.save();
        }; 
    }
}
