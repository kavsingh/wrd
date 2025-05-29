use std::sync::Arc;

use color_eyre::eyre::Result;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent};
use ratatui::layout::Constraint::{Length, Min};
use ratatui::layout::{Layout, Rect};
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::{Block, Padding, Paragraph, Row, Table, Widget};

use super::{AppTab, AppTabIo};
use crate::widgets::WordGrid;

#[derive(Default, Debug, Clone)]
enum TargetInput {
	#[default]
	None,
	Pattern,
	Within,
	Include,
	Exclude,
}

#[derive(Debug, Clone)]
pub struct MatchWords<'a> {
	is_active: bool,
	target_input: TargetInput,
	pattern: String,
	within: String,
	include: String,
	exclude: String,
	results: Vec<&'static str>,
	word_grid: WordGrid<'a>,
}

const LABEL: &str = "Match";

impl Default for MatchWords<'_> {
	fn default() -> Self {
		let pattern = "* * * **";
		let within = "trubador";
		let include = "d";
		let exclude = "";

		MatchWords {
			target_input: TargetInput::default(),
			is_active: false,
			pattern: pattern.into(),
			within: within.into(),
			include: include.into(),
			exclude: exclude.into(),
			results: wrd_lib::match_words(pattern, include, exclude, within, None)
				.unwrap_or_default(),
			word_grid: WordGrid::default(),
		}
	}
}

impl MatchWords<'_> {
	fn handle_key_event(&mut self, event: &KeyEvent) -> Result<()> {
		match event.code {
			KeyCode::Char('p') => self.target_input = TargetInput::Pattern,
			KeyCode::Char('w') => self.target_input = TargetInput::Within,
			KeyCode::Char('i') => self.target_input = TargetInput::Include,
			KeyCode::Char('e') => self.target_input = TargetInput::Exclude,
			KeyCode::Esc => self.target_input = TargetInput::None,
			_ => (),
		}

		Ok(())
	}

	fn render_inputs(&self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_set(border::PLAIN)
			.title(" inputs ");
		let input_text = match self.target_input {
			TargetInput::None => "None",
			TargetInput::Pattern => "Pattern",
			TargetInput::Within => "Within",
			TargetInput::Include => "Include",
			TargetInput::Exclude => "Exclude",
		};

		// block.render(area, buf);
		Paragraph::new(Text::from(input_text))
			.centered()
			.block(block)
			.render(area, buf);
	}

	fn render_results(&mut self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_set(border::PLAIN)
			.title(" results ")
			.padding(Padding::horizontal(1));
		let grid_area = block.inner(area);

		block.render(area, buf);

		self.word_grid.update(&self.results);
		self.word_grid.clone().render(grid_area, buf);
	}
}

impl AppTabIo for MatchWords<'_> {
	fn label(&self) -> &'static str {
		LABEL
	}

	fn set_active(&mut self, is_active: bool) {
		self.is_active = is_active;
		self.target_input = TargetInput::default();
	}

	fn handle_event(&mut self, event: &Event) -> Result<()> {
		if !self.is_active {
			return Ok(());
		}

		match event {
			Event::Key(key_event) => self.handle_key_event(key_event),
			_ => Ok(()),
		}
	}
}

impl Widget for MatchWords<'_> {
	fn render(mut self, area: Rect, buf: &mut Buffer) {
		let [inputs_area, results_area] = Layout::vertical([Length(12), Min(0)]).areas(area);

		self.render_inputs(inputs_area, buf);
		self.render_results(results_area, buf);
	}
}

impl AppTab for MatchWords<'_> {}
