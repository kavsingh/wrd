use color_eyre::Result;
use color_eyre::eyre::WrapErr;
use ratatui::buffer::Buffer;
use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::palette::tailwind;
use ratatui::style::{Color, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Block, Paragraph, Tabs, Widget};
use ratatui::{DefaultTerminal, Frame};

use crate::app_tabs::{AppTab, AppTabIo, MatchWord, NotWordle};

#[derive(Debug, Default)]
enum Tab {
	#[default]
	Match,
	Wordle,
}

#[derive(Default, Debug)]
pub struct App {
	match_word: MatchWord,
	not_wordle: NotWordle,
	selected_tab: Tab,
	exit: bool,
}

impl App {
	pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
		while !self.exit {
			terminal.draw(|frame| self.draw(frame))?;
			self.handle_events().wrap_err("handle events failed")?;
		}
		Ok(())
	}

	fn draw(&self, frame: &mut Frame) {
		frame.render_widget(self, frame.area());
	}

	fn handle_events(&mut self) -> Result<()> {
		match event::read()? {
			Event::Key(key_event) => self
				.handle_key_event(key_event)
				.wrap_err_with(|| format!("handling key event failed:\n{key_event:#?}")),
			_ => Ok(()),
		}
	}

	fn get_current_tab(&self) -> &dyn AppTab {
		match self.selected_tab {
			Tab::Match => &self.match_word,
			Tab::Wordle => &self.not_wordle,
		}
	}

	fn handle_key_event(&mut self, key_event: KeyEvent) -> Result<()> {
		let _current_tab = self.get_current_tab();

		if key_event.kind != KeyEventKind::Press {
			return Ok(());
			// @TODO
			// return current_tab.handle_key_event(key_event);
		}

		match key_event.code {
			// @TODO: can we use abort signals etc?
			KeyCode::Char('c') if key_event.modifiers.contains(KeyModifiers::CONTROL) => {
				self.exit()
			}
			KeyCode::Char(c) => {
				if let Some(num) = c.to_digit(10) {
					self.go_to_tab(num as usize);
				}
			}
			_ => {
				// @TODO
				// return current_tab.handle_key_event(key_event);
			}
		}

		Ok(())
	}

	fn go_to_tab(&mut self, tab_num: usize) {
		match tab_num {
			1 => self.selected_tab = Tab::Match,
			2 => self.selected_tab = Tab::Wordle,
			_ => (),
		}
	}

	fn render_header(&self, area: Rect, buf: &mut Buffer) {
		let labels = vec![self.match_word.label(), self.not_wordle.label()]
			.into_iter()
			.enumerate()
			.map(|(i, label)| format!(" {label} ({}) ", i + 1));
		let highlight_style = (Color::default(), tailwind::BLUE.c700);
		let selected_tab_index = match &self.selected_tab {
			Tab::Match => 0,
			Tab::Wordle => 1,
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
			.border_set(border::THICK)
			.title(format!(" {} ", self.get_current_tab().label()));

		Paragraph::new(Text::from("here we are"))
			.centered()
			.block(block)
			.render(area, buf);
	}

	fn exit(&mut self) {
		self.exit = true;
	}
}

impl Widget for &App {
	fn render(self, area: Rect, buf: &mut Buffer) {
		use Constraint::{Length, Min};
		let vertical = Layout::vertical([Length(3), Min(0), Length(3)]);
		let [header_area, body_area, footer_area] = vertical.areas(area);

		self.render_header(header_area, buf);
		self.render_body(body_area, buf);
		self.render_footer(footer_area, buf);
	}
}
