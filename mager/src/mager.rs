use std::io::Stdout;
use std::time::Duration;

use crate::Arguments;

use crossterm::event::{self, KeyEventKind};
use crossterm::event::{Event, KeyCode};
use ratatui::backend::CrosstermBackend;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::block::Title;
use ratatui::widgets::*;
use ratatui::{Frame, Terminal};

pub(crate) enum Page {
    SourceMenu {
        source_on_local: Vec<String>,
        source_on_repo: Vec<String>,
    },
    Search,
    MangaInfo,
    ChapterDownload,
}

pub(crate) struct Mager {
    page: Page,
    exit: bool,
    args: Arguments,
}

pub(crate) type Tui = Terminal<CrosstermBackend<Stdout>>;

impl Mager {
    pub(crate) fn init(args: Arguments) -> Mager {
        Mager {
            page: Page::SourceMenu {
                source_on_local: Vec::new(),
                source_on_repo: Vec::new(),
            },
            exit: false,
            args,
        }
    }

    pub(crate) fn run(&mut self, terminal: &mut Tui) {
        while !self.exit {
            terminal.draw(|f| self.ui_handler(f)).unwrap();

            if event::poll(Duration::from_millis(16)).unwrap() {
                let event = event::read().unwrap();
                self.event_hander(event);
            }
        }
    }

    fn event_hander(&mut self, event: Event) {
        if let Event::Key(key) = event {
            let KeyEventKind::Press = key.kind else {
                return;
            };

            if let KeyCode::Char('q') = key.code {
                self.exit = true;
                return;
            }

            match &self.page {
                Page::SourceMenu {
                    source_on_local,
                    source_on_repo,
                } => match key.code {
                    KeyCode::Char('j') => todo!(),
                    KeyCode::Char('k') => todo!(),
                    KeyCode::Char('s') => todo!(),
                    KeyCode::Char('\n') => {}
                    _ => {}
                },
                Page::Search => match key.code {
                    _ => todo!(),
                },
                Page::MangaInfo => match key.code {
                    KeyCode::Char('j') => todo!(),
                    KeyCode::Char('k') => todo!(),
                    KeyCode::Char('s') => todo!(),
                    KeyCode::Char('\n') => {}
                    _ => {}
                },
                Page::ChapterDownload => todo!(),
            }
        }
    }

    fn ui_handler(&self, frame: &mut Frame) {
        match &self.page {
            Page::SourceMenu {
                source_on_local,
                source_on_repo,
            } => self.source_page_ui(frame),
            Page::Search => todo!(),
            Page::MangaInfo => todo!(),
            Page::ChapterDownload => todo!(),
        }
    }

    fn source_page_ui(&self, frame: &mut Frame) {
        let instruction = Title::from(Line::from(vec![
            "[q]".dim(),
            " Quit ".into(),
            "[s]".dim(),
            " Search ".into(),
            "[b]".dim(),
            " Back ".into(),
        ]))
        .alignment(ratatui::layout::Alignment::Left)
        .position(ratatui::widgets::block::Position::Bottom);

        let block = Block::new().title(instruction);

        let layout = Layout::default()
            .direction(ratatui::layout::Direction::Vertical)
            .constraints(vec![Constraint::Fill(1), Constraint::Length(1)])
            .split(frame.size());

        frame.render_widget(block, frame.size());
    }
}
