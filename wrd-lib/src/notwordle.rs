use std::sync::LazyLock;

use fancy_regex::Regex;

use crate::match_words::{MatcherToken, match_words_from_tokens};
use crate::util::{non_empty_str, unique_string};

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
		words: Option<&[&'static str]>,
	) -> Result<(Vec<&str>, Vec<GuessResultToken>), String> {
		let new_result = tokenize_guess_result(result)?;

		if let Some(stored) = self.guess_results.last() {
			let stored_len = stored.len();
			let new_len = new_result.len();

			if stored_len != new_len {
				return Err(format!(
					"previous had {stored_len} items, got {new_len} items"
				));
			}
		}

		self.guess_results.push(new_result.clone());

		let (tokens, include, exclude) = get_match_args_from_results(&self.guess_results);
		let matches = match_words_from_tokens(&tokens, &include, &exclude, "", words)?;

		Ok((matches, new_result))
	}
}

static GUESS_TOKEN_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^([!?])?([a-z])$").expect("invalid guess regex"));

fn tokenize_guess_result(input: &str) -> Result<Vec<GuessResultToken>, String> {
	let entries: Vec<_> = input.split(" ").filter_map(non_empty_str).collect();
	let mut result: Vec<GuessResultToken> = vec![];

	for entry in entries.iter() {
		let captures = match GUESS_TOKEN_REGEX.captures(entry) {
			Ok(Some(cap)) => cap,
			_ => return Err(format!("invalid input {entry}")),
		};

		match (
			captures.get(1).map(|c| c.as_str()),
			captures.get(2).map(|c| c.as_str().to_owned()),
		) {
			(_, None) => return Err(format!("no values in input {entry}")),
			(None, Some(c)) => result.push(GuessResultToken::Right(c)),
			(Some("!"), Some(c)) => result.push(GuessResultToken::Wrong(c)),
			(Some("?"), Some(c)) => result.push(GuessResultToken::WrongPosition(c)),
			_ => return Err(format!("invalid input {entry}")),
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
					if !include.contains(c) {
						exclude.push_str(c);
					}
				}
			}

			let resolved_op = match result_char {
				GuessResultToken::Right(c) => MatcherToken::MatchAnyCharIn(c.to_string()),
				GuessResultToken::Wrong(c) | GuessResultToken::WrongPosition(c) => {
					let candidate_op = MatcherToken::ExcludeAllCharsIn(c.to_string());
					let current_op = match_tokens.get(i);

					match (&candidate_op, current_op) {
						(
							MatcherToken::ExcludeAllCharsIn(a),
							Some(MatcherToken::ExcludeAllCharsIn(b)),
						) => {
							let mut acc = b.to_owned();

							acc.push_str(&a.clone());
							MatcherToken::ExcludeAllCharsIn(unique_string(&acc))
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

		let result = match tokenize_guess_result("p ??q !r a") {
			Ok(_) => panic!("should not pass"),
			Err(message) => message,
		};

		assert_eq!(result, "invalid input ??q");

		let result = match tokenize_guess_result("p ?q !?r a") {
			Ok(_) => panic!("should not pass"),
			Err(message) => message,
		};

		assert_eq!(result, "invalid input !?r");

		let result = match tokenize_guess_result("p? ?q !r a") {
			Ok(_) => panic!("should not pass"),
			Err(message) => message,
		};

		assert_eq!(result, "invalid input p?");
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
	use super::*;
	use crate::match_words::MatcherToken;

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
				MatcherToken::MatchAnyCharIn("p".to_string()),
				MatcherToken::ExcludeAllCharsIn("l".to_string()),
				MatcherToken::ExcludeAllCharsIn("a".to_string()),
				MatcherToken::ExcludeAllCharsIn("t".to_string()),
				MatcherToken::ExcludeAllCharsIn("e".to_string()),
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
				MatcherToken::MatchAnyCharIn("p".to_string()),
				MatcherToken::ExcludeAllCharsIn("lo".to_string()),
				MatcherToken::MatchAnyCharIn("l".to_string()),
				MatcherToken::ExcludeAllCharsIn("ti".to_string()),
				MatcherToken::MatchAnyCharIn("t".to_string()),
			]
		);

		//

		let guesses = [
			// !p ?l a ?t !e
			vec![
				GuessResultToken::Wrong("p".to_string()),
				GuessResultToken::WrongPosition("l".to_string()),
				GuessResultToken::Right("a".to_string()),
				GuessResultToken::WrongPosition("t".to_string()),
				GuessResultToken::Wrong("e".to_string()),
			],
			// !s ?t a ?l !k'
			vec![
				GuessResultToken::Wrong("s".to_string()),
				GuessResultToken::WrongPosition("t".to_string()),
				GuessResultToken::Right("a".to_string()),
				GuessResultToken::WrongPosition("l".to_string()),
				GuessResultToken::Wrong("k".to_string()),
			],
		];
		let (pattern, include, exclude) = get_match_args_from_results(&guesses);

		assert_eq!(
			pattern,
			vec![
				MatcherToken::ExcludeAllCharsIn("ps".to_string()),
				MatcherToken::ExcludeAllCharsIn("lt".to_string()),
				MatcherToken::MatchAnyCharIn("a".to_string()),
				MatcherToken::ExcludeAllCharsIn("tl".to_string()),
				MatcherToken::ExcludeAllCharsIn("ek".to_string()),
			]
		);
		assert_eq!(include, "lat".to_string());
		assert_eq!(exclude, "pesk".to_string());

		//

		let guesses = [
			// !p !l ?a ?t !e
			vec![
				GuessResultToken::Wrong("p".to_string()),
				GuessResultToken::Wrong("l".to_string()),
				GuessResultToken::WrongPosition("a".to_string()),
				GuessResultToken::WrongPosition("t".to_string()),
				GuessResultToken::Wrong("e".to_string()),
			],
			// ?a !c t !o !r
			vec![
				GuessResultToken::WrongPosition("a".to_string()),
				GuessResultToken::Wrong("c".to_string()),
				GuessResultToken::Right("t".to_string()),
				GuessResultToken::Wrong("o".to_string()),
				GuessResultToken::Wrong("r".to_string()),
			],
			// !s a t !i !n
			vec![
				GuessResultToken::Wrong("s".to_string()),
				GuessResultToken::Right("a".to_string()),
				GuessResultToken::Right("t".to_string()),
				GuessResultToken::Wrong("i".to_string()),
				GuessResultToken::Wrong("n".to_string()),
			],
			// ?m a t !z !a
			vec![
				GuessResultToken::WrongPosition("m".to_string()),
				GuessResultToken::Right("a".to_string()),
				GuessResultToken::Right("t".to_string()),
				GuessResultToken::Wrong("z".to_string()),
				GuessResultToken::Wrong("a".to_string()),
			],
		];
		let (pattern, include, exclude) = get_match_args_from_results(&guesses);

		assert_eq!(
			pattern,
			vec![
				MatcherToken::ExcludeAllCharsIn("ps".to_string()),
				MatcherToken::ExcludeAllCharsIn("lt".to_string()),
				MatcherToken::MatchAnyCharIn("a".to_string()),
				MatcherToken::ExcludeAllCharsIn("tl".to_string()),
				MatcherToken::ExcludeAllCharsIn("ek".to_string()),
			]
		);
		assert_eq!(include, "atm".to_string());
		assert_eq!(exclude, "plecorsinz".to_string());

		Ok(())
	}

	#[test]
	fn should_refine_words() -> Result<(), String> {
		let mut nw = Notwordle::default();
		let words = ["plate", "pastor", "panda", "datum"];
		let guesses = [
			"!p !l ?a ?t !e",
			"?a !c t !o !r",
			"!s a t !i !n",
			"?m a t !z !a",
		];

		let mut result: Vec<&str> = vec![];
		let expected = "datum";

		for guess in guesses {
			let (items, _) = nw.register_guess_result(guess, Some(&words)).unwrap();

			result = items
		}

		assert_eq!(result, vec![expected]);

		Ok(())
	}
}
