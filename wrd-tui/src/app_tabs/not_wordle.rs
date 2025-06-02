use color_eyre::eyre::Result;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyEvent};
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, StatefulWidgetRef, Widget};

use super::{AppTab, AppTabIo};
use crate::app::AppState;

#[derive(Default, Debug, Clone)]
pub struct NotWordle {
	is_active: bool,
}

const LABEL: &str = "Not wordle";

impl NotWordle {
	fn handle_key_event(&mut self, _event: &KeyEvent) -> Result<()> {
		Ok(())
	}
}

impl AppTabIo for NotWordle {
	fn label(&self) -> &'static str {
		LABEL
	}

	fn set_active(&mut self, is_active: bool) {
		self.is_active = is_active
	}

	fn handle_event(&mut self, event: &Event) -> Result<()> {
		match event {
			Event::Key(key_event) => self.handle_key_event(key_event),
			_ => Ok(()),
		}
	}
}

impl StatefulWidgetRef for NotWordle {
	type State = AppState;

	fn render_ref(&self, area: Rect, buf: &mut Buffer, _state: &mut AppState) {
		let block = Block::new();

		Paragraph::new(Text::from("Not wordle content"))
			.centered()
			.block(block)
			.render(area, buf);
	}
}

impl AppTab for NotWordle {}
