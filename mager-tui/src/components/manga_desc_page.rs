use std::env;
use std::path::PathBuf;

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyEventKind};
use dto::carriers::{Command, Request};
use dto::{Chapter, ChapterList, Filter, Manga};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Cell, Padding, Paragraph, Row, Table, TableState, Wrap};

use crate::actions::{Action, ActionTx, AsyncItem};
use crate::tui::Event;

use super::Component;

enum Focus {
    ChapterList,
    MangaDesc,
    MangaDetail,
}

pub struct MangaDescPage {
    manga: Option<Manga>,
    chapter_list: Vec<Chapter>,
    table_state: TableState,
    ch_list_req: Option<Request>,
    action_tx: ActionTx,
    focus: Focus,
}

impl MangaDescPage {
    pub(crate) fn new(action_tx: ActionTx) -> Self {
        Self {
            manga: None,
            chapter_list: Vec::new(),
            table_state: TableState::default(),
            ch_list_req: None,
            action_tx,
            focus: Focus::ChapterList,
        }
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        let KeyEventKind::Press = key_event.kind else {
            return Ok(());
        };

        // Page key event handling
        match key_event.code.clone() {
            KeyCode::Char('b') => self.action_tx.send(Action::PrevPage)?,
            _ => {}
        }

        // Widgets key event handling
        match self.focus {
            Focus::ChapterList => self.handle_table_key_event(key_event)?,
            Focus::MangaDesc => todo!(),
            Focus::MangaDetail => todo!(),
        }

        Ok(())
    }

    fn handle_table_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
        match key_event.code {
            KeyCode::Up => {
                self.table_state.select_previous();
            }
            KeyCode::Down => {
                self.table_state.select_next();

                let Some(i) = self.table_state.selected() else {
                    return Ok(());
                };

                if i == self.chapter_list.len() - 1 {
                    let Some(r) = self.ch_list_req.as_ref() else {
                        return Ok(());
                    };

                    if let Some(next_r) = r.next_page() {
                        self.action_tx.send(Action::SendRequest(next_r.clone()))?;
                        self.ch_list_req = Some(next_r);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    pub(crate) fn fetch_chapters(&mut self, filter: &Filter) -> Result<()> {
        let Some(m) = &self.manga else {
            return Ok(());
        };

        self.ch_list_req = Some(Request {
            command: Command::Chapters {
                identifier: m.identifier.to_string(),
                page: 1,
                filter: filter.clone(),
            },
            version: String::from("0.0.0"),
        });

        self.action_tx
            .send(Action::SendRequest(self.ch_list_req.clone().unwrap()))?;

        Ok(())
    }

    fn draw_table(&mut self, frame: &mut Frame, area: Rect) {
        let mut block = Block::bordered()
            .title(" Chapter List ".bold().light_yellow())
            .dim();

        if let Focus::ChapterList = self.focus {
            block = block.not_dim();
        };

        let rows: Vec<Row> = self
            .chapter_list
            .iter()
            .map(|c| {
                Row::from_iter([
                    Cell::from(Text::from(format!("{}", c.number)).alignment(Alignment::Left)),
                    Cell::from(Text::from(format!("{}", c.title)).alignment(Alignment::Left)),
                    Cell::from(Text::from("00-00-0000").alignment(Alignment::Center)),
                ])
                .bottom_margin(1)
            })
            .collect();

        let table = Table::new(
            rows,
            vec![
                Constraint::Length(4),
                Constraint::Fill(1),
                Constraint::Length(17),
            ],
        )
        .header(
            Row::from_iter([
                Text::from("Num."),
                Text::from("Title"),
                Text::from("Release Date").alignment(Alignment::Center),
            ])
            .bold(),
        )
        .block(block)
        .column_spacing(3)
        .highlight_symbol("│ ")
        .highlight_spacing(ratatui::widgets::HighlightSpacing::Always)
        .highlight_style(Style::new().yellow().bold());

        frame.render_stateful_widget(table, area, &mut self.table_state);
    }

    fn draw_desc(&self, frame: &mut Frame, area: Rect) {
        let Some(m) = self.manga.as_ref() else {
            return;
        };
        let block = Block::bordered().padding(Padding::horizontal(1));

        let manga_desc = Paragraph::new(Text::from_iter([Line::from(m.description.clone())]))
            .wrap(Wrap { trim: true })
            .block(block.title(" Description ".bold().light_yellow()));

        frame.render_widget(manga_desc, area);
    }

    fn draw_details(&self, frame: &mut Frame, area: Rect) {
        let Some(m) = self.manga.as_ref() else {
            return;
        };

        let block = Block::bordered().padding(Padding::horizontal(1));

        let pub_status = match m.status {
            dto::PublicationStatus::Ongoing => "Ongoing".yellow(),
            dto::PublicationStatus::Completed => "Completed".green(),
            dto::PublicationStatus::Hiatus => "Hiatus".magenta(),
            dto::PublicationStatus::Cancelled => "Cancelled".red(),
            dto::PublicationStatus::Unknown => "Unknown".dim(),
        };

        let identifier = Line::from(m.identifier.clone());
        let title = Line::from_iter(["Title: ".bold(), m.title.clone().into()]);
        let status = Line::from_iter(["Status: ".bold(), pub_status]);
        let authors = Line::from_iter([
            "Authors: ".bold(),
            m.authors
                .iter()
                .map(|a| format!("{} ({})", a.name, a.details))
                .collect::<Vec<String>>()
                .join(", ")
                .into(),
        ]);
        let original_lang =
            Line::from_iter(["Language: ".bold(), m.original_language.clone().into()]);

        let manga_info = Paragraph::new(Text::from_iter([
            identifier,
            title,
            status,
            authors,
            original_lang,
        ]))
        .wrap(Wrap { trim: false })
        .block(block.clone().title(" Manga Info ".bold().light_yellow()));

        frame.render_widget(manga_info, area);
    }
}

impl Component for MangaDescPage {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        match event.clone() {
            Event::Key(k_event) => self.handle_key_event(k_event)?,
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action.clone() {
            Action::SetActiveManga(manga) => {
                self.manga = Some(manga);
                let filter = Filter {
                    language: String::from("en"),
                    sort: dto::Order::Descending,
                };

                self.fetch_chapters(&filter)?;
            }
            Action::Process(AsyncItem::Chapters(mut cl)) => {
                self.chapter_list.append(&mut cl.data);
            }
            // Action::Process(crate::actions::AsyncItem::Pages(u)) => {
            //     let home = env::var("HOME").unwrap();
            //     let base_folder = format!("{home}/Downloads/mager/{}", "mangadex");

            //     let mut counter = 1;
            //     // for url in u {
            //     //     let mut path = PathBuf::from(base_folder.clone());
            //     //     path.push(format!(
            //     //         "{}/{} - {}/{counter}.png",
            //     //         chapter.number, chapter.title
            //     //     ));

            //     //     let handle = rt.spawn(download_resource(url.clone(), path.clone()));

            //     //     handles.push(handle);
            //     //     counter += 1;
            //     // }
            // }
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let main_layout = Layout::horizontal(Constraint::from_percentages([30, 70]))
            .spacing(1)
            .split(area);

        let manga_info_layout =
            Layout::vertical([Constraint::Fill(1), Constraint::Min(10)]).split(main_layout[0]);

        let chapter_list = main_layout[1];

        self.draw_details(frame, manga_info_layout[0]);
        self.draw_desc(frame, manga_info_layout[1]);
        self.draw_table(frame, chapter_list);
        Ok(())
    }
}
