use std::sync::OnceLock;

use regex_lite::Regex;

use crate::{
	match_words::{match_words, MatcherToken},
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
			None => return Err(format!("invalid input {}", entry)),
		};

		let char = match captures.get(2) {
			Some(c) => c.as_str(),
			None => return Err(format!("no values in input {}", entry)),
		};

		let modifier = captures.get(1).map(|m| m.as_str()).unwrap_or("");
		let char_string = char.to_string();

		match modifier {
			"" => result.push(GuessResultToken::Right(char_string)),
			"!" => result.push(GuessResultToken::Wrong(char_string)),
			"?" => result.push(GuessResultToken::WrongPosition(char_string)),
			_ => return Err(format!("invalid modifier {}", modifier)),
		}
	}

	Ok(result)
}

fn get_match_args_from_results(
	guess_results: &[Vec<GuessResultToken>],
) -> (Vec<MatcherToken>, String, String) {
	let mut include = "".to_string();
	let mut exclude = "".to_string();
	let mut match_tokens: Vec<MatcherToken> = vec![];

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
				GuessResultToken::Right(c) => MatcherToken::MatchAnyIn(c.to_string()),
				GuessResultToken::WrongPosition(c) | GuessResultToken::Wrong(c) => {
					let candidate_op = MatcherToken::ExcludeAllIn(c.to_string());
					let current_op = match_tokens.get(i);

					match (&candidate_op, current_op) {
						(MatcherToken::ExcludeAllIn(a), Some(MatcherToken::ExcludeAllIn(b))) => {
							let mut joined = b.to_owned();

							joined.push_str(&a.clone());
							MatcherToken::ExcludeAllIn(unique_string(&joined))
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
mod tokenize_tests {
	use super::*;

	#[test]
	fn should_error_on_invalid_input() {
		let result = match tokenize_guess_result("p ?q !r aa") {
			Ok(_) => panic!("should not pass"),
			Err(message) => message,
		};

		assert_eq!(result, "invalid input aa");
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
}

#[cfg(test)]
mod match_args_tests {
	use crate::match_words::MatcherToken;

	use super::*;

	#[test]
	fn should_build_match_inputs_from_guesses() -> Result<(), String> {
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
				MatcherToken::MatchAnyIn("p".to_string()),
				MatcherToken::ExcludeAllIn("l".to_string()),
				MatcherToken::ExcludeAllIn("a".to_string()),
				MatcherToken::ExcludeAllIn("t".to_string()),
				MatcherToken::ExcludeAllIn("e".to_string()),
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
				MatcherToken::MatchAnyIn("p".to_string()),
				MatcherToken::ExcludeAllIn("lo".to_string()),
				MatcherToken::MatchAnyIn("l".to_string()),
				MatcherToken::ExcludeAllIn("ti".to_string()),
				MatcherToken::MatchAnyIn("t".to_string()),
			]
		);

		Ok(())
	}
}
