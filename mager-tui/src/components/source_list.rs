use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;
use ratatui::widgets::{Block, List, ListDirection, ListItem, ListState, Padding};

use crate::actions::{Action, ActionTx, AsyncItem, Page};
use crate::source::Source;
use crate::tui::Event;

use super::Component;

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

    fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let KeyEventKind::Press = key_event.kind else {
            return Ok(());
        };

        match key_event.code {
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
}

impl Component for SourceListComp {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        match event {
            Event::Key(k) => self.handle_key_events(k)?,
            _ => {}
        }
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::Process(i) => match i {
                AsyncItem::Sources(s) => {
                    self.source_list = s;
                }
                _ => {}
            },
            _ => {}
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
