use std::sync::LazyLock;

use fancy_regex::Regex;

use crate::match_words::{MatcherToken, match_words_from_tokens};
use crate::util::{non_empty_str, unique_string};

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum NotwordleError {
	#[error("could not get remaining words: {0}")]
	FailedToMatch(#[from] crate::match_words::MatchWordsError),
	#[error(
		"all guess results must have the same number of entries, got {current}, previous had {previous}"
	)]
	InvalidGuessResultLength { current: usize, previous: usize },
	#[error("invalid guess result entry: {0}")]
	InvalidGuessResultEntry(String),
	#[error("no characters to match in entry: {0}")]
	GuessResultEntryNeedsChar(String),
}

#[derive(Default)]
pub struct Notwordle {
	guess_results: Vec<Vec<GuessResultToken>>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GuessResultToken {
	Right(String),
	Wrong(String),
	WrongPosition(String),
}

impl Notwordle {
	/// # Errors
	/// Errors if guess result token count does not match previous entries.
	/// Propagates errors from `tokenize_guess_result`.
	pub fn register_guess_result(
		&mut self,
		result: &str,
	) -> Result<Vec<GuessResultToken>, NotwordleError> {
		let tokenized = tokenize_guess_result(result)?;

		if let Some(stored) = self.guess_results.last() {
			let stored_len = stored.len();
			let new_len = tokenized.len();

			if stored_len != new_len {
				return Err(NotwordleError::InvalidGuessResultLength {
					current: new_len,
					previous: stored_len,
				});
			}
		}

		self.guess_results.push(tokenized.clone());

		Ok(tokenized)
	}

	/// # Errors
	/// Propagates errors from `match_words_from_tokens`.
	pub fn refine(&self, words: Option<&[&'static str]>) -> Result<Vec<&str>, String> {
		let (tokens, include, exclude) = get_match_args_from_results(&self.guess_results);

		match_words_from_tokens(&tokens, &include, &exclude, "", words).map_err(|e| e.to_string())
	}
}

#[allow(clippy::expect_used)]
static GUESS_TOKEN_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^([!?])?([a-z])$").expect("invalid guess regex"));

fn tokenize_guess_result(input: &str) -> Result<Vec<GuessResultToken>, NotwordleError> {
	let entries: Vec<_> = input.split(' ').filter_map(non_empty_str).collect();
	let mut result: Vec<GuessResultToken> = vec![];

	for entry in entries {
		let Ok(Some(captures)) = GUESS_TOKEN_REGEX.captures(entry) else {
			return Err(NotwordleError::InvalidGuessResultEntry(entry.to_string()));
		};

		match (
			captures.get(1).map(|c| c.as_str()),
			captures.get(2).map(|c| c.as_str().to_owned()),
		) {
			(None, Some(c)) => result.push(GuessResultToken::Right(c)),
			(Some("!"), Some(c)) => result.push(GuessResultToken::Wrong(c)),
			(Some("?"), Some(c)) => result.push(GuessResultToken::WrongPosition(c)),
			(_, None) => {
				return Err(NotwordleError::GuessResultEntryNeedsChar(entry.to_string()));
			}
			_ => return Err(NotwordleError::InvalidGuessResultEntry(entry.to_string())),
		}
	}

	Ok(result)
}

fn get_match_args_from_results(
	guess_results: &[Vec<GuessResultToken>],
) -> (Vec<MatcherToken>, String, String) {
	let mut include = String::new();
	let mut exclude = String::new();
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
				GuessResultToken::Right(c) => MatcherToken::MatchAnyCharIn(c.clone()),
				GuessResultToken::Wrong(c) | GuessResultToken::WrongPosition(c) => {
					let candidate_op = MatcherToken::ExcludeAllCharsIn(c.clone());
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

			#[allow(clippy::indexing_slicing)]
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
#[allow(clippy::unwrap_used)]
mod tokenize_tests {
	use super::*;

	#[test]
	fn should_error_on_invalid_input() {
		assert_eq!(
			tokenize_guess_result("p ?q !r aa").unwrap_err(),
			NotwordleError::InvalidGuessResultEntry("aa".to_string())
		);
		assert_eq!(
			tokenize_guess_result("p ??q !r a").unwrap_err(),
			NotwordleError::InvalidGuessResultEntry("??q".to_string())
		);
		assert_eq!(
			tokenize_guess_result("p ?q !?r a").unwrap_err(),
			NotwordleError::InvalidGuessResultEntry("!?r".to_string())
		);
		assert_eq!(
			tokenize_guess_result("p? ?q !r a").unwrap_err(),
			NotwordleError::InvalidGuessResultEntry("p?".to_string())
		);
	}

	#[test]
	fn should_parse_guess_patterns() {
		assert_eq!(
			tokenize_guess_result("p ?l !a t !e").unwrap(),
			vec![
				GuessResultToken::Right("p".to_string()),
				GuessResultToken::WrongPosition("l".to_string()),
				GuessResultToken::Wrong("a".to_string()),
				GuessResultToken::Right("t".to_string()),
				GuessResultToken::Wrong("e".to_string()),
			]
		);
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod match_args_tests {
	use super::*;
	use crate::match_words::MatcherToken;

	#[test]
	#[allow(clippy::too_many_lines)]
	fn should_build_match_inputs_from_guesses() {
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
				MatcherToken::ExcludeAllCharsIn("pasm".to_string()),
				MatcherToken::MatchAnyCharIn("a".to_string()),
				MatcherToken::MatchAnyCharIn("t".to_string()),
				MatcherToken::ExcludeAllCharsIn("toiz".to_string()),
				MatcherToken::ExcludeAllCharsIn("erna".to_string()),
			]
		);
		assert_eq!(include, "atm".to_string());
		assert_eq!(exclude, "plecorsinz".to_string());
	}

	#[test]
	fn should_refine_words() {
		let mut nw = Notwordle::default();
		let words = ["plate", "pastor", "panda", "datum"];

		nw.register_guess_result("!p !l ?a ?t !e").unwrap();
		nw.register_guess_result("?a !c t !o !r").unwrap();
		nw.register_guess_result("!s a t !i !n").unwrap();
		nw.register_guess_result("?m a t !z !a").unwrap();

		assert_eq!(nw.refine(Some(&words)).unwrap(), vec!["datum"]);
	}
}
