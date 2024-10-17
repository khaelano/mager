use color_eyre::eyre::Result;
use dto::carriers::Status;
use tokio::sync::mpsc;
use tokio::task::JoinHandle;

use crate::actions::*;
use crate::components::manga_details_page::MangaDetailsPage;
use crate::components::manga_list_page::MangaListPage;
use crate::components::source_list_page::SourcesPage;
use crate::components::Component;
use crate::mager::*;
use crate::source::Source;
use crate::tui::{Event, Tui};
use crate::utils;

pub(crate) struct App {
    tui: Tui,
    active_source: Option<Source>,
    source_port: u16,
    should_exit: bool,
    action_tx: ActionTx,
    action_rx: ActionRx,
    active_pages: Vec<Box<dyn Component>>,
}

impl App {
    pub(crate) fn new() -> Result<Self> {
        let (action_tx, action_rx) = mpsc::unbounded_channel();

        Ok(Self {
            tui: Tui::new()?,
            active_source: None,
            source_port: 7878,
            should_exit: false,
            active_pages: vec![Box::new(SourcesPage::new(action_tx.clone()))],
            action_tx,
            action_rx,
        })
    }

    async fn handle_commands(&mut self, command: Command) -> Result<()> {
        let source_port = self.source_port;
        let action_tx = self.action_tx.clone();
        let _: JoinHandle<()> = tokio::spawn(async move {
            let result: Action = match command {
                Command::SearchManga {
                    keyword,
                    page,
                    filter,
                } => match search_manga(source_port, &keyword, page, &filter).await {
                    Ok(response) => match response.status {
                        Status::Ok => Action::DisplayMangaList(response.content.unwrap()),
                        Status::Error => Action::InvokeError(response.reason),
                    },
                    Err(error) => Action::InvokeError(error.to_string()),
                },
                Command::FetchChapterList {
                    identifier,
                    page,
                    filter,
                } => match fetch_chapters(source_port, &identifier, page, &filter).await {
                    Ok(response) => match response.status {
                        Status::Ok => Action::DisplayChapterList(response.content.unwrap()),
                        Status::Error => Action::InvokeError(response.reason),
                    },
                    Err(error) => Action::InvokeError(error.to_string()),
                },
                Command::FetchMangaDetail { identifier } => {
                    match fetch_manga(source_port, &identifier).await {
                        Ok(response) => match response.status {
                            Status::Ok => Action::SetActiveManga(response.content.unwrap()),
                            Status::Error => Action::InvokeError(response.reason),
                        },
                        Err(error) => Action::InvokeError(error.to_string()),
                    }
                }
                Command::FetchChapterDetail { identifier } => {
                    match fetch_chapter(source_port, &identifier).await {
                        Ok(response) => match response.status {
                            Status::Ok => Action::SetActiveChapter(response.content.unwrap()),
                            Status::Error => Action::InvokeError(response.reason),
                        },
                        Err(error) => Action::InvokeError(error.to_string()),
                    }
                }
            };

            action_tx.send(result);
        });

        Ok(())
    }

    pub(crate) async fn run(&mut self) -> Result<()> {
        self.tui.enter()?;

        loop {
            self.handle_events().await?;
            self.handle_actions().await?;
            if self.should_exit {
                break;
            }
        }

        self.tui.exit()?;
        if let Some(c) = self.active_source.as_mut() {
            c.deactivate_source();
        }
        Ok(())
    }

    async fn handle_events(&mut self) -> Result<()> {
        let Some(event) = self.tui.next_event().await else {
            return Ok(());
        };

        let action_sx = self.action_tx.clone();
        match event.clone() {
            Event::Tick => action_sx.send(Action::Tick)?,
            Event::Render => action_sx.send(Action::Render)?,
            _ => {}
        }

        if let Some(p) = self.active_pages.last_mut() {
            p.handle_events(event)?;
        }

        Ok(())
    }

    async fn handle_actions(&mut self) -> Result<()> {
        while let Ok(action) = self.action_rx.try_recv() {
            match action.clone() {
                Action::Render => self.render()?,
                Action::Quit => self.should_exit = true,
                Action::NextPage(p) => {
                    let page: Box<dyn Component> = match p {
                        Page::Sources => Box::new(SourcesPage::new(self.action_tx.clone())),
                        Page::Mangas => Box::new(MangaListPage::new(self.action_tx.clone())),
                        Page::MangaDetails => {
                            Box::new(MangaDetailsPage::new(self.action_tx.clone()))
                        }
                    };

                    self.active_pages.push(page);
                }
                Action::PrevPage => {
                    let _ = self.active_pages.pop();
                }
                Action::SetActiveSource(mut s) => {
                    s.activate_source(self.source_port).await?;
                    self.active_source = Some(s);
                }
                Action::FetchSources => {
                    let action_tx = self.action_tx.clone();

                    tokio::spawn(async move {
                        fetch_sources(action_tx).await.unwrap();
                    });
                }
                Action::RunCommand(c) => {
                    self.handle_commands(c).await?;
                }
                Action::DownloadChapter(ch_id) => {
                    let ch_id = ch_id.clone();
                    let sc_port = self.source_port;
                    tokio::spawn(async move {
                        let ch_id_2 = ch_id;
                        let sc_port_2 = sc_port;
                        let _ = download_chapter(sc_port_2, &ch_id_2).await;
                    });
                }
                _ => {}
            }

            if let Some(p) = self.active_pages.last_mut() {
                p.update(action)?;
            }
        }

        Ok(())
    }

    fn render(&mut self) -> Result<()> {
        self.tui.terminal.draw(|f| {
            let Some(p) = self.active_pages.last_mut() else {
                return;
            };

            if p.draw(f, f.area()).is_err() {
                eprintln!("Error drawing");
            }
        })?;
        Ok(())
    }
}
