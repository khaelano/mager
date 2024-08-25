use app::App;

mod actions;
mod app;
mod components;
mod mager;
mod runner;
mod source;
mod tui;
mod utils;

use tokio;

#[tokio::main]
async fn main() {
    let mut app = App::new().unwrap();
    app.run().await.unwrap();
}
