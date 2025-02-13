mod data;
mod matcher;

use clap::Parser;
use matcher::match_from_pattern;

use crate::matcher::{MatchOperation, MatchPattern};

// wrd -p *_b_!ar_!r_* -e stlen -i ard

/// find those words
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
	/// search pattern
	#[arg(short, long)]
	pattern: String,

	/// letters to always exclude
	#[arg(short, long, default_value_t = ("").to_string())]
	exclude: String,

	/// letters to always include
	#[arg(short, long, default_value_t = ("").to_string())]
	include: String,
}

fn main() {
	let args = Args::parse();
	let pattern = parse_pattern(&args.pattern);
	let include: Vec<&str> = args.include.split("").filter(|c| !c.is_empty()).collect();
	let exclude: Vec<&str> = args.exclude.split("").filter(|c| !c.is_empty()).collect();
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
