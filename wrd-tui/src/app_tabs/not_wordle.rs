use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Constraint::{Fill, Length, Min};
use ratatui::layout::{Layout, Rect};
use ratatui::style::palette::tailwind;
use ratatui::style::{Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph, StatefulWidgetRef, Widget, WidgetRef};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;
use wrd_lib::{GuessResultToken, get_dictionary};

use super::{AppTab, AppTabIo, Tab};
use crate::state::AppState;
use crate::widgets::WordGrid;

#[derive(Debug)]
struct GuessResult {
	input: Input,
	tokenized: Option<Vec<GuessResultToken>>,
}

impl Default for GuessResult {
	fn default() -> Self {
		Self {
			input: Input::new("".to_string()),
			tokenized: None,
		}
	}
}

#[derive(Default, Debug)]
pub struct NotWordle<'a> {
	guesses: Vec<GuessResult>,
	word_grid: WordGrid<'a>,
	edit_guess: Option<u16>,
	results: Vec<String>,
	is_active: bool,
}

impl NotWordle<'_> {
	fn add_guess(&mut self) {
		self.guesses.push(GuessResult::default());
		self.edit_guess = Some((self.guesses.len() - 1) as u16);
	}

	fn refresh_results(&mut self, state: &AppState) -> Result<()> {
		let mut not_wordle = wrd_lib::Notwordle::default();

		for guess in self.guesses.iter_mut() {
			guess.tokenized = not_wordle
				.register_guess_result(guess.input.value().trim())
				.ok()
		}

		match not_wordle.refine(Some(get_dictionary(&state.dictionary))) {
			Ok(results) => {
				self.results = results.iter().map(|s| s.to_string()).collect();
				self.word_grid.update(&self.results);

				Ok(())
			}
			// @TODO: surface error
			Err(_) => Ok(()),
		}
	}

	fn forward_event_to_input(&mut self, event: &Event) {
		if let Some(guess) = self
			.edit_guess
			.and_then(|i| self.guesses.get_mut(i as usize))
		{
			guess.input.handle_event(event);
		}
	}

	fn render_inputs(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
		let areas = Layout::vertical(vec![Length(1); self.guesses.len() + 1]).split(area);
		let (intro, inputs) = areas.split_at_checked(1).unwrap_or_default();

		if let Some(intro_area) = intro.first() {
			Paragraph::new(" <+> add guess result")
				.style(Style::default().fg(tailwind::NEUTRAL.c500).bold())
				.render(*intro_area, buf);
		}

		for (index, layout_area) in inputs.iter().enumerate() {
			if let Some(guess) = self.guesses.get(index) {
				let is_active = self.edit_guess.map(|v| v == index as u16).unwrap_or(false);
				let [label_area, input_area, formatted_area] =
					Layout::horizontal([Length(5), Length(16), Min(20)]).areas(*layout_area);

				Paragraph::new(format!(" <{}>", index_to_char(index as u16)))
					.style(if is_active {
						Style::default().fg(tailwind::BLUE.c600).bold()
					} else {
						Style::default().dim()
					})
					.render(label_area, buf);

				Paragraph::new(guess.input.value())
					.scroll((
						0,
						guess.input.visual_scroll(input_area.width as usize) as u16,
					))
					.render(input_area, buf);

				if let Some(tokenized) = &guess.tokenized {
					let formatted = format_tokenized(tokenized);

					Paragraph::new(Line::from(formatted)).render(formatted_area, buf);
				} else {
					Paragraph::new("").render(formatted_area, buf);
				}

				if is_active {
					let scroll = guess.input.visual_scroll(input_area.width as usize);
					let x = guess.input.visual_cursor().max(scroll);

					state.cursor_position = Some((input_area.x + x as u16, input_area.y));
				}
			}
		}
	}

	fn commit_guess(&mut self, state: &AppState) {
		self.refresh_results(state).unwrap_or(());

		if let Some(index) = self.edit_guess {
			if index == (self.guesses.len() as u16) - 1 {
				self.add_guess();
			} else {
				self.go_to_next_guess();
			}
		}
	}

	fn go_to_next_guess(&mut self) {
		match self.edit_guess {
			None => self.edit_guess = Some(0),
			Some(current) => {
				let next = (current + 1) % self.guesses.len() as u16;

				self.edit_guess = Some(next);
			}
		}
	}

	fn render_results(&self, area: Rect, buf: &mut Buffer) {
		let title = if self.guesses.is_empty() {
			" Enter a guess result ".to_string()
		} else {
			format!(" {} words remaining ", self.results.len())
		};

		let block = Block::bordered()
			.border_set(border::PLAIN)
			.title(title)
			.padding(Padding::horizontal(1));
		let grid_area = block.inner(area);

		block.render(area, buf);
		self.word_grid.render_ref(grid_area, buf);
	}
}

fn format_tokenized(tokenized: &[GuessResultToken]) -> Vec<Span> {
	tokenized
		.iter()
		.cloned()
		.map(|token| match token {
			GuessResultToken::Right(c) => upper_span(c)
				.bg(tailwind::ORANGE.c400)
				.fg(tailwind::WHITE)
				.bold(),
			GuessResultToken::WrongPosition(c) => upper_span(c)
				.bg(tailwind::BLUE.c400)
				.fg(tailwind::WHITE)
				.bold(),
			GuessResultToken::Wrong(c) => upper_span(c).dim(),
		})
		.collect()
}

fn upper_span<'a>(c: String) -> Span<'a> {
	Span::from(format!(" {} ", c.to_uppercase()))
}

// a = ascii 97
fn index_to_char(index: u16) -> char {
	(index as u8 + 97) as char
}

fn char_to_index(ch: char) -> Option<u16> {
	let as_8 = ch as u8;

	if as_8 < 97 {
		None
	} else {
		Some((as_8 - 97) as u16)
	}
}

impl AppTabIo for NotWordle<'_> {
	fn label(&self) -> &'static str {
		"Not wordle"
	}

	fn tab(&self) -> Tab {
		Tab::NotWordle
	}

	fn set_active(&mut self, is_active: bool, _: &mut AppState) {
		self.is_active = is_active
	}

	fn handle_event(&mut self, event: &Event, state: &mut AppState) -> Result<()> {
		if !self.is_active {
			return Ok(());
		}

		if let Event::Key(key_event) = event {
			let is_editing = self.edit_guess.is_some();

			match key_event.code {
				KeyCode::Char('+') if !is_editing => self.add_guess(),
				KeyCode::Esc => self.edit_guess = None,
				KeyCode::Enter if is_editing => self.commit_guess(state),
				KeyCode::Tab => self.go_to_next_guess(),
				// Skip digit keys (0-9) as they are handled by the main app for tab switching
				KeyCode::Char(c) if c.is_ascii_digit() => (),
				KeyCode::Char(c) if !is_editing => {
					let target_index = char_to_index(c);

					if let Some(index) = target_index {
						if index < self.guesses.len() as u16 {
							self.edit_guess = Some(index);
						};
					}
				}
				_ if is_editing => self.forward_event_to_input(event),
				_ => (),
			}
		};

		Ok(())
	}
}

impl StatefulWidgetRef for NotWordle<'_> {
	type State = AppState;

	fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
		let [inputs_area, results_area] =
			Layout::vertical([Length(self.guesses.len() as u16 + 1), Fill(1)]).areas(area);

		self.render_inputs(inputs_area, buf, state);
		self.render_results(results_area, buf);
	}
}

impl AppTab for NotWordle<'_> {}
