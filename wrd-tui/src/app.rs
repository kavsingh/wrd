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
use ratatui::widgets::{Block, Paragraph, Tabs, Widget};
use ratatui::{DefaultTerminal, Frame};

use crate::app_tabs::{AppTab, AppTabIo, MatchWords, NotWordle};

#[derive(Debug, Default)]
enum Tab {
	#[default]
	MatchWords,
	NotWordle,
}

#[derive(Debug)]
pub struct App<'a> {
	match_words: MatchWords<'a>,
	not_wordle: NotWordle,
	selected_tab: Tab,
	exit: bool,
}

impl Default for App<'_> {
	fn default() -> Self {
		let mut match_words = MatchWords::default();

		match_words.set_active(true);

		Self {
			match_words,
			not_wordle: NotWordle::default(),
			selected_tab: Tab::default(),
			exit: false,
		}
	}
}

impl App<'_> {
	pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
		while !self.exit {
			terminal.draw(|frame| self.draw(frame))?;
			self.handle_events().wrap_err("handle events failed")?;
		}
		Ok(())
	}

	fn draw(&self, frame: &mut Frame) {
		frame.render_widget(self, frame.area());

		let cursor_position = self
			.match_words
			.get_cursor_position()
			.or_else(|| self.not_wordle.get_cursor_position());

		if let Some(position) = cursor_position {
			frame.set_cursor_position(position)
		}
	}

	fn handle_events(&mut self) -> Result<()> {
		let received_event = event::read()?;

		if let Event::Key(key_event) = received_event {
			self.handle_key_event(key_event)
				.wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}"))?;
		}

		self.match_words
			.handle_event(&received_event)
			.wrap_err("match words: handle events failed")?;
		self.not_wordle
			.handle_event(&received_event)
			.wrap_err("not wordle: handle events failed")?;

		Ok(())
	}

	fn get_current_tab(&self) -> &dyn AppTab {
		match self.selected_tab {
			Tab::MatchWords => &self.match_words,
			Tab::NotWordle => &self.not_wordle,
		}
	}

	fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
		match key_event.code {
			KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
				self.exit()
			}
			KeyCode::Char(c) => {
				if let Some(num) = c.to_digit(10) {
					self.go_to_tab(num as usize);
				}
			}
			_ => (),
		}

		Ok(())
	}

	fn go_to_tab(&mut self, tab_num: usize) {
		match tab_num {
			1 => {
				self.selected_tab = Tab::MatchWords;
				self.match_words.set_active(true);
				self.not_wordle.set_active(false);
			}
			2 => {
				self.selected_tab = Tab::NotWordle;
				self.match_words.set_active(false);
				self.not_wordle.set_active(true);
			}
			_ => (),
		}
	}

	fn render_header(&self, area: Rect, buf: &mut Buffer) {
		let labels = vec![self.match_words.label(), self.not_wordle.label()]
			.into_iter()
			.enumerate()
			.map(|(i, label)| format!(" {label} ({}) ", i + 1));
		let highlight_style = (Color::default(), tailwind::BLUE.c700);
		let selected_tab_index = match &self.selected_tab {
			Tab::MatchWords => 0,
			Tab::NotWordle => 1,
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

	fn render_body(&self, area: Rect, buf: &mut Buffer) {
		let block = Block::bordered()
			.border_set(border::DOUBLE)
			.title(format!(" {} ", self.get_current_tab().label()));

		let content_area = block.inner(area);

		match self.selected_tab {
			Tab::MatchWords => self.match_words.clone().render(content_area, buf),
			Tab::NotWordle => self.not_wordle.clone().render(content_area, buf),
		}

		block.render(area, buf);
	}

	fn exit(&mut self) {
		self.exit = true;
	}
}

impl Widget for &App<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let vertical = Layout::vertical([Length(3), Min(0), Length(3)]);
		let [header_area, body_area, footer_area] = vertical.areas(area);

		self.render_header(header_area, buf);
		self.render_body(body_area, buf);
		self.render_footer(footer_area, buf);
	}
}
