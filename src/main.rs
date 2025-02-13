mod data;
mod matcher;
mod notwordle;
mod util;

use clap::{Parser, Subcommand};
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
	Match {
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

	// see whats left from subsequent guesses
	Nw {
		#[arg(short, long)]
		guess_result: String,
	},
}

fn main() {
	let cli = Cli::parse();

	match &cli.command {
		Some(Commands::Match {
			pattern,
			exclude,
			include,
		}) => {
			match_runner(pattern, include, exclude);
		}
		Some(Commands::Nw { guess_result }) => {
			not_wordle_runner(guess_result);
		}
		None => panic!("expected a command"),
	}
}

fn match_runner(pattern: &str, include: &str, exclude: &str) {
	let pattern = parse_pattern(pattern);
	let result = match_from_pattern(&pattern, include, exclude);

	println!("{}", result.join("\n"))
}

fn not_wordle_runner(guess_result: &str) {
	let mut not_wordle = NotWordle::default();
	let guesses: Vec<&str> = guess_result.split(",").collect();
	let mut print_items: Vec<String> = vec![];

	for guess in guesses {
		match not_wordle.register_guess_result(guess) {
			Ok((items, guess_result)) => {
				println!(
					"{} remaining after {}:",
					items.len(),
					format_not_wordle_guess_result(&guess_result)
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
		.chunks(10)
		.map(|c| c.join("\t"))
		.collect::<Vec<_>>()
		.join("\n")
}

fn format_not_wordle_guess_result(result: &GuessResult) -> String {
	result
		.iter()
		.map(|result_char| match result_char {
			GuessResultChar::Right(c) => c,
			GuessResultChar::Wrong(c) => c,
			GuessResultChar::WrongPosition(c) => c,
		})
		.cloned()
		.collect::<Vec<String>>()
		.join("")
}
