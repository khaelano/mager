use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Padding, Paragraph};

use super::Component;
use crate::actions::Action;
use crate::tui::Event;

pub(crate) struct SearchBarComponent {
    cursor_pos: usize,
    query_string: String,
    dim: bool,
}

impl SearchBarComponent {
    pub(crate) fn new() -> Self {
        Self {
            cursor_pos: 0,
            query_string: String::from(" "),
            dim: false,
        }
    }

    pub(crate) fn get_contents(&self) -> String {
        self.query_string.trim().to_string()
    }

    pub(crate) fn clear_contents(&mut self) {
        self.query_string = String::from(" ");
        self.cursor_pos = 0;
    }

    pub(crate) fn set_dim(&mut self, dim: bool) {
        self.dim = dim;
    }

    fn delete_char(&mut self) {
        if self.query_string.is_empty() || self.cursor_pos == 0 {
            return;
        }

        let i = self
            .query_string
            .char_indices()
            .nth(self.cursor_pos - 1)
            .map(|(i, _)| i)
            .unwrap_or(self.query_string.len());

        self.query_string.remove(i);
        self.cursor_pos -= 1;
    }

    fn add_char(&mut self, ch: char) {
        let i = self
            .query_string
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.query_string.len());

        self.query_string.insert(i, ch);
        self.cursor_pos += 1;
    }

    fn move_cursor_next(&mut self) {
        if self.cursor_pos == self.query_string.chars().count() - 1 {
            return;
        }

        self.cursor_pos += 1;
    }

    fn move_cursor_prev(&mut self) {
        if self.cursor_pos == 0 {
            return;
        }

        self.cursor_pos -= 1;
    }
}

impl Component for SearchBarComponent {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        let Event::Key(key_event) = event else {
            return Ok(());
        };

        let KeyEventKind::Press = key_event.kind else {
            return Ok(());
        };

        match key_event.code {
            KeyCode::Backspace => {
                self.delete_char();
            }
            KeyCode::Left => {
                self.move_cursor_prev();
            }
            KeyCode::Right => {
                self.move_cursor_next();
            }
            KeyCode::Char(c) => {
                self.add_char(c);
            }
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        let _ = action;
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let cursor_idx = self
            .query_string
            .char_indices()
            .nth(self.cursor_pos)
            .map(|(i, _)| i)
            .unwrap_or(self.query_string.chars().count());

        let after_cursor_idx = self
            .query_string
            .char_indices()
            .nth(self.cursor_pos + 1)
            .map(|(i, _)| i)
            .unwrap_or(self.query_string.len());

        let before_cursor = &self.query_string[..cursor_idx];
        let after_cursor = &self.query_string[after_cursor_idx..];
        let cursor = &self.query_string[cursor_idx..after_cursor_idx];

        let line = Line::from_iter([
            before_cursor.into(),
            cursor.on_gray().black(),
            after_cursor.into(),
        ]);

        let text: Line = if &self.query_string == " " {
            Line::from("Type to search".dim())
        } else {
            line
        };

        let mut block = Block::bordered().padding(Padding::horizontal(1));
        if self.dim {
            block = block.dim();
        }

        let search_bar = Paragraph::new(text).block(block);

        frame.render_widget(search_bar, area);
        Ok(())
    }
}
