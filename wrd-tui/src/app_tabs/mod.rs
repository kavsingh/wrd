use ratatui::widgets::Widget;

mod match_word;
mod not_wordle;

pub use match_word::MatchWord;
pub use not_wordle::NotWordle;

pub trait AppTab: AppTabIo + Widget {}

pub trait AppTabIo {
	fn label(&self) -> &'static str;
	fn set_active(&mut self, is_active: bool);
}

impl std::fmt::Debug for dyn AppTabIo {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.label())
	}
}
