mod config;
mod manga;
mod source;
mod utils;

use clap::{Parser, Subcommand};

use manga::manga_menu_handler;

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
    let args = Arguments::parse();

    match args.menu {
        Menu::Manga { source, operation } => {
            manga_menu_handler(&source, &operation);
        }
        Menu::Source => {
            println!("Hello world! this is a source operation");
        }
        Menu::Config => {
            println!("Hello world! this is a source operation");
        }
    };
}
