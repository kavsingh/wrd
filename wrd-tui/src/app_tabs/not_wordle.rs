use color_eyre::eyre::Result;
use crossterm::event::KeyCode;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::Event;
use ratatui::layout::Constraint::{Fill, Length, Min};
use ratatui::layout::{Layout, Rect};
use ratatui::style::Style;
use ratatui::style::palette::tailwind;
use ratatui::symbols::border;
use ratatui::widgets::{Block, Padding, Paragraph, StatefulWidgetRef, Widget};
use tui_input::Input;
use tui_input::backend::crossterm::EventHandler;

use super::{AppTab, AppTabIo};
use crate::app::AppState;
use crate::widgets::WordGrid;

#[derive(Default, Debug, Clone)]
pub struct NotWordle<'a> {
	guess_inputs: Vec<Input>,
	word_grid: WordGrid<'a>,
	edit_guess: Option<u16>,
	is_active: bool,
}

const LABEL: &str = "Not wordle";

impl NotWordle<'_> {
	fn add_guess(&mut self) {
		self.guess_inputs.push(Input::new("".to_string()));
		self.edit_guess = Some((self.guess_inputs.len() - 1) as u16);
	}

	fn refresh_results(&mut self) -> Result<()> {
		let mut not_wordle = wrd_lib::Notwordle::default();
		let patterns: Vec<_> = self
			.guess_inputs
			.iter()
			.map(|input| input.value())
			.collect();
		let mut results: Vec<String> = vec![];

		for pattern in patterns {
			let cleaned = pattern.trim();

			if cleaned.is_empty() {
				continue;
			}

			if let Ok((items, _)) = not_wordle.register_guess_result(cleaned) {
				results = items.iter().map(|s| s.to_string()).collect()
			}
		}

		self.word_grid.update(&results);

		Ok(())
	}

	fn forward_event_to_input(&mut self, event: &Event) {
		if let Some(input) = self
			.edit_guess
			.and_then(|i| self.guess_inputs.get_mut(i as usize))
		{
			input.handle_event(event);
		}
	}

	fn render_inputs(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
		let areas = Layout::vertical(vec![Length(1); self.guess_inputs.len() + 1]).split(area);
		let (intro, inputs) = areas.split_at_checked(1).unwrap_or_default();

		if let Some(intro_area) = intro.first() {
			Paragraph::new(" <+> add guess result")
				.style(Style::default().fg(tailwind::NEUTRAL.c500).bold())
				.render(*intro_area, buf);
		}

		for (index, layout_area) in inputs.iter().enumerate() {
			if let Some(input) = self.guess_inputs.get(index) {
				let is_active = self.edit_guess.map(|v| v == index as u16).unwrap_or(false);
				let [label_area, input_area] =
					Layout::horizontal([Length(5), Min(0)]).areas(*layout_area);

				Paragraph::new(format!(" <{}>", index_to_char(index as u16)))
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
	}

	fn render_results(&self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_set(border::PLAIN)
			.title(" Results ")
			.padding(Padding::horizontal(1));
		let grid_area = block.inner(area);

		block.render(area, buf);
		self.word_grid.clone().render(grid_area, buf);
	}
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
		LABEL
	}

	fn set_active(&mut self, is_active: bool) {
		self.is_active = is_active
	}

	fn handle_event(&mut self, event: &Event) -> Result<()> {
		if !self.is_active {
			return Ok(());
		}

		if let Event::Key(key_event) = event {
			let is_editing = self.edit_guess.is_some();

			match key_event.code {
				KeyCode::Char('+') if !is_editing => self.add_guess(),
				KeyCode::Esc => self.edit_guess = None,
				KeyCode::Enter => self.refresh_results().unwrap_or(()),
				KeyCode::Tab => match self.edit_guess {
					None => self.edit_guess = Some(0),
					Some(current) => {
						let next = (current + 1) % self.guess_inputs.len() as u16;

						self.edit_guess = Some(next);
					}
				},
				KeyCode::Char(c) if !is_editing => {
					let target_index = char_to_index(c);

					if let Some(index) = target_index {
						if index < self.guess_inputs.len() as u16 {
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
			Layout::vertical([Length(self.guess_inputs.len() as u16 + 1), Fill(1)]).areas(area);

		self.render_inputs(inputs_area, buf, state);
		self.render_results(results_area, buf);
	}
}

impl AppTab for NotWordle<'_> {}
