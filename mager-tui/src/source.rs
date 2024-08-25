use std::env;
use std::process::{Child, Command};
use std::sync::Arc;

use color_eyre::eyre::{eyre, Result};
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct Source {
    pub name: String,
    pub url: Option<String>,
    pub is_local: bool,
    pub process: Option<Arc<Mutex<Child>>>,
}

impl Source {
    pub(crate) fn get_process(&mut self) -> Option<Arc<Mutex<Child>>> {
        if let Some(arc) = self.process.clone() {
            return Some(arc.clone());
        }
        None
    }

    pub(crate) async fn is_active(&mut self) -> bool {
        if let Some(child) = self.process.as_mut() {
            if let Ok(None) = child.lock().await.try_wait() {
                return true;
            }
        }
        false
    }

    pub(crate) async fn activate_source(&mut self, port: u16) -> Result<()> {
        if self.is_active().await {
            return Err(eyre!("This source is already active!"));
        }

        let home = env::var("HOME")?;
        let path = if cfg!(debug_assertions) {
            format!("{home}/Projects/mager/target/debug/{}", self.name)
        } else {
            format!("{home}/.local/mager/sources/{}", self.name)
        };

        let process = Command::new(path).arg(&port.to_string()).spawn()?;

        self.process = Some(Arc::new(Mutex::new(process)));

        Ok(())
    }

    pub(crate) fn deactivate_source(&mut self) {
        let Some(child) = self.process.clone() else {
            return;
        };

        tokio::spawn(async move { child.lock().await.kill() });
    }
}

impl Drop for Source {
    fn drop(&mut self) {
        self.deactivate_source();
    }
}
