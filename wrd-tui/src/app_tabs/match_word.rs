use color_eyre::eyre::Result;
use ratatui::crossterm::event::KeyEvent;

use super::AppTab;

#[derive(Default, Debug)]
pub struct MatchWord {}

const LABEL: &str = "Match";

impl MatchWord {
	fn key_event_handler(&mut self, event: KeyEvent) -> Result<()> {
		Ok(())
	}
}

impl AppTab for MatchWord {
	fn label(&self) -> &'static str {
		LABEL
	}

	fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
		self.key_event_handler(key_event)
	}
}
