use serde::{Deserialize, Serialize};

pub mod carriers {
    // GMTP: Gajelas (or Goblok) Manga Transfer Protocol
    // Version 0.0.0
    //
    // A dumb, whacky, freaky, skibidy attempt to make my own transfer protocol for transferring
    // manga data between my applications. (i know, it's dumb)
    // Please don't use it anywhere.
    //
    // Request:
    // `
    // GMTP-Version CRLF
    // COMMAND [params],[],... CRLF
    // CRLF CRLF
    // `
    //
    // Response:
    // `
    // STATUS Reason-Phrase GMTP-Version CRLF
    // Source-Name CRLF
    // transferred-data
    // CRLF CRLF
    // `
    //
    // Available COMMAND:
    // - SEARCH -> This command will ask server to search a manga (params: ["Manga Keyword"],[Filter])
    // - CHAPTERS -> This command will ask server to fetch a manga's chapter list (params: ["Manga's URL"],[Filter])
    // - PAGES -> This command will ask server to fetch a chapter's URL for its pages (params: ["Chapter's URL"])
    //
    // STATUS code meaning:
    // - 200 -> This means "Ok"
    // - 400 -> This means "The format is wrong, WTF is wrong with you!?"
    // - 404 -> This means "No data"
    // - 444 -> This means "Network problem"

    use super::*;

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Request {
        pub command: Command,
        pub version: String,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub struct Response<T> {
        pub status: Status,
        pub reason: String,
        pub source_name: String,
        pub content: Option<T>,
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    #[serde(tag = "command", content = "args")]
    pub enum Command {
        Ping,
        Search {
            keyword: String,
            page: u32,
            filter: Filter,
        },
        FetchChapterList {
            identifier: String,
            page: u32,
            filter: Filter,
        },
        FetchManga {
            manga_identifier: String,
        },
        FetchChapter {
            chapter_identifier: String,
        },
    }

    #[derive(Serialize, Deserialize, Debug, Clone)]
    pub enum Status {
        Ok,
        Error,
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Filter {
    pub language: String,
    pub sort: Order,
}

impl Default for Filter {
    fn default() -> Self {
        Self {
            language: String::from("en"),
            sort: Order::Descending,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Order {
    Ascending,
    Descending,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum PublicationStatus {
    Ongoing,
    Completed,
    Hiatus,
    Cancelled,
    Unknown,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Author {
    pub name: String,
    pub details: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct MangaList {
    pub page: u32,
    pub total_page: u32,
    pub data: Vec<MangaListEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MangaListEntry {
    pub identifier: String,
    pub title: String,
    pub status: PublicationStatus,
    // pub cover_art_url: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Manga {
    // Identifier could mean URL or hash code for each manga.
    // Either way it doesn't matter because it doesn't affect the
    // client in any way. But i suggest to use hash code if available
    pub identifier: String,
    pub title: String,
    pub authors: Vec<Author>,
    pub original_language: String,
    pub language: String,
    pub description: String,
    pub status: PublicationStatus,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct ChapterList {
    pub page: u32,
    pub total_page: u32,
    pub data: Vec<ChapterListEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ChapterListEntry {
    pub identifier: String,
    pub title: String,
    pub number: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Chapter {
    // Identifier could mean URL or hash code for each manga.
    // Either way it doesn't matter because it doesn't affect the
    // client in any way. But i suggest to use hash code if available
    pub identifier: String,
    pub manga_identifier: String,
    pub title: String,
    pub number: String,
    pub language: String,
    pub page_urls: Vec<String>,
}
