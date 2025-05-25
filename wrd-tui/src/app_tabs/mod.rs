use color_eyre::Result;
use ratatui::crossterm::event::KeyEvent;

mod match_word;
mod not_wordle;

pub use match_word::MatchWord;
pub use not_wordle::NotWordle;

pub trait AppTab {
	fn label(&self) -> &'static str;
	fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()>;
}

impl std::fmt::Debug for dyn AppTab {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.label())
	}
}
