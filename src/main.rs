mod data;
mod matcher;

use clap::{Parser, Subcommand};
use matcher::match_from_pattern;

use crate::matcher::{MatchOperation, MatchPattern};

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
		/// _: seperator for char positions
		/// *: match anything in this position
		/// any number of lowercase chars: match any of the chars in the string in this position
		/// ! + any number of lowercase chars: exclude any of the chars in the string in this position
		///
		/// e.g.
		///   *_b_!ar_!r_*
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
}

fn main() {
	let cli = Cli::parse();

	match &cli.command {
		Some(Commands::Match {
			pattern,
			exclude,
			include,
		}) => {
			match_command_runner(pattern, include, exclude);
		}
		None => panic!("expected a command"),
	}
}

fn match_command_runner(pattern: &str, include: &str, exclude: &str) {
	let pattern = parse_pattern(pattern);
	let result = match_from_pattern(&pattern, include, exclude);

	println!("{}", result.join("\n"))
}

fn parse_pattern(descriptor: &str) -> MatchPattern {
	descriptor
		.split("_")
		.map(|desc| {
			if desc.contains("*") {
				return MatchOperation::MatchAny;
			}

			let letters: String = desc.chars().filter(|c| c.is_ascii() && *c != '!').collect();

			if letters.is_empty() {
				panic!("empty or non-ascii descriptors not allowed")
			}

			if desc.starts_with("!") {
				MatchOperation::ExcludeAllIn(letters)
			} else {
				MatchOperation::MatchAnyIn(letters)
			}
		})
		.collect()
}
