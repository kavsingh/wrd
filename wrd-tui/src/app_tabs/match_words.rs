use color_eyre::eyre::Result;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Constraint::{Fill, Length, Min};
use ratatui::layout::{Layout, Rect};
use ratatui::style::Style;
use ratatui::style::palette::tailwind;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Padding, Paragraph, StatefulWidgetRef, Widget, WidgetRef};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use wrd_lib::get_dictionary;

use super::{AppTab, AppTabIo, Tab};
use crate::state::AppState;
use crate::widgets::WordGrid;

#[derive(Default, Debug, Clone, PartialEq)]
enum TargetInput {
	#[default]
	None,
	Pattern,
	Within,
	Include,
	Exclude,
}

impl TargetInput {
	fn next(&self) -> Self {
		match &self {
			TargetInput::None => TargetInput::Pattern,
			TargetInput::Pattern => TargetInput::Within,
			TargetInput::Within => TargetInput::Include,
			TargetInput::Include => TargetInput::Exclude,
			TargetInput::Exclude => TargetInput::Pattern,
		}
	}
}

#[derive(Debug)]
pub struct MatchWords<'a> {
	is_active: bool,
	target_input: TargetInput,
	pattern_input: Input,
	within_input: Input,
	include_input: Input,
	exclude_input: Input,
	results: Vec<String>,
	word_grid: WordGrid<'a>,
}

impl Default for MatchWords<'_> {
	fn default() -> Self {
		let mut word_grid = WordGrid::default();
		let pattern = "* * * **";
		let within = "trubador";
		let include = "d";
		let exclude = "";
		let results: Vec<_> = wrd_lib::match_words(pattern, include, exclude, within, None)
			.unwrap_or_default()
			.iter()
			.map(|s| s.to_string())
			.collect();

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
	fn forward_event_to_input(&mut self, event: &Event) {
		let _ = match self.target_input {
			TargetInput::Pattern => self.pattern_input.handle_event(event),
			TargetInput::Within => self.within_input.handle_event(event),
			TargetInput::Include => self.include_input.handle_event(event),
			TargetInput::Exclude => self.exclude_input.handle_event(event),
			_ => None,
		};
	}

	fn refresh_results(&mut self, state: &AppState) {
		let results = wrd_lib::match_words(
			self.pattern_input.value(),
			self.include_input.value(),
			self.exclude_input.value(),
			self.within_input.value(),
			Some(get_dictionary(&state.dictionary)),
		)
		.unwrap_or_default();

		self.results = results.iter().map(|s| s.to_string()).collect();
		self.word_grid.update(&self.results);
	}

	fn render_inputs(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
		let [pattern_area, within_area, include_area, exclude_area] =
			Layout::vertical([Length(1), Length(1), Length(1), Length(1)]).areas(area);
		let inputs = [
			(
				&self.pattern_input,
				pattern_area,
				self.target_input == TargetInput::Pattern,
				"<p>attern",
			),
			(
				&self.within_input,
				within_area,
				self.target_input == TargetInput::Within,
				"<w>ithin",
			),
			(
				&self.include_input,
				include_area,
				self.target_input == TargetInput::Include,
				"<i>nclude",
			),
			(
				&self.exclude_input,
				exclude_area,
				self.target_input == TargetInput::Exclude,
				"<e>xclude",
			),
		];

		state.cursor_position = None;

		for (input, area, is_active, label) in inputs {
			let [label_area, input_area] = Layout::horizontal([Length(18), Min(0)]).areas(area);

			Paragraph::new(format!(" {label} "))
				.style(if is_active {
					Style::default().fg(tailwind::BLUE.c600).bold()
				} else {
					Style::default().dim()
				})
				.render(label_area, buf);

			Paragraph::new(input.value())
				.scroll((0, input.visual_scroll(area.width as usize) as u16))
				.render(input_area, buf);

			if is_active {
				let scroll = input.visual_scroll(input_area.width as usize);
				let x = input.visual_cursor().max(scroll);

				state.cursor_position = Some((input_area.x + x as u16, input_area.y));
			}
		}
	}

	fn render_results(&self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_set(border::PLAIN)
			.title(" Results ")
			.padding(Padding::horizontal(1));
		let grid_area = block.inner(area);

		block.render(area, buf);
		self.word_grid.render_ref(grid_area, buf);
	}
}

impl AppTabIo for MatchWords<'_> {
	fn label(&self) -> &'static str {
		"Match"
	}

	fn tab(&self) -> Tab {
		Tab::MatchWords
	}

	fn set_active(&mut self, is_active: bool, _: &mut AppState) {
		self.is_active = is_active;
		self.target_input = TargetInput::default();
	}

	fn handle_event(&mut self, event: &Event, state: &mut AppState) -> Result<()> {
		if !self.is_active {
			return Ok(());
		}

		if let Event::Key(key_event) = event {
			let not_focused = self.target_input == TargetInput::None;

			match key_event.code {
				KeyCode::Char('p') if not_focused => self.target_input = TargetInput::Pattern,
				KeyCode::Char('w') if not_focused => self.target_input = TargetInput::Within,
				KeyCode::Char('i') if not_focused => self.target_input = TargetInput::Include,
				KeyCode::Char('e') if not_focused => self.target_input = TargetInput::Exclude,
				KeyCode::Tab => self.target_input = self.target_input.next(),
				KeyCode::Esc => self.target_input = TargetInput::None,
				KeyCode::Enter => self.refresh_results(state),
				_ => self.forward_event_to_input(event),
			}
		};

		Ok(())
	}
}

impl StatefulWidgetRef for MatchWords<'_> {
	type State = AppState;

	fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
		let [inputs_area, results_area] = Layout::vertical([Length(4), Fill(1)]).areas(area);

		self.render_inputs(inputs_area, buf, state);
		self.render_results(results_area, buf);
	}
}

impl AppTab for MatchWords<'_> {}
