use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListDirection, ListItem, ListState, Padding};

use crate::actions::{Action, ActionTx, AsyncItem, Page};
use crate::source::Source;
use crate::tui::Event;

use super::search_bar::SearchBarComponent;
use super::Component;

pub struct SourcesPage {
    search_bar: SearchBarComponent,
    source_list: SourceListComp,
    action_tx: ActionTx,
}

impl SourcesPage {
    pub(crate) fn new(action_tx: ActionTx) -> Self {
        action_tx.send(Action::FetchSources).unwrap();

        Self {
            search_bar: SearchBarComponent::new(),
            source_list: SourceListComp::new(action_tx.clone()),
            action_tx,
        }
    }
}

impl Component for SourcesPage {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        let Event::Key(k_event) = event.clone() else {
            return Ok(());
        };

        let KeyEventKind::Press = k_event.kind else {
            return Ok(());
        };

        match k_event.code {
            KeyCode::Char('q') => self.action_tx.send(Action::Quit)?,
            KeyCode::Enter => {
                // let keyword = self.search_bar.get_contents();
            }
            _ => {}
        }

        self.source_list.handle_events(event)?;

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        self.source_list.update(action)?;

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Length(3), Constraint::Fill(10)])
            .split(area);

        self.search_bar.draw(frame, layout[0])?;
        self.source_list.draw(frame, layout[1])?;

        Ok(())
    }
}

pub struct SourceListComp {
    action_tx: ActionTx,
    list_state: ListState,
    source_list: Vec<Source>,
}

impl SourceListComp {
    pub(crate) fn new(action_tx: ActionTx) -> Self {
        Self {
            action_tx,
            list_state: ListState::default(),
            source_list: Vec::new(),
        }
    }
}

impl Component for SourceListComp {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        let Event::Key(k_event) = event else {
            return Ok(());
        };

        let KeyEventKind::Press = k_event.kind else {
            return Ok(());
        };

        match k_event.code {
            KeyCode::Up => self.list_state.select_previous(),
            KeyCode::Down => self.list_state.select_next(),
            KeyCode::Enter => {
                if let Some(index) = self.list_state.selected() {
                    self.action_tx.send(Action::SetActiveSource(
                        self.source_list.get(index).cloned().unwrap(),
                    ))?;
                    self.action_tx.send(Action::NextPage(Page::Mangas))?;
                };
            }
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        if let Action::DisplaySourceList(s_list) = action {
            self.source_list = s_list
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let block = Block::bordered().padding(Padding::horizontal(0));

        let list_items: Vec<ListItem> = self
            .source_list
            .iter()
            .map(|s| {
                let is_local = match s.is_local {
                    true => "local".green(),
                    false => "repo".gray(),
                };

                Text::from_iter([Line::from(s.name.clone()), Line::from(is_local)]).into()
            })
            .collect();

        let list = List::new(list_items)
            .direction(ListDirection::TopToBottom)
            .block(block)
            .highlight_symbol(" ")
            .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
            .repeat_highlight_symbol(true)
            .highlight_style(Style::new().on_dark_gray());

        frame.render_stateful_widget(list, area, &mut self.list_state);

        Ok(())
    }
}
