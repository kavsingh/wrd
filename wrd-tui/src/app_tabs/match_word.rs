use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::text::Text;
use ratatui::widgets::{Block, Paragraph, Widget};

use super::{AppTab, AppTabIo};

#[derive(Default, Debug, Clone)]
pub struct MatchWord {
	is_active: bool,
}

const LABEL: &str = "Match";

impl AppTabIo for MatchWord {
	fn label(&self) -> &'static str {
		LABEL
	}

	fn set_active(&mut self, is_active: bool) {
		self.is_active = is_active
	}
}

impl Widget for MatchWord {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let block = Block::new();

		Paragraph::new(Text::from("Match word content"))
			.centered()
			.block(block)
			.render(area, buf);
	}
}

impl AppTab for MatchWord {}
