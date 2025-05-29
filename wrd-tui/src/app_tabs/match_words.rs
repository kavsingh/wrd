use color_eyre::eyre::Result;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode, KeyEvent};
use ratatui::layout::Constraint::{Length, Min};
use ratatui::layout::{Layout, Rect};
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::{Block, Padding, Paragraph, Widget};
use tui_input::Input;

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
	pattern_input: Input,
	within_input: Input,
	include_input: Input,
	exclude_input: Input,
	results: Vec<&'static str>,
	word_grid: WordGrid<'a>,
}

const LABEL: &str = "Match";

impl Default for MatchWords<'_> {
	fn default() -> Self {
		let mut word_grid = WordGrid::default();
		let pattern = "* * * **";
		let within = "trubador";
		let include = "d";
		let exclude = "";
		let results =
			wrd_lib::match_words(pattern, include, exclude, within, None).unwrap_or_default();

		word_grid.update(&results);

		Self {
			target_input: TargetInput::default(),
			is_active: false,
			pattern_input: Input::new(pattern.into()),
			within_input: Input::new(within.into()),
			include_input: Input::new(include.into()),
			exclude_input: Input::new(exclude.into()),
			results,
			word_grid,
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
			KeyCode::Enter => self.refresh_results(),
			_ => (),
		}

		Ok(())
	}

	fn refresh_results(&mut self) {
		let results = wrd_lib::match_words(
			self.pattern_input.value(),
			self.include_input.value(),
			self.exclude_input.value(),
			self.within_input.value(),
			None,
		)
		.unwrap_or_default();

		self.results = results;
		self.word_grid.update(&self.results);
	}

	fn render_inputs(&self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_set(border::PLAIN)
			.title(" Inputs ");

		let block_area = block.inner(area);

		block.render(area, buf);

		// block.render(area, buf);
		let width = area.width.max(3) - 3;
		let scroll = self.pattern_input.visual_scroll(width as usize);

		Paragraph::new(self.pattern_input.value())
			.scroll((0, scroll as u16))
			.block(Block::bordered().title(" Pattern "))
			.render(block_area, buf);

		// if self.input_mode == InputMode::Editing {
		// 	let x = self.pattern_input.visual_cursor().max(scroll) - scroll + 1;
		// 	frame.set_cursor_position((area.x + x as u16, area.y + 1))
		// }
	}

	fn render_results(&mut self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_set(border::PLAIN)
			.title(" Results ")
			.padding(Padding::horizontal(1));
		let grid_area = block.inner(area);

		block.render(area, buf);
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
