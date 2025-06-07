use color_eyre::eyre::Result;
use ratatui::crossterm::event::Event;
use ratatui::widgets::StatefulWidgetRef;

mod match_words;
mod not_wordle;
mod settings;

pub use match_words::MatchWords;
pub use not_wordle::NotWordle;
pub use settings::Settings;

use crate::app::AppState;

#[derive(Debug, Default)]
pub enum Tab {
	#[default]
	MatchWords,
	NotWordle,
	Settings,
}

pub trait AppTab: AppTabIo + StatefulWidgetRef {}

pub trait AppTabIo {
	fn label(&self) -> &'static str;
	#[allow(dead_code)]
	fn tab(&self) -> Tab;
	fn set_active(&mut self, is_active: bool, state: &mut AppState);
	fn handle_event(&mut self, event: &Event, state: &mut AppState) -> Result<()>;
}

impl std::fmt::Debug for dyn AppTabIo {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		write!(f, "{}", self.label())
	}
}
