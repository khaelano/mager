mod config;
mod mager;
mod manga;
mod source;
mod utils;

use std::io::stdout;

use mager::*;

use clap::{Parser, Subcommand};

use ratatui::prelude::CrosstermBackend;
use ratatui::Terminal;

use crossterm::terminal;
use crossterm::terminal::{EnterAlternateScreen, LeaveAlternateScreen};
use crossterm::ExecutableCommand;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Arguments {
    #[command(subcommand)]
    menu: Menu,
}

#[derive(Subcommand)]
enum Menu {
    Manga {
        #[command(subcommand)]
        operation: MangaOperation,

        #[arg(short, long, value_name = "SOURCE", default_value = "mangadex")]
        source: String,
    },

    Source,
    Config,
}

#[derive(Subcommand)]
enum MangaOperation {
    Search {
        keyword: String,

        #[arg(short, long, value_name = "PAGE", default_value = "1")]
        page: u32,
    },
    Download,
}

fn main() {
    // Parsing the arguments
    let args = Arguments::parse();

    // Boilerplate
    stdout().execute(EnterAlternateScreen).unwrap();
    terminal::enable_raw_mode().unwrap();
    let backend = CrosstermBackend::new(stdout());
    let mut terminal = Terminal::new(backend).unwrap();

    let mut mager = Mager::init(args);
    mager.run(&mut terminal);

    terminal::disable_raw_mode().unwrap();
    stdout().execute(LeaveAlternateScreen).unwrap();
}
