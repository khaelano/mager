pub mod chapter_list;
pub mod manga_desc_page;
pub mod manga_list;
pub mod mangas_page;
pub mod search_bar;
pub mod source_list;
pub mod sources_page;

use color_eyre::eyre::Result;
use ratatui::layout::Rect;
use ratatui::Frame;
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};

use crate::actions::Action;
use crate::tui::Event;

pub trait Component {
    fn handle_events(&mut self, event: Event) -> Result<()>;

    fn update(&mut self, action: Action) -> Result<()>;

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}
