use color_eyre::eyre::{eyre, Result};
use tokio::sync::mpsc;

use crate::actions::{Action, ActionRx, ActionTx, Page};
use crate::components::manga_desc_page::MangaDescPage;
use crate::components::mangas_page::MangasPage;
use crate::components::sources_page::SourcesPage;
use crate::components::Component;
use crate::mager::{fetch_sources, send_request};
use crate::runner::AsyncRunner;
use crate::source::Source;
use crate::tui::{Event, Tui};

pub(crate) struct App {
    tui: Tui,
    async_runner: AsyncRunner,
    active_source: Option<Source>,
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
            async_runner: AsyncRunner::new(),
            active_source: None,
            should_exit: false,
            active_pages: vec![Box::new(SourcesPage::new(action_tx.clone()))],
            action_tx,
            action_rx,
        })
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

        self.async_runner.stop().await?;
        self.tui.exit()?;
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
            Event::Quit => action_sx.send(Action::Quit)?,
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
                        Page::Mangas => Box::new(MangasPage::new(self.action_tx.clone())),
                        Page::MangaDetails => Box::new(MangaDescPage::new(self.action_tx.clone())),
                    };

                    self.active_pages.push(page);
                }
                Action::PrevPage => {
                    let _ = self.active_pages.pop();
                }
                Action::SetActiveSource(s) => {
                    self.active_source = Some(s);
                }
                Action::SendRequest(r) => {
                    let source = self
                        .active_source
                        .as_mut()
                        .ok_or(eyre!("Source is empty"))?;

                    if !source.is_active().await {
                        source.activate_source(7878).await?;
                    }

                    let tx = self.action_tx.clone();
                    self.async_runner
                        .add_task(Box::new(async move {
                            send_request(tx, r).await.unwrap();
                        }))
                        .await?;
                }
                Action::FetchSources => {
                    let action_tx = self.action_tx.clone();

                    self.async_runner
                        .add_task(Box::new(async move {
                            fetch_sources(action_tx).await.unwrap();
                        }))
                        .await?;
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

            if let Err(_) = p.draw(f, f.area()) {
                eprintln!("Error drawing");
            }
        })?;
        Ok(())
    }
}
