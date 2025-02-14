use std::sync::OnceLock;

use regex::Regex;

use crate::{
	match_pattern::{match_words, MatchPatternToken},
	util::unique_string,
};

static GUESS_PART_REGEX: OnceLock<Regex> = OnceLock::new();

#[derive(Default)]
pub struct Notwordle {
	guess_results: Vec<Vec<GuessResultToken>>,
}

#[derive(Clone, Debug)]
pub enum GuessResultToken {
	Right(String),
	Wrong(String),
	WrongPosition(String),
}

impl PartialEq for GuessResultToken {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Right(a), Self::Right(b)) => a == b,
			(Self::Wrong(a), Self::Wrong(b)) => a == b,
			(Self::WrongPosition(a), Self::WrongPosition(b)) => a == b,
			_ => false,
		}
	}
}

impl Notwordle {
	// p ?q -r -s -t
	pub fn register_guess_result(
		&mut self,
		result: &str,
	) -> Result<(Vec<String>, Vec<GuessResultToken>), String> {
		let new_result = tokenize_guess_result(result)?;

		if let Some(stored) = self.guess_results.last() {
			let stored_len = stored.len();
			let new_len = new_result.len();

			if stored_len != new_len {
				return Err(format!(
					"previous had {} items, got {} items",
					stored_len, new_len
				));
			}
		}

		self.guess_results.push(new_result.clone());

		let (tokens, include, exclude) = get_match_args_from_results(&self.guess_results);
		let matches = match_words(&tokens, &include, &exclude, "", None);

		Ok((matches, new_result))
	}
}

fn tokenize_guess_result(input: &str) -> Result<Vec<GuessResultToken>, String> {
	let regex = GUESS_PART_REGEX
		.get_or_init(|| Regex::new(r"^([!?]{1})?([a-z]{1})$").expect("invalid guess regex"));

	let entries: Vec<&str> = input
		.split(" ")
		.filter_map(|p| {
			let trimmed = p.trim();

			if trimmed.is_empty() {
				None
			} else {
				Some(trimmed)
			}
		})
		.collect();

	let mut result: Vec<GuessResultToken> = vec![];

	for entry in entries.iter() {
		let captures = match regex.captures(entry) {
			Some(cap) => cap,
			None => return Err(format!("invalid pattern {}", entry)),
		};

		let char = match captures.get(2) {
			Some(c) => c.as_str(),
			None => return Err(format!("no value in pattern {}", entry)),
		};

		let op = captures.get(1).map(|m| m.as_str()).unwrap_or("");

		match op {
			"!" => result.push(GuessResultToken::Wrong(char.to_string())),
			"?" => result.push(GuessResultToken::WrongPosition(char.to_string())),
			_ => result.push(GuessResultToken::Right(char.to_string())),
		}
	}

	Ok(result)
}

fn get_match_args_from_results(
	guess_results: &[Vec<GuessResultToken>],
) -> (Vec<MatchPatternToken>, String, String) {
	let mut include = "".to_string();
	let mut exclude = "".to_string();
	let mut match_tokens: Vec<MatchPatternToken> = vec![];

	for result in guess_results {
		for (i, result_char) in result.iter().enumerate() {
			match result_char {
				GuessResultToken::Right(c) | GuessResultToken::WrongPosition(c) => {
					include.push_str(c);
				}
				GuessResultToken::Wrong(c) => {
					exclude.push_str(c);
				}
			}

			let resolved_op = match result_char {
				GuessResultToken::Right(c) => MatchPatternToken::MatchAnyIn(c.to_string()),
				GuessResultToken::WrongPosition(c) | GuessResultToken::Wrong(c) => {
					let candidate_op = MatchPatternToken::ExcludeAllIn(c.to_string());
					let current_op = match_tokens.get(i);

					match (&candidate_op, current_op) {
						(
							MatchPatternToken::ExcludeAllIn(a),
							Some(MatchPatternToken::ExcludeAllIn(b)),
						) => {
							let mut joined = b.to_owned();

							joined.push_str(&a.clone());
							MatchPatternToken::ExcludeAllIn(unique_string(&joined))
						}
						_ => candidate_op,
					}
				}
			};

			if i < match_tokens.len() {
				match_tokens[i] = resolved_op;
			} else {
				match_tokens.push(resolved_op);
			}
		}
	}

	(
		match_tokens,
		unique_string(&include),
		unique_string(&exclude),
	)
}

#[cfg(test)]
mod tests {
	use crate::match_pattern::MatchPatternToken;

	use super::*;

	#[test]
	fn should_error_on_incorrect_pattern() {
		let result = match tokenize_guess_result("p ?q !r aa") {
			Ok(_) => panic!("should not pass"),
			Err(message) => message,
		};

		assert_eq!(result, "invalid pattern aa");
	}

	#[test]
	fn should_parse_guess_patterns() -> Result<(), String> {
		let result = tokenize_guess_result("p ?l !a t !e")?;

		assert_eq!(
			result,
			vec![
				GuessResultToken::Right("p".to_string()),
				GuessResultToken::WrongPosition("l".to_string()),
				GuessResultToken::Wrong("a".to_string()),
				GuessResultToken::Right("t".to_string()),
				GuessResultToken::Wrong("e".to_string()),
			]
		);

		Ok(())
	}

	#[test]
	fn should_build_matcher_pattern_from_guesses() -> Result<(), String> {
		// word is pilot

		// plate
		let first_guess = vec![
			GuessResultToken::Right("p".to_string()),
			GuessResultToken::WrongPosition("l".to_string()),
			GuessResultToken::Wrong("a".to_string()),
			GuessResultToken::WrongPosition("t".to_string()),
			GuessResultToken::Wrong("e".to_string()),
		];

		let guesses = vec![first_guess.clone()];
		let (pattern, include, exclude) = get_match_args_from_results(&guesses);

		assert_eq!(include, "plt".to_string());
		assert_eq!(exclude, "ae".to_string());
		assert_eq!(
			pattern,
			vec![
				MatchPatternToken::MatchAnyIn("p".to_string()),
				MatchPatternToken::ExcludeAllIn("l".to_string()),
				MatchPatternToken::ExcludeAllIn("a".to_string()),
				MatchPatternToken::ExcludeAllIn("t".to_string()),
				MatchPatternToken::ExcludeAllIn("e".to_string()),
			]
		);

		// polit (whatever)
		let second_guess = vec![
			GuessResultToken::Right("p".to_string()),
			GuessResultToken::WrongPosition("o".to_string()),
			GuessResultToken::Right("l".to_string()),
			GuessResultToken::WrongPosition("i".to_string()),
			GuessResultToken::Right("t".to_string()),
		];

		let guesses = vec![first_guess, second_guess];
		let (pattern, include, exclude) = get_match_args_from_results(&guesses);

		assert_eq!(include, "pltoi".to_string());
		assert_eq!(exclude, "ae".to_string());
		assert_eq!(
			pattern,
			vec![
				MatchPatternToken::MatchAnyIn("p".to_string()),
				MatchPatternToken::ExcludeAllIn("lo".to_string()),
				MatchPatternToken::MatchAnyIn("l".to_string()),
				MatchPatternToken::ExcludeAllIn("ti".to_string()),
				MatchPatternToken::MatchAnyIn("t".to_string()),
			]
		);

		Ok(())
	}
}
