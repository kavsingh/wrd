use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::{Length, Min};
use ratatui::layout::{Layout, Rect};
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::{Block, Padding, Paragraph, Row, Table, Widget};

use super::{AppTab, AppTabIo};
use crate::widgets::WordGrid;

#[derive(Debug, Clone)]
pub struct MatchWords {
	is_active: bool,
	pattern: String,
	within: String,
	include: String,
	exclude: String,
	results: Vec<&'static str>,
}

const LABEL: &str = "Match";

impl Default for MatchWords {
	fn default() -> Self {
		let pattern = "* * * **";
		let within = "trubador";
		let include = "d";
		let exclude = "";

		MatchWords {
			is_active: false,
			pattern: pattern.into(),
			within: within.into(),
			include: include.into(),
			exclude: exclude.into(),
			results: wrd_lib::match_words(pattern, include, exclude, within, None)
				.unwrap_or_default(),
		}
	}
}

impl MatchWords {
	fn render_inputs(&self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_set(border::PLAIN)
			.title(" inputs ");

		block.render(area, buf);
	}

	fn render_results(&self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_set(border::PLAIN)
			.title(" results ")
			.padding(Padding::horizontal(1));
		let grid_area = block.inner(area);

		block.render(area, buf);

		WordGrid::new(self.results.clone()).render(grid_area, buf);
	}
}

impl AppTabIo for MatchWords {
	fn label(&self) -> &'static str {
		LABEL
	}

	fn set_active(&mut self, is_active: bool) {
		self.is_active = is_active
	}
}

impl Widget for MatchWords {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let [inputs_area, results_area] = Layout::vertical([Length(12), Min(0)]).areas(area);

		self.render_inputs(inputs_area, buf);
		self.render_results(results_area, buf);
	}
}

impl AppTab for MatchWords {}
