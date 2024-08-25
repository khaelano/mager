// use color_eyre::eyre::Result;
// use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
// use dto::Chapter;
// use ratatui::prelude::*;
// use ratatui::widgets::{Block, Cell, Row, Table, TableState};

// use crate::actions::{Action, ActionTx};
// use crate::tui::Event;

// use super::Component;

// pub struct ChapterListComponent<'a> {
//     items: &'a [Chapter],
//     table_state: TableState,
//     dim: bool,
//     action_tx: ActionTx,
// }

// impl<'a> ChapterListComponent<'a> {
//     pub(crate) fn new(action_tx: ActionTx, items: &'a [Chapter]) -> Self {
//         Self {
//             items,
//             table_state: TableState::default(),
//             dim: false,
//             action_tx,
//         }
//     }

//     pub(crate) fn set_dim(&mut self, dim: bool) {
//         self.dim = dim;
//     }

//     pub(crate) fn reset(&mut self) {
//         self.table_state.select(None);
//     }

//     pub(crate) fn selected(&self) -> Option<usize> {
//         self.table_state.selected()
//     }

// pub(crate) fn fetch_chapters(&mut self, identifier: &str, filter: &Filter) -> Result<()> {
//     self.clear();
//     self.request = Some(Request {
//         command: Command::Chapters {
//             identifier: identifier.to_string(),
//             page: 1,
//             filter: filter.clone(),
//         },
//         version: String::from("0.0.0"),
//     });

//     self.action_tx
//         .send(Action::SendRequest(self.request.clone().unwrap()))?;

//     Ok(())
// }

//     fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
//         let KeyEventKind::Press = key_event.kind else {
//             return Ok(());
//         };

//         match key_event.code {
//             KeyCode::Up => {
//                 self.table_state.select_previous();
//                 if let Some(i) = self.table_state.selected() {
//                     if i == 0 {
//                         self.child_event_tx.send(ChildEvent::FirstRow)?;
//                     }
//                 };
//             }
//             KeyCode::Down => {
//                 self.table_state.select_next();
//                 if let Some(i) = self.table_state.selected() {
//                     if i == self.items.len() - 1 {
//                         self.child_event_tx.send(ChildEvent::LastRow)?;
//                     }
//                 };
//             }
//             KeyCode::Enter => {
//                 if let Some(_) = self.table_state.selected() {
//                     self.child_event_tx.send(ChildEvent::Enter)?;
//                 };
//             }
//             _ => {}
//         }

//         Ok(())
//     }
// }

// impl Component for ChapterListComponent<'_> {
//     fn handle_events(&mut self, event: Event) -> Result<()> {
//         match event {
//             Event::Key(k_event) => self.handle_key_event(k_event)?,
//             _ => {}
//         }

//         Ok(())
//     }

//     fn update(&mut self, _action: Action) -> Result<()> {
//         Ok(())
//     }

//     fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
//         let mut block = Block::bordered().title(" Chapter List ".bold().light_yellow());

//         let rows: Vec<Row> = self
//             .items
//             .iter()
//             .map(|c| {
//                 Row::from_iter([
//                     Cell::from(Text::from(format!("{}", c.number)).alignment(Alignment::Left)),
//                     Cell::from(Text::from(format!("{}", c.title)).alignment(Alignment::Left)),
//                     Cell::from(Text::from("00-00-0000").alignment(Alignment::Center)),
//                 ])
//                 .bottom_margin(1)
//             })
//             .collect();

//         if self.dim {
//             block = block.dim();
//         }

//         let table = Table::new(
//             rows,
//             vec![
//                 Constraint::Length(4),
//                 Constraint::Fill(1),
//                 Constraint::Length(17),
//             ],
//         )
//         .header(
//             Row::from_iter([
//                 Text::from("Num."),
//                 Text::from("Title"),
//                 Text::from("Release Date").alignment(Alignment::Center),
//             ])
//             .bold(),
//         )
//         .block(block)
//         .column_spacing(3)
//         .highlight_symbol("│ ")
//         .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
//         .highlight_style(Style::new().yellow().bold());

//         frame.render_stateful_widget(table, area, &mut self.table_state);
//         Ok(())
//     }
// }
