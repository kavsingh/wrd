use color_eyre::Result;
use color_eyre::eyre::WrapErr;

fn main() -> Result<()> {
	color_eyre::install()?;

	let mut terminal = ratatui::init();
	let app_result = wrd_tui::App::default().run(&mut terminal);

	ratatui::restore();
	app_result.wrap_err("application run failed")
}
