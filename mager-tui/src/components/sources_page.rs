use color_eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use ratatui::prelude::*;

use crate::actions::{Action, ActionTx};
use crate::tui::Event;

use super::search_bar::SearchBarComponent;
use super::source_list::SourceListComp;
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

    fn handle_key_events(&mut self, key_event: KeyEvent) -> Result<()> {
        let KeyEventKind::Press = key_event.kind else {
            return Ok(());
        };

        match key_event.code {
            KeyCode::Char('q') => self.action_tx.send(Action::Quit)?,
            KeyCode::Enter => {
                // let keyword = self.search_bar.get_contents();
            }
            _ => {}
        }

        Ok(())
    }
}

impl Component for SourcesPage {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        match event.clone() {
            Event::Key(k) => self.handle_key_events(k)?,
            _ => {}
        }

        self.source_list.handle_events(event)?;

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action.clone() {
            // Action::SwitchPage(p) => match p {
            //     Page::SourceSelect => self.action_tx.send(Action::FetchSources)?,
            //     _ => {}
            // },
            _ => {}
        }

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
