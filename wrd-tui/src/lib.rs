mod app_tabs;
mod state;
mod widgets;

use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::layout::Constraint::{Length, Min};
use ratatui::layout::{Layout, Rect};
use ratatui::style::palette::tailwind;
use ratatui::style::{Color, Stylize};
use ratatui::symbols::border;
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, StatefulWidget, StatefulWidgetRef, Tabs, Widget};
use ratatui::{DefaultTerminal, Frame};

use crate::app_tabs::{AppTab, AppTabIo, MatchWords, NotWordle, Settings, Tab};
use crate::state::AppState;

#[derive(Default, Debug)]
pub struct App<'a> {
	match_words: MatchWords<'a>,
	not_wordle: NotWordle<'a>,
	settings: Settings,
	selected_tab: Tab,
	exit: bool,
}

impl App<'_> {
	pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
		let mut state = AppState::default();

		self.match_words.set_active(true, &mut state);

		while !self.exit {
			terminal.draw(|frame| self.draw(frame, &mut state))?;
			self.handle_events(&mut state)
				.wrap_err("handle events failed")?;
		}
		Ok(())
	}

	fn draw(&self, frame: &mut Frame, state: &mut AppState) {
		frame.render_stateful_widget(self, frame.area(), state);

		if let Some(position) = state.cursor_position {
			frame.set_cursor_position(position)
		}
	}

	fn handle_events(&mut self, state: &mut AppState) -> Result<()> {
		let received_event = event::read()?;

		if let Event::Key(key_event) = received_event {
			self.handle_key_event(key_event, state)
				.wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))?;
		}

		self.match_words
			.handle_event(&received_event, state)
			.wrap_err("match words: handle events failed")?;
		self.not_wordle
			.handle_event(&received_event, state)
			.wrap_err("not wordle: handle events failed")?;
		self.settings
			.handle_event(&received_event, state)
			.wrap_err("settings: handle events failed")?;

		Ok(())
	}

	fn get_current_tab(&self) -> &dyn AppTab<State = AppState> {
		match self.selected_tab {
			Tab::MatchWords => &self.match_words,
			Tab::NotWordle => &self.not_wordle,
			Tab::Settings => &self.settings,
		}
	}

	fn handle_key_event(&mut self, key_event: KeyEvent, state: &mut AppState) -> Result<()> {
		match key_event.code {
			KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
				self.exit()
			}
			KeyCode::Char(c) => {
				if let Some(num) = c.to_digit(10) {
					self.go_to_tab(num as usize, state);
				}
			}
			_ => (),
		}

		Ok(())
	}

	fn go_to_tab(&mut self, tab_num: usize, state: &mut AppState) {
		match tab_num {
			1 => {
				state.cursor_position = None;
				self.selected_tab = Tab::MatchWords;
				self.match_words.set_active(true, state);
				self.not_wordle.set_active(false, state);
				self.settings.set_active(false, state);
			}
			2 => {
				state.cursor_position = None;
				self.selected_tab = Tab::NotWordle;
				self.match_words.set_active(false, state);
				self.not_wordle.set_active(true, state);
				self.settings.set_active(false, state);
			}
			3 => {
				state.cursor_position = None;
				self.selected_tab = Tab::Settings;
				self.match_words.set_active(false, state);
				self.not_wordle.set_active(false, state);
				self.settings.set_active(true, state);
			}
			_ => (),
		}
	}

	fn render_header(&self, area: Rect, buf: &mut Buffer) {
		let labels = vec![
			self.match_words.label(),
			self.not_wordle.label(),
			self.settings.label(),
		]
		.into_iter()
		.enumerate()
		.map(|(i, label)| format!(" {label} ({}) ", i + 1));
		let highlight_style = (Color::default(), tailwind::BLUE.c700);
		let selected_tab_index = match &self.selected_tab {
			Tab::MatchWords => 0,
			Tab::NotWordle => 1,
			Tab::Settings => 2,
		};
		let block = Block::bordered()
			.title(Line::from(" WRD ".bold()))
			.border_set(border::PLAIN);

		Tabs::new(labels)
			.highlight_style(highlight_style)
			.select(selected_tab_index)
			.padding("", "")
			.divider(" ")
			.block(block)
			.render(area, buf);
	}

	fn render_footer(&self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered().border_set(border::PLAIN);
		let instructions = Line::from(vec![
			"<1-9>".blue().bold(),
			" Go To Tab (n)  ".into(),
			"<Ctrl+C>".blue().bold(),
			" Quit".into(),
		]);

		Paragraph::new(instructions)
			.centered()
			.block(block)
			.render(area, buf);
	}

	fn render_body(&self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
		let block = Block::bordered()
			.border_set(border::DOUBLE)
			.title(format!(" {} ", self.get_current_tab().label()));

		let content_area = block.inner(area);

		match self.selected_tab {
			Tab::MatchWords => self.match_words.render_ref(content_area, buf, state),
			Tab::NotWordle => self.not_wordle.render_ref(content_area, buf, state),
			Tab::Settings => self.settings.render_ref(content_area, buf, state),
		}

		block.render(area, buf);
	}

	fn exit(&mut self) {
		self.exit = true;
	}
}

impl StatefulWidget for &App<'_> {
	type State = AppState;

	fn render(self, area: Rect, buf: &mut Buffer, state: &mut AppState) {
		let vertical = Layout::vertical([Length(3), Min(0), Length(3)]);
		let [header_area, body_area, footer_area] = vertical.areas(area);

		self.render_header(header_area, buf);
		self.render_body(body_area, buf, state);
		self.render_footer(footer_area, buf);
	}
}
