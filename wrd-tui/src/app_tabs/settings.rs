use color_eyre::eyre::Result;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{Event, KeyCode};
use ratatui::layout::Constraint::Length;
use ratatui::layout::{Layout, Rect};
use ratatui::style::Style;
use ratatui::style::palette::tailwind;
use ratatui::widgets::{Block, Padding, Paragraph, StatefulWidgetRef, Widget};
use wrd_lib::Dictionary;

use super::{AppTab, AppTabIo, Tab};
use crate::app::AppState;

#[derive(Debug)]
pub struct Settings {
	is_active: bool,
	highlighted_dict_index: Option<usize>,
	dict_options: Vec<Dictionary>,
}

impl Default for Settings {
	fn default() -> Self {
		Self {
			is_active: false,
			highlighted_dict_index: None,
			dict_options: vec![Dictionary::Moby, Dictionary::Gwicks],
		}
	}
}

impl Settings {
	fn get_highlighted_dict(&self) -> Option<&Dictionary> {
		self.highlighted_dict_index
			.map(|index| self.dict_options.get(index))?
	}

	fn get_dict_option_index(&self, dict: &Dictionary) -> Option<usize> {
		for (index, option) in self.dict_options.iter().enumerate() {
			if option == dict {
				return Some(index);
			}
		}

		None
	}

	fn render_dictionary_select(&self, area: Rect, buf: &mut Buffer, state: &AppState) {
		let block = Block::bordered()
			.padding(Padding::horizontal(1))
			.title(" Select dictionary ");
		let option_areas =
			Layout::vertical(vec![Length(1); self.dict_options.len()]).split(block.inner(area));

		for (index, option_area) in option_areas.iter().enumerate() {
			if let Some(dict) = self.dict_options.get(index) {
				let is_selected = *dict == state.dictionary;
				let is_highlighted = match self.get_highlighted_dict() {
					Some(highlighted) => *highlighted == *dict,
					None => false,
				};

				Paragraph::new(format!(
					" {} {} ",
					if is_selected { "⦿" } else { "○" },
					dict.name(),
				))
				.style(if is_highlighted {
					Style::default().bg(tailwind::BLUE.c700)
				} else {
					Style::default()
				})
				.render(*option_area, buf);
			}
		}

		block.render(area, buf);
	}
}

impl AppTabIo for Settings {
	fn tab(&self) -> Tab {
		Tab::Settings
	}

	fn label(&self) -> &'static str {
		"Settings"
	}

	fn set_active(&mut self, is_active: bool, state: &mut AppState) {
		self.is_active = is_active;

		if !is_active {
			return;
		}

		self.highlighted_dict_index = self.get_dict_option_index(&state.dictionary);
	}

	fn handle_event(&mut self, event: &Event, state: &mut AppState) -> Result<()> {
		if !self.is_active {
			return Ok(());
		}

		if let Event::Key(key_event) = event {
			match key_event.code {
				KeyCode::Tab => {
					self.highlighted_dict_index = match self.highlighted_dict_index {
						Some(index) => Some((index + 1) % self.dict_options.len()),
						None => Some(0),
					}
				}
				KeyCode::Enter => {
					if let Some(dict) = self.get_highlighted_dict() {
						state.dictionary = dict.clone();
					}
				}
				_ => (),
			}
		};

		Ok(())
	}
}

impl StatefulWidgetRef for Settings {
	type State = AppState;

	fn render_ref(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
		let block = Block::new().padding(Padding::uniform(1));
		let [dict_area] =
			Layout::vertical([Length(self.dict_options.len() as u16 + 2)]).areas(block.inner(area));

		self.render_dictionary_select(dict_area, buf, state);
		block.render(area, buf);
	}
}

impl AppTab for Settings {}
