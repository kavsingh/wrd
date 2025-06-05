use ratatui::buffer::Buffer;
use ratatui::layout::Constraint::Min;
use ratatui::layout::Rect;
use ratatui::widgets::{Row, Table, Widget, WidgetRef};

const COLUMNS: usize = 10;

#[derive(Default, Debug, Clone)]
pub struct WordGrid<'a> {
	rows: Vec<Row<'a>>,
}

impl WordGrid<'_> {
	pub fn update(&mut self, words: &[String]) {
		self.rows = words
			.chunks(COLUMNS)
			.map(|row| Row::new::<Vec<String>>(row.into()))
			.collect();
	}
}

impl WidgetRef for WordGrid<'_> {
	fn render_ref(&self, area: Rect, buf: &mut Buffer) {
		let widths = (0..COLUMNS).map(|_| Min(0));

		Table::new(self.rows.clone(), widths)
			.column_spacing(1)
			.render(area, buf);
	}
}
