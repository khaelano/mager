use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use dto::carriers::{Command, Request};
use dto::{Filter, Manga};
use ratatui::prelude::*;
use ratatui::style::Stylize;
use ratatui::widgets::{Block, List, ListItem, ListState};

use crate::actions::{Action, ActionTx, AsyncItem, Page};
use crate::components::Component;
use crate::tui::Event;

pub(crate) struct MangaListComponent {
    action_tx: ActionTx,
    items: Vec<Manga>,
    list_state: ListState,
    request: Option<Request>,
    dim: bool,
}

impl MangaListComponent {
    pub(crate) fn new(action_tx: ActionTx) -> Self {
        Self {
            action_tx,
            items: Vec::new(),
            list_state: ListState::default(),
            request: None,
            dim: true,
        }
    }

    pub(crate) fn set_dim(&mut self, dim: bool) {
        self.dim = dim;
    }

    pub(crate) fn clear(&mut self) {
        self.items.clear();
        self.list_state.select(None);
        self.request = None;
    }

    pub(crate) fn search_manga(&mut self, keyword: &str, filter: &Filter) -> Result<()> {
        self.clear();
        self.request = Some(Request {
            command: Command::Search {
                keyword: keyword.to_string(),
                page: 1,
                filter: filter.clone(),
            },
            version: String::from("0.0.0"),
        });

        self.action_tx
            .send(Action::SendRequest(self.request.clone().unwrap()))?;

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let KeyEventKind::Press = key_event.kind else {
            return Ok(());
        };

        match key_event.code {
            KeyCode::Enter => {
                if let Some(i) = self.list_state.selected() {
                    self.action_tx.send(Action::NextPage(Page::MangaDetails))?;
                    self.action_tx
                        .send(Action::SetActiveManga(self.items.get(i).unwrap().clone()))?;
                }
            }
            KeyCode::Up => {
                self.list_state.select_previous();
            }
            KeyCode::Down => {
                self.list_state.select_next();
                if self.list_state.selected().unwrap_or(0) == self.items.len() - 1 {
                    let Some(req) = self.request.as_ref() else {
                        return Ok(());
                    };

                    if let Some(next_req) = req.next_page() {
                        self.request = Some(next_req.clone());
                        self.action_tx.send(Action::SendRequest(next_req))?;
                    }
                }
            }
            _ => {}
        }

        Ok(())
    }
}

impl Component for MangaListComponent {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(k_event) => self.handle_key_event(k_event)?,
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Process(i) => {
                if let AsyncItem::Mangas(mut m) = i {
                    self.items.append(&mut m.data);
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let mut block = Block::bordered();

        let list_item: Vec<ListItem> = self
            .items
            .iter()
            .map(|m| {
                let pub_status = match m.status {
                    dto::PublicationStatus::Ongoing => "Ongoing".yellow(),
                    dto::PublicationStatus::Completed => "Completed".green(),
                    dto::PublicationStatus::Hiatus => "Hiatus".magenta(),
                    dto::PublicationStatus::Cancelled => "Cancelled".red(),
                    dto::PublicationStatus::Unknown => "Unknown".dim(),
                };

                Text::from_iter([Line::from(m.title.clone()), Line::from(pub_status)]).into()
            })
            .collect();

        if self.dim {
            block = block.dim();
        }

        let list = List::new(list_item)
            .direction(ratatui::widgets::ListDirection::TopToBottom)
            .block(block)
            .highlight_symbol(" ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .repeat_highlight_symbol(true)
            .highlight_style(Style::new().on_dark_gray());

        frame.render_stateful_widget(list, area, &mut self.list_state);
        Ok(())
    }
}
