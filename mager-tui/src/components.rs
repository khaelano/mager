pub mod manga_details_page;
pub mod manga_list_page;
pub mod search_bar;
pub mod source_list_page;

use color_eyre::eyre::Result;
use ratatui::layout::Rect;
use ratatui::Frame;

use crate::actions::Action;
use crate::tui::Event;

pub trait Component {
    fn handle_events(&mut self, event: Event) -> Result<()>;

    fn update(&mut self, action: Action) -> Result<()>;

    fn draw(&mut self, frame: &mut Frame, area: Rect) -> Result<()>;
}
