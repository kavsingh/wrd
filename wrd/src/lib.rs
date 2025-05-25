use std::error::Error;

use colored::Colorize;
use wrd_lib::{GuessResultToken, Notwordle, match_words};

pub fn match_words_runner(
	pattern: &str,
	include: &str,
	exclude: &str,
	within: &str,
) -> Result<(), Box<dyn Error>> {
	let result = match_words(pattern, include, exclude, within, None)?;

	println!("{}", format_word_grid(&result));

	Ok(())
}

pub fn notwordle_runner(guess_results: &str) -> Result<(), Box<dyn Error>> {
	let mut notwordle = Notwordle::default();
	let results: Vec<&str> = guess_results.split(",").collect();
	let mut print_items: Vec<&str> = vec![];

	for result in results {
		let (items, parsed_result) = notwordle.register_guess_result(result)?;

		println!(
			"{} remaining after {}",
			items.len(),
			format_notwordle_guess_result(&parsed_result)
		);
		print_items = items;
	}

	println!("{}", format_word_grid(&print_items));

	Ok(())
}

fn format_word_grid(words: &[&str]) -> String {
	words
		.chunks(14)
		.map(|c| {
			c.iter()
				.fold("".to_string(), |s, c| format!("{s}\t{}", c.dimmed()))
		})
		.collect::<Vec<_>>()
		.join("\n")
}

fn format_notwordle_guess_result(result: &[GuessResultToken]) -> String {
	result
		.iter()
		.map(|result_char| match result_char {
			GuessResultToken::Right(c) => c.bright_yellow().underline(),
			GuessResultToken::WrongPosition(c) => c.blue(),
			GuessResultToken::Wrong(c) => c.dimmed(),
		})
		.fold("".to_string(), |s, c| format!("{s}{c}"))
}
