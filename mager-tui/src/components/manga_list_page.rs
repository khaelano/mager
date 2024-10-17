use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use dto::{Filter, Manga, MangaListEntry};
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListItem, ListState};

use crate::actions::*;
use crate::tui::Event;

use super::search_bar::SearchBarComponent;
use super::Component;

pub enum Focus {
    MangaList,
    SearchBar,
}

pub struct MangaListPage {
    search_bar: SearchBarComponent,
    manga_list: MangaListComponent,
    action_tx: ActionTx,
    focus: Focus,
}

impl MangaListPage {
    pub fn new(action_tx: ActionTx) -> Self {
        Self {
            search_bar: SearchBarComponent::new(),
            manga_list: MangaListComponent::new(action_tx.clone()),
            action_tx,
            focus: Focus::SearchBar,
        }
    }
}

impl Component for MangaListPage {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        match self.focus {
            Focus::SearchBar => self.search_bar.handle_events(event.clone())?,
            Focus::MangaList => self.manga_list.handle_events(event.clone())?,
        };

        let Event::Key(k_event) = event else {
            return Ok(());
        };

        let KeyEventKind::Press = k_event.kind else {
            return Ok(());
        };

        let key_code = k_event.code;
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

    fn update(&mut self, action: Action) -> Result<()> {
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

pub(crate) struct MangaListComponent {
    action_tx: ActionTx,
    items: Vec<MangaListEntry>,
    list_state: ListState,
    curr_page: u32,
    max_page: u32,
    dim: bool,
}

impl MangaListComponent {
    pub(crate) fn new(action_tx: ActionTx) -> Self {
        Self {
            action_tx,
            items: Vec::new(),
            list_state: ListState::default(),
            curr_page: 0,
            max_page: 0,
            dim: true,
        }
    }

    pub(crate) fn set_dim(&mut self, dim: bool) {
        self.dim = dim;
    }

    pub(crate) fn clear(&mut self) {
        self.list_state = ListState::default();
        self.items.clear();
        self.curr_page = 0;
        self.max_page = 0;
    }

    pub(crate) fn search_manga(&mut self, keyword: &str, filter: &Filter) -> Result<()> {
        let command = Command::SearchManga {
            keyword: keyword.to_string(),
            page: 1,
            filter: filter.clone(),
        };

        self.action_tx.send(Action::RunCommand(command))?;

        Ok(())
    }
}

impl Component for MangaListComponent {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        let Event::Key(k_event) = event else {
            return Ok(());
        };

        let KeyEventKind::Press = k_event.kind else {
            return Ok(());
        };

        match k_event.code {
            KeyCode::Enter => {
                if let Some(i) = self.list_state.selected() {
                    self.action_tx.send(Action::NextPage(Page::MangaDetails))?;
                    // self.action_tx.send(
                    //     Action::RunCommand(Command::Search { keyword: String::from("Blue arcive"), page: 1, filter: Filter::default() })
                    // )?;
                    self.action_tx
                        .send(Action::RunCommand(Command::FetchMangaDetail {
                            identifier: self.items.get(i).unwrap().identifier.clone(),
                        }))?;
                }
            }
            KeyCode::Up => {
                self.list_state.select_previous();
            }
            KeyCode::Down => {
                self.list_state.select_next();
                if self.list_state.selected().unwrap_or(0) == self.items.len() - 1 {}
            }
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        if let Action::DisplayMangaList(mut mg_list) = action {
            self.curr_page += 1;
            self.max_page = mg_list.total_page;

            self.items.append(&mut mg_list.data);
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
