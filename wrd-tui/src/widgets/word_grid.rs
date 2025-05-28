use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::Min;
use ratatui::layout::Rect;
use ratatui::widgets::{Row, Table, Widget};

#[derive(Default, Debug, Clone)]
pub struct WordGrid {
	words: Vec<&'static str>,
}

impl WordGrid {
	pub fn new(words: Vec<&'static str>) -> Self {
		Self { words }
	}
}

impl Widget for WordGrid {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let columns: usize = 10;
		let widths = (0..columns).map(|_| Min(0));
		let rows: Vec<Row> = self
			.words
			.chunks(columns)
			.map(|row| Row::new::<Vec<&str>>(row.into()))
			.collect();

		Table::new(rows, widths).column_spacing(1).render(area, buf);
	}
}
