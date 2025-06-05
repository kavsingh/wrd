use color_eyre::Result;
use color_eyre::eyre::WrapErr;

use crate::app::App;

mod app;
mod app_tabs;
mod widgets;

fn main() -> Result<()> {
	color_eyre::install()?;

	let mut terminal = ratatui::init();
	let app_result = App::default().run(&mut terminal);

	ratatui::restore();
	app_result.wrap_err("application run failed")
}
