use color_eyre::eyre::Result;
use ratatui::crossterm::event::KeyEvent;

use super::AppTab;

#[derive(Default, Debug)]
pub struct NotWordle {}

const LABEL: &str = "Not wordle";

impl AppTab for NotWordle {
	fn label(&self) -> &'static str {
		LABEL
	}

	fn handle_key_event(&mut self, event: KeyEvent) -> Result<()> {
		Ok(())
	}
}
