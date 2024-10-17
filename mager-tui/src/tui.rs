use std::io::{stdout, Stdout};
use std::time::Duration;

use color_eyre::eyre::Result;
use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent};
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use futures::{FutureExt, StreamExt};
use ratatui::{backend::CrosstermBackend, Terminal};
use tokio::sync::mpsc;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use tokio::task::JoinHandle;
use tokio::time::interval;
use tokio_util::{self, sync::CancellationToken};

#[derive(Clone)]
pub enum Event {
    Tick,
    Render,
    Init,
    Error,
    Key(KeyEvent),
}

pub struct Tui {
    pub terminal: Terminal<CrosstermBackend<Stdout>>,
    pub task: JoinHandle<()>,
    pub cancellation_token: CancellationToken,
    pub tick_rate: f64,
    pub frame_rate: f64,
    pub event_rx: UnboundedReceiver<Event>,
    pub event_sx: UnboundedSender<Event>,
}

impl Tui {
    pub(crate) fn new() -> Result<Tui> {
        let (event_sx, event_rx) = mpsc::unbounded_channel();
        Ok(Self {
            terminal: Terminal::new(CrosstermBackend::new(stdout()))?,
            task: tokio::spawn(async {}),
            cancellation_token: CancellationToken::new(),
            tick_rate: 4.0,
            frame_rate: 24.0,
            event_rx,
            event_sx,
        })
    }

    pub(crate) fn enter(&mut self) -> Result<()> {
        crossterm::terminal::enable_raw_mode()?;
        crossterm::execute!(stdout(), EnterAlternateScreen)?;

        Self::set_panic_hook();

        self.start();
        Ok(())
    }

    pub(crate) fn exit(&mut self) -> Result<()> {
        self.stop()?;
        if crossterm::terminal::is_raw_mode_enabled()? {
            Self::restore_term()?;
        }

        Ok(())
    }

    fn set_panic_hook() {
        let hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |panic_info| {
            let _ = Self::restore_term(); // ignore any errors as we are already failing
            hook(panic_info);
        }));
    }

    pub(crate) fn restore_term() -> Result<()> {
        crossterm::terminal::disable_raw_mode()?;
        crossterm::execute!(stdout(), LeaveAlternateScreen)?;
        Ok(())
    }

    pub(crate) fn start(&mut self) {
        self.cancel();
        self.cancellation_token = CancellationToken::new();

        let event_loop = Tui::event_loop(
            self.event_sx.clone(),
            self.cancellation_token.clone(),
            self.tick_rate,
            self.frame_rate,
        );

        self.task = tokio::spawn(event_loop);
    }

    pub(crate) fn stop(&mut self) -> Result<()> {
        self.cancel();
        let mut counter = 0;
        while !self.task.is_finished() {
            std::thread::sleep(Duration::from_millis(1));
            counter += 1;
            if counter > 50 {
                self.task.abort();
            }
            if counter > 100 {
                break;
            }
        }

        Ok(())
    }

    pub(crate) fn cancel(&mut self) {
        self.cancellation_token.cancel();
    }

    pub(crate) async fn event_loop(
        event_sx: UnboundedSender<Event>,
        cancellation_token: CancellationToken,
        tick_rate: f64,
        frame_rate: f64,
    ) {
        let mut tick_interval = interval(Duration::from_secs_f64(1.0 / tick_rate));
        let mut frame_interval = interval(Duration::from_secs_f64(1.0 / frame_rate));
        let mut event_stream = EventStream::new();

        event_sx
            .send(Event::Init)
            .expect("Failed to send init event");
        loop {
            let event = tokio::select! {
                _ = cancellation_token.cancelled() => {
                    break;
                },
                _ = tick_interval.tick() => Event::Tick,
                _ = frame_interval.tick() => Event::Render,
                crossterm_event = event_stream.next().fuse() => match crossterm_event {
                    Some(Ok(event)) => {
                        match event {
                            CrosstermEvent::Key(key) => Event::Key(key),
                            _ => {continue;}
                        }
                    },
                    Some(Err(_)) => Event::Error,
                    None => break,
                }
            };

            if event_sx.send(event).is_err() {
                break;
            }
        }
        cancellation_token.cancel();
    }

    pub(crate) async fn next_event(&mut self) -> Option<Event> {
        self.event_rx.recv().await
    }
}

impl Drop for Tui {
    fn drop(&mut self) {
        self.exit().unwrap();
    }
}
