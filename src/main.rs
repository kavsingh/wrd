use std::process;

use clap::{Parser, Subcommand};
use wrd::{match_words_runner, notwordle_runner};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Cli {
	#[command(subcommand)]
	command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
enum Commands {
	/// find word matches from patterns
	Mw {
		/// match words{n}
		/// matches lowercase ascii only. character positions seperated by space
		///
		/// wrd mw -p '** * ae !bcd **'
		///
		/// - **: match any number of any character{n}
		/// - *: match any single character{n}
		/// - a-z: match any of these chars (1 or more) in this position{n}
		/// - !a-z: exclude any of these chars (1 or more) in this position
		///
		/// e.g.{n}
		/// - match any word starting with "y" and "e"{n}
		///   wrd mw -p 'y e **'
		///
		/// - match any word ending with "r" and "y" or "n" and "y"{n}
		///   wrd mw -p '** rn y'
		///
		/// - match a five character word with "r" as 2nd char, "n" or "t"
		///   as 4th char, and not "s" or "y" as the last character{n}
		///   wrd mw -p '* r * nt !sy'
		#[arg(short, long)]
		pattern: String,

		/// word must include all of these letters
		#[arg(short, long, default_value_t = ("").to_string())]
		include: String,

		/// words must not include any of these letters
		#[arg(short, long, default_value_t = ("").to_string())]
		exclude: String,

		/// words can only contain letters within this group
		#[arg(short, long, default_value_t = ("").to_string())]
		within: String,
	},

	// see words left after guesses
	Nw {
		/// guess results: comma seperated list of guess results
		///
		/// a guess result is a space seperated list of the results of a guess:{n}
		/// - single a-z: letter in correct position{n}
		/// - ? + single a-z: letter in word but in wrong position{n}
		/// - ! + single a-z: letter not in word{n}
		///
		/// e.g. encoding the result of the guess "plate" where:{n}
		/// - 'p' is in correct position{n}
		/// - 'l' is in word but in wrong position{n}
		/// - 'a' is not in word{n}
		/// - 't' is in word but in wrong position{n}
		/// - 'e' is in correct position{n}
		///
		/// wrd nw 'p ?l !a ?t e'
		///
		/// to see words remaning from compounding guesses, provide a comma
		/// seperated list of results:
		///
		/// wrd nw 'p ?l !a ?t e,p ?o l ?i t'
		///
		#[arg(short, long)]
		guess_results: String,
	},
}

fn main() {
	let cli = Cli::parse();

	match &cli.command {
		Some(Commands::Mw {
			pattern,
			exclude,
			include,
			within,
		}) => {
			if let Err(err) = match_words_runner(pattern, include, exclude, within) {
				eprintln!("mw error: {err}");
				process::exit(1);
			}
		}
		Some(Commands::Nw { guess_results }) => {
			if let Err(err) = notwordle_runner(guess_results) {
				eprintln!("nw error: {err}");
				process::exit(1);
			}
		}
		None => {
			eprintln!("expected a command");
			process::exit(1);
		}
	}
}
