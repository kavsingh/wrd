mod data;
mod matcher;
mod notwordle;
mod util;

use clap::{Parser, Subcommand};
use colored::Colorize;
use matcher::{match_from_pattern, parse_pattern};
use notwordle::{GuessResult, GuessResultChar, NotWordle};

/// find those words
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
	/// find matches from patterns
	Mp {
		/// match pattern
		/// character positions seperated by space
		/// *: match anything in this position
		/// any number of lowercase chars: match any of the chars in the string in this position
		/// ! + any number of lowercase chars: exclude any of the chars in the string in this position
		///
		/// e.g.
		///   '* b !ar !r *'
		///
		///   1st position - match anything
		///   2nd position - match 'b'
		///   3rd position - do not match 'a' or 'r'
		///   4th position - do not match 'r'
		///   5th position - match anything
		#[arg(short, long)]
		pattern: String,

		/// letters to always exclude
		#[arg(short, long, default_value_t = ("").to_string())]
		exclude: String,

		/// letters to always include
		#[arg(short, long, default_value_t = ("").to_string())]
		include: String,
	},

	// see words left after guesses
	Nw {
		/// guess results: comma seperated list of guess results
		///
		/// a guess result is a space seperated list of the results of a guess:
		///   - single a-z: letter in correct position
		///   - ? + single a-z: letter in word but in wrong position
		///   - ! + single a-z: letter not in word
		///
		/// e.g. encoding the result of the guess "plate" where
		///   - 'p' is in correct position
		///   - 'l' is in word but in wrong position
		///   - 'a' is not in word
		///   - 't' is in word but in wrong position
		///   - 'e' is in correct position
		///
		///   enter as: 'p ?l !a ?t e'
		///
		///   to see words remaning from compounding guesses, provide a comma
		///   seperated list of results
		///
		///   'p ?l !a ?t e,p ?o l ?i t'
		///
		#[arg(short, long)]
		guess_results: String,
	},
}

fn main() {
	let cli = Cli::parse();

	match &cli.command {
		Some(Commands::Mp {
			pattern,
			exclude,
			include,
		}) => {
			match_runner(pattern, include, exclude);
		}
		Some(Commands::Nw { guess_results }) => {
			not_wordle_runner(guess_results);
		}
		None => panic!("expected a command"),
	}
}

fn match_runner(pattern: &str, include: &str, exclude: &str) {
	let pattern = parse_pattern(pattern);
	let result = match_from_pattern(&pattern, include, exclude);

	println!("{}", format_word_grid(&result));
}

fn not_wordle_runner(guess_results: &str) {
	let mut not_wordle = NotWordle::default();
	let results: Vec<&str> = guess_results.split(",").collect();
	let mut print_items: Vec<String> = vec![];

	for result in results {
		match not_wordle.register_guess_result(result) {
			Ok((items, parsed_result)) => {
				println!(
					"{} remaining after {}",
					items.len(),
					format_not_wordle_guess_result(&parsed_result)
				);
				print_items = items;
			}
			Err(e) => println!("Error: {}", e),
		};
	}

	println!("{}", format_word_grid(&print_items));
}

fn format_word_grid(words: &[String]) -> String {
	words
		.chunks(14)
		.map(|c| {
			c.iter()
				.fold("".to_string(), |s, c| format!("{}\t{}", s, c.dimmed()))
		})
		.collect::<Vec<_>>()
		.join("\n")
}

fn format_not_wordle_guess_result(result: &GuessResult) -> String {
	result
		.iter()
		.map(|result_char| match result_char {
			GuessResultChar::Right(c) => c.bright_yellow().underline(),
			GuessResultChar::WrongPosition(c) => c.blue(),
			GuessResultChar::Wrong(c) => c.dimmed(),
		})
		.fold("".to_string(), |s, c| format!("{}{}", s, c))
}
