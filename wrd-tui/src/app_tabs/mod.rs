use color_eyre::eyre::Result;
use ratatui::crossterm::event::Event;
use ratatui::widgets::Widget;

mod match_words;
mod not_wordle;

pub use match_words::MatchWords;
pub use not_wordle::NotWordle;

pub trait AppTab: AppTabIo + Widget {}

pub trait AppTabIo {
	fn label(&self) -> &'static str;
	fn set_active(&mut self, is_active: bool);
	fn handle_event(&mut self, event: &Event) -> Result<()>;
	fn get_cursor_position(&self) -> Option<(u16, u16)>;
}

impl std::fmt::Debug for dyn AppTabIo {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.label())
	}
}
