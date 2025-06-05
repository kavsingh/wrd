use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::Min;
use ratatui::layout::Rect;
use ratatui::widgets::{Row, Table, Widget};

const COLUMNS: usize = 10;

#[derive(Default, Debug, Clone)]
pub struct WordGrid<'a> {
	rows: Vec<Row<'a>>,
}

impl WordGrid<'_> {
	pub fn update(&mut self, words: &Vec<String>) {
		self.rows = words
			.chunks(COLUMNS)
			.map(|row| Row::new::<Vec<String>>(row.into()))
			.collect();
	}
}

impl Widget for WordGrid<'_> {
	fn render(self, area: Rect, buf: &mut Buffer) {
		let widths = (0..COLUMNS).map(|_| Min(0));

		Table::new(self.rows, widths)
			.column_spacing(1)
			.render(area, buf);
	}
}
