use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use dto::Filter;
use ratatui::prelude::*;

use crate::actions::{Action, ActionTx};
use crate::tui::Event;

use super::manga_list::MangaListComponent;
use super::search_bar::SearchBarComponent;
use super::Component;

pub enum Focus {
    MangaList,
    SearchBar,
}

pub struct MangasPage {
    search_bar: SearchBarComponent,
    manga_list: MangaListComponent,
    action_tx: ActionTx,
    focus: Focus,
}

impl MangasPage {
    pub fn new(action_tx: ActionTx) -> Self {
        Self {
            search_bar: SearchBarComponent::new(),
            manga_list: MangaListComponent::new(action_tx.clone()),
            action_tx,
            focus: Focus::SearchBar,
        }
    }

    fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let KeyEventKind::Press = key_event.kind else {
            return Ok(());
        };

        let key_code = key_event.code;
        match self.focus {
            Focus::MangaList => match key_code {
                KeyCode::Char('s') => {
                    self.search_bar.clear_contents();
                    self.focus = Focus::SearchBar;
                }
                KeyCode::Char('b') => {
                    self.action_tx.send(Action::PrevPage)?;
                }
                _ => {}
            },
            Focus::SearchBar => match key_code {
                KeyCode::Esc => {
                    self.focus = Focus::MangaList;
                }
                KeyCode::Enter => {
                    let keyword = self.search_bar.get_contents();
                    let filter = Filter {
                        language: String::from("en"),
                        sort: dto::Order::Descending,
                    };

                    self.manga_list.clear();
                    self.manga_list.search_manga(&keyword, &filter)?;
                    self.focus = Focus::MangaList;
                }
                _ => {}
            },
        };

        Ok(())
    }
}

impl Component for MangasPage {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        match self.focus {
            Focus::SearchBar => self.search_bar.handle_events(event.clone())?,
            Focus::MangaList => self.manga_list.handle_events(event.clone())?,
        };

        match event {
            Event::Key(k) => self.handle_key_events(k)?,
            _ => {}
        };

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action.clone() {
            _ => {}
        }

        self.manga_list.update(action)?;
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(10)])
            .split(area);

        match self.focus {
            Focus::MangaList => {
                self.manga_list.set_dim(false);
                self.search_bar.set_dim(true);
            }
            Focus::SearchBar => {
                self.manga_list.set_dim(true);
                self.search_bar.set_dim(false);
            }
        }

        self.search_bar.draw(frame, layout[0])?;
        self.manga_list.draw(frame, layout[1])?;
        Ok(())
    }
}
