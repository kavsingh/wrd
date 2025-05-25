use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, Widget};

use super::{AppTab, AppTabIo};

#[derive(Default, Debug, Clone)]
pub struct NotWordle {
	is_active: bool,
}

const LABEL: &str = "Not wordle";

impl AppTabIo for NotWordle {
	fn label(&self) -> &'static str {
		LABEL
	}

	fn set_active(&mut self, is_active: bool) {
		self.is_active = is_active
	}
}

impl Widget for NotWordle {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let block = Block::new();

		Paragraph::new(Text::from("Not wordle content"))
			.centered()
			.block(block)
			.render(area, buf);
	}
}

impl AppTab for NotWordle {}
