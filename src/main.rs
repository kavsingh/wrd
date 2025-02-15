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
		/// match words
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
