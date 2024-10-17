use tracing::Level;
use tracing_subscriber::FmtSubscriber;

mod actions;
mod app;
mod components;
mod mager;
mod source;
mod tui;
mod utils;

use color_eyre::Result;

use app::App;

#[tokio::main]
async fn main() -> Result<()> {
    // Set up tracing for logging
    let writer = tracing_appender::rolling::hourly("./logs", "mager.log");
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::TRACE)
        .with_writer(writer)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    // Set up eyre for easy error handling
    color_eyre::install()?;
    let mut app = App::new()?;
    app.run().await
}
