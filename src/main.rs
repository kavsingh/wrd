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
			match_command(pattern, include, exclude);
		}
		None => panic!("expected a command"),
	}
}

fn match_command(pattern: &str, include: &str, exclude: &str) {
	let pattern = parse_pattern(pattern);
	let include: Vec<&str> = include.split("").filter(|c| !c.is_empty()).collect();
	let exclude: Vec<&str> = exclude.split("").filter(|c| !c.is_empty()).collect();
	let result = match_from_pattern(&pattern, &include, &exclude);

	println!("{}", result.join("\n"))
}

fn parse_pattern(descriptor: &str) -> MatchPattern {
	descriptor
		.split("_")
		.map(|desc| {
			if desc.contains("*") {
				return MatchOperation::MatchAnything;
			}

			let letters: Vec<String> = desc
				.replacen("!", "", 1)
				.split("")
				.filter_map(|c| {
					if c.is_empty() {
						None
					} else {
						Some(c.to_string())
					}
				})
				.collect();

			if letters.is_empty() {
				panic!("empty descriptors not allowed")
			}

			if desc.starts_with("!") {
				MatchOperation::ExcludeAllOf(letters)
			} else {
				MatchOperation::MatchOneOf(letters)
			}
		})
		.collect()
}
