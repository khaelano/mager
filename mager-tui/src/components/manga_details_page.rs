use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEventKind};
use dto::{Chapter, ChapterListEntry, Filter, Manga};
use ratatui::prelude::*;
use ratatui::widgets::{Block, Cell, Padding, Paragraph, Row, Table, TableState, Wrap};

use crate::actions::*;
use crate::tui::Event;

use super::Component;

enum Focus {
    ChapterList,
    MangaDesc,
    MangaDetail,
}

pub struct MangaDetailsPage {
    manga: Option<Manga>,
    action_tx: ActionTx,
    chapter_table: ChapterTableComponent,
    manga_details: MangaDetailsComponent,
    focus: Focus,
}

impl MangaDetailsPage {
    pub(crate) fn new(action_tx: ActionTx) -> Self {
        Self {
            manga: None,
            chapter_table: ChapterTableComponent::new(action_tx.clone()),
            manga_details: MangaDetailsComponent::new(action_tx.clone()),
            focus: Focus::ChapterList,
            action_tx,
        }
    }
}

impl Component for MangaDetailsPage {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        if let Event::Key(k) = event.clone() {
            match k.code {
                KeyCode::Char('b') => self.action_tx.send(Action::PrevPage)?,
                KeyCode::Char('s') => todo!(),
                _ => {}
            }
        }

        match self.focus {
            Focus::MangaDetail => todo!(),
            Focus::MangaDesc => self.manga_details.handle_events(event)?,
            Focus::ChapterList => self.chapter_table.handle_events(event)?,
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        if let Action::SetActiveManga(manga) = action.clone() {
            self.manga = Some(manga);
        }

        self.chapter_table.update(action.clone())?;
        self.manga_details.update(action.clone())?;

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let layout = Layout::horizontal(Constraint::from_percentages([30, 70]))
            .spacing(1)
            .split(area);

        self.manga_details.draw(frame, layout[0])?;
        self.chapter_table.draw(frame, layout[1])?;
        Ok(())
    }
}

struct MangaDetailsComponent {
    active_manga: Option<Manga>,
    action_tx: ActionTx,
}

impl MangaDetailsComponent {
    fn new(action_tx: ActionTx) -> Self {
        Self {
            action_tx,
            active_manga: None,
        }
    }

    fn draw_desc(&self, frame: &mut Frame, area: Rect) {
        let Some(m) = self.active_manga.as_ref() else {
            return;
        };

        let block = Block::bordered().padding(Padding::horizontal(1));

        let manga_desc = Paragraph::new(Text::from_iter([Line::from(m.description.clone())]))
            .wrap(Wrap { trim: true })
            .block(block.title(" Description ".bold().light_yellow()));

        frame.render_widget(manga_desc, area);
    }

    fn draw_info(&self, frame: &mut Frame, area: Rect) {
        let Some(m) = self.active_manga.as_ref() else {
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

impl Component for MangaDetailsComponent {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        let Event::Key(k_event) = event else {
            return Ok(());
        };

        let KeyEventKind::Press = k_event.kind else {
            return Ok(());
        };

        // todo: Implements manga description scrolling
        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        if let Action::SetActiveManga(m) = action.clone() {
            self.active_manga = Some(m);
        }

        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let details_layout =
            Layout::vertical([Constraint::Fill(1), Constraint::Min(10)]).split(area);

        self.draw_info(frame, details_layout[0]);
        self.draw_desc(frame, details_layout[1]);
        Ok(())
    }
}

struct ChapterTableComponent {
    active_manga: Option<Manga>,
    action_tx: ActionTx,
    state: TableState,
    chapters: Vec<ChapterListEntry>,
    curr_page: u32,
    max_page: u32,
}

impl ChapterTableComponent {
    fn new(action_tx: ActionTx) -> Self {
        Self {
            active_manga: None,
            action_tx,
            state: TableState::default(),
            chapters: Vec::new(),
            curr_page: 0,
            max_page: 0,
        }
    }
}

impl Component for ChapterTableComponent {
    fn handle_events(&mut self, event: Event) -> Result<()> {
        let Event::Key(k_event) = event else {
            return Ok(());
        };

        let KeyEventKind::Press = k_event.kind else {
            return Ok(());
        };

        match k_event.code {
            KeyCode::Up => self.state.select_previous(),
            KeyCode::Down => {
                self.state.select_next();

                let Some(i) = self.state.selected() else {
                    return Ok(());
                };

                if self.chapters.is_empty() {
                    return Ok(());
                }

                if i == self.chapters.len() - 1 {
                    let Some(m) = self.active_manga.as_ref() else {
                        return Ok(());
                    };

                    if self.curr_page < self.max_page {
                        let command = Command::FetchChapterList {
                            identifier: m.identifier.clone(),
                            page: self.curr_page + 1,
                            filter: Filter::default(),
                        };
                        self.action_tx.send(Action::RunCommand(command))?;
                    }
                }
            }
            KeyCode::Enter => {
                let Some(i) = self.state.selected() else {
                    return Ok(());
                };

                let Some(m) = self.active_manga.as_ref() else {
                    return Ok(());
                };

                let selected_chapter = self.chapters.get(i).unwrap();

                self.action_tx
                    .send(Action::DownloadChapter(selected_chapter.identifier.clone()))?;
            }
            _ => {}
        }

        Ok(())
    }

    fn update(&mut self, action: Action) -> Result<()> {
        match action {
            Action::DisplayChapterList(mut r) => {
                self.chapters.append(&mut r.data);
                self.curr_page += 1;
                self.max_page = r.total_page;
            }
            Action::SetActiveManga(m) => {
                self.active_manga = Some(m);

                self.action_tx
                    .send(Action::RunCommand(Command::FetchChapterList {
                        identifier: self.active_manga.as_ref().unwrap().identifier.clone(),
                        page: 1,
                        filter: Filter::default(),
                    }))?;
            }
            _ => {}
        }
        Ok(())
    }

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        // for me next morning: Implement draw. fuck it, we go back to initial design
        let mut block = Block::bordered().title(" Chapter List ".bold().light_yellow());

        // if let Focus::ChapterList = self.focus {
        //     block = block.not_dim();
        // };

        let rows: Vec<Row> = self
            .chapters
            .iter()
            .map(|c| {
                Row::from_iter([
                    Cell::from(Text::from(c.number.clone()).alignment(Alignment::Left)),
                    Cell::from(Text::from(c.title.clone()).alignment(Alignment::Left)),
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

        frame.render_stateful_widget(table, area, &mut self.state);
        Ok(())
    }
}
