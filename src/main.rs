mod data;

use crate::data::get_words;
use clap::Parser;

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

	println!("{:?}", pattern);

	let result: Vec<String> = get_words()
		.iter()
		.filter(|word| {
			if word.len() != pattern.len() {
				return false;
			}

			if !include.is_empty() && include.iter().any(|c| !word.contains(c)) {
				return false;
			}

			if !exclude.is_empty() && exclude.iter().any(|c| word.contains(c)) {
				return false;
			}

			let chars: Vec<&str> = word.split("").filter(|c| !c.is_empty()).collect();

			for (i, char) in chars.iter().enumerate() {
				let matcher = match pattern.get(i) {
					Some(op) => op,
					None => continue,
				};

				match matcher {
					MatchOperation::MatchAnything => continue,
					MatchOperation::MatchOneOf(letters) => {
						if !letters.iter().any(|l| l == char) {
							return false;
						}

						continue;
					}
					MatchOperation::ExcludeAllOf(letters) => {
						if letters.iter().any(|l| l == char) {
							return false;
						}

						continue;
					}
				}
			}

			true
		})
		.cloned()
		.collect();

	println!("{}", result.join("\n"))
}

#[derive(Debug)]
enum MatchOperation {
	MatchAnything,
	MatchOneOf(Vec<String>),
	ExcludeAllOf(Vec<String>),
}

fn parse_pattern(descriptor: &str) -> Vec<MatchOperation> {
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
