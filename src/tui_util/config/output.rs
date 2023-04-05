use crossterm::event::{Event, KeyCode};
use std::io::Stdout;
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph},
    Frame,
};

use crate::tui_util::data::ConfigData;

use super::ConfigItem;

pub struct OutputConfig {
    input: String,
    edit: bool,
}

impl OutputConfig {
    pub fn new() -> Box<Self> {
        Box::new(Self {
            input: String::new(),
            edit: false,
        })
    }
}

impl ConfigItem for OutputConfig {
    fn init(&mut self, config_data: &ConfigData) {
        self.input = config_data.output.clone();
    }

    fn render(&self, area: Rect, f: &mut Frame<CrosstermBackend<Stdout>>, forcu: bool) {
        let check = Layout::default()
            .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
            .direction(Direction::Horizontal)
            .split(area);
        let forcu_style = Style::default().fg(if forcu { Color::White } else { Color::DarkGray });
        let config_name = Paragraph::new("Output Path").style(forcu_style);
        let input = Paragraph::new(self.input.clone()).block(
            Block::default()
                .borders(Borders::ALL)
                .style(forcu_style)
                .title(Spans::from(vec![
                    Span::styled("Enter ", Style::default().fg(Color::Red)),
                    Span::raw(if !self.edit { "Edit" } else { "Save" }),
                ])),
        );

        if self.edit {
            f.set_cursor(check[1].x + self.input.len() as u16 + 1, check[1].y + 1);
        }

        f.render_widget(
            config_name,
            check[0].inner(&Margin {
                horizontal: 1,
                vertical: 1,
            }),
        );
        f.render_widget(input, check[1]);
    }

    fn update(&mut self, config: &mut ConfigData, event: &Event) {
        if let Event::Key(key) = event {
            match key.code {
                KeyCode::Char(c) => {
                    if self.edit {
                        self.input.push(c);
                    }
                }
                KeyCode::Backspace => {
                    if self.edit {
                        self.input.pop();
                    }
                }
                KeyCode::Enter => {
                    if self.edit {
                        config.output = self.input.clone();
                        self.edit = false;
                        config.save();
                    } else {
                        self.edit = true;
                    }
                }
                _ => {}
            }
        }
    }
}
