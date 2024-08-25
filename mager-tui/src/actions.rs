use dto::carriers::Request;
use dto::{Chapter, ChapterList, ChapterPages, Manga, MangaList};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::source::Source;

pub type ActionTx = UnboundedSender<Action>;
pub type ActionRx = UnboundedReceiver<Action>;

#[derive(Clone)]
pub enum Page {
    Sources,
    Mangas,
    MangaDetails,
}

#[derive(Clone)]
pub enum Action {
    Tick,
    Render,
    NextPage(Page),
    PrevPage,
    Quit,
    FetchSources,
    Process(AsyncItem),
    SetActiveSource(Source),
    SetActiveManga(Manga),
    SetActiveChapter(Chapter),
    SendRequest(Request),
}

#[derive(Clone)]
pub enum AsyncItem {
    Mangas(MangaList),
    Chapters(ChapterList),
    Pages(ChapterPages),
    Sources(Vec<Source>),
}
