use crossterm::event::Event;
use std::io::Stdout;
use tui::{backend::CrosstermBackend, layout::Rect, Frame};

pub trait Compose {
    fn render(&mut self, f: &mut Frame<CrosstermBackend<Stdout>>, focus: bool, area: Rect);

    fn update(&mut self, event: &Event);

    fn init(&mut self);
}
