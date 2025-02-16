use std::sync::LazyLock;

use regex_lite::Regex;

use crate::{
	match_words::{match_words_from_tokenized, MatchCharsToken, MatcherToken},
	util::unique_string,
};

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
	) -> Result<(Vec<&str>, Vec<GuessResultToken>), String> {
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
		let matches = match_words_from_tokenized(&tokens, &include, &exclude, "", None);

		Ok((matches, new_result))
	}
}

static GUESS_TOKEN_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^([!?]{1})?([a-z]{1})$").expect("invalid guess regex"));

fn tokenize_guess_result(input: &str) -> Result<Vec<GuessResultToken>, String> {
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

	let regex = &GUESS_TOKEN_REGEX;
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
) -> (MatcherToken, String, String) {
	let mut include = "".to_string();
	let mut exclude = "".to_string();
	let mut match_tokens: Vec<MatchCharsToken> = vec![];

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
				GuessResultToken::Right(c) => MatchCharsToken::MatchAnyCharIn(c.to_string()),
				GuessResultToken::Wrong(c) | GuessResultToken::WrongPosition(c) => {
					let candidate_op = MatchCharsToken::ExcludeAllCharsIn(c.to_string());
					let current_op = match_tokens.get(i);

					match (&candidate_op, current_op) {
						(
							MatchCharsToken::ExcludeAllCharsIn(a),
							Some(MatchCharsToken::ExcludeAllCharsIn(b)),
						) => {
							let mut acc = b.to_owned();

							acc.push_str(&a.clone());
							MatchCharsToken::ExcludeAllCharsIn(unique_string(&acc))
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
		MatcherToken::MatchOnChars(match_tokens),
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
			MatcherToken::MatchOnChars(vec![
				MatchCharsToken::MatchAnyCharIn("p".to_string()),
				MatchCharsToken::ExcludeAllCharsIn("l".to_string()),
				MatchCharsToken::ExcludeAllCharsIn("a".to_string()),
				MatchCharsToken::ExcludeAllCharsIn("t".to_string()),
				MatchCharsToken::ExcludeAllCharsIn("e".to_string()),
			])
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
			MatcherToken::MatchOnChars(vec![
				MatchCharsToken::MatchAnyCharIn("p".to_string()),
				MatchCharsToken::ExcludeAllCharsIn("lo".to_string()),
				MatchCharsToken::MatchAnyCharIn("l".to_string()),
				MatchCharsToken::ExcludeAllCharsIn("ti".to_string()),
				MatchCharsToken::MatchAnyCharIn("t".to_string()),
			])
		);

		Ok(())
	}
}
