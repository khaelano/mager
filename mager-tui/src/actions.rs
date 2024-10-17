use dto::carriers::Response;
use dto::{Chapter, ChapterList, Filter, Manga, MangaList};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::source::Source;

pub type ActionTx = UnboundedSender<Action>;
pub type ActionRx = UnboundedReceiver<Action>;

#[derive(Clone)]
pub enum Action {
    Tick,
    Render,
    NextPage(Page),
    PrevPage,
    Quit,
    FetchSources,
    RunCommand(Command),
    DownloadChapter(String),
    SetActiveSource(Source),
    SetActiveManga(Manga),
    SetActiveChapter(Chapter),
    DisplayMangaList(MangaList),
    DisplayChapterList(ChapterList),
    DisplaySourceList(Vec<Source>),
    InvokeError(String),
}

#[derive(Clone)]
pub enum Command {
    SearchManga {
        keyword: String,
        page: u32,
        filter: Filter,
    },
    FetchChapterList {
        identifier: String,
        page: u32,
        filter: Filter,
    },
    FetchMangaDetail {
        identifier: String,
    },
    FetchChapterDetail {
        identifier: String,
    },
}

#[derive(Clone)]
pub enum Page {
    Sources,
    Mangas,
    MangaDetails,
}

#[derive(Clone)]
pub enum AsyncItem {
    Ok,
    MangaList(Response<MangaList>),
    ChapterList(Response<ChapterList>),
    Manga(Response<Manga>),
    Chapter(Response<Chapter>),
    Sources(Vec<Source>),
}
