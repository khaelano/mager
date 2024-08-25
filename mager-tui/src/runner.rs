use std::pin::Pin;

use color_eyre::eyre::Result;
use futures::Future;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

pub type Task = Box<dyn Future<Output = ()> + Sync + Send + 'static>;

enum Command {
    Run(Task),
    Stop,
}

pub(crate) struct AsyncRunner {
    command_tx: Sender<Command>,
    runner: JoinHandle<()>,
}

impl AsyncRunner {
    pub(crate) fn new() -> Self {
        let (command_tx, mut command_rx) = tokio::sync::mpsc::channel::<Command>(3);

        let runner = tokio::spawn(async move {
            while let Some(command) = command_rx.recv().await {
                match command {
                    Command::Run(task) => {
                        Pin::from(task).await;
                    }
                    Command::Stop => break,
                }
            }
        });

        AsyncRunner { command_tx, runner }
    }

    pub(crate) async fn add_task(&mut self, task: Task) -> Result<()> {
        self.command_tx.send(Command::Run(task)).await?;
        Ok(())
    }

    pub(crate) async fn stop(&mut self) -> Result<()> {
        self.command_tx.send(Command::Stop).await?;
        Ok(())
    }
}
