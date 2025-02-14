use std::sync::OnceLock;

use regex::Regex;

use crate::{
	matcher::{match_from_pattern, MatchOperation, MatchPattern},
	util::unique_string,
};

static GUESS_PART_REGEX: OnceLock<Regex> = OnceLock::new();

#[derive(Default)]
pub struct NotWordle {
	guess_results: Vec<GuessResult>,
}

pub type GuessResult = Vec<GuessResultChar>;

#[derive(Clone, Debug)]
pub enum GuessResultChar {
	Right(String),
	Wrong(String),
	WrongPosition(String),
}

impl PartialEq for GuessResultChar {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::Right(a), Self::Right(b)) => a == b,
			(Self::Wrong(a), Self::Wrong(b)) => a == b,
			(Self::WrongPosition(a), Self::WrongPosition(b)) => a == b,
			_ => false,
		}
	}
}

impl NotWordle {
	// p ?q -r -s -t
	pub fn register_guess_result(
		&mut self,
		result_pattern: &str,
	) -> Result<(Vec<String>, GuessResult), String> {
		let guess_result = parse_result_pattern(result_pattern)?;

		self.guess_results.push(guess_result.clone());

		let (pattern, include, exclude) = get_match_args_from_results(&self.guess_results);
		let matches = match_from_pattern(&pattern, &include, &exclude);

		Ok((matches, guess_result))
	}
}

fn parse_result_pattern(result_pattern: &str) -> Result<GuessResult, String> {
	let regex = GUESS_PART_REGEX
		.get_or_init(|| Regex::new(r"^([!?]{1})?([a-z]{1})$").expect("invalid guess regex"));

	let patterns: Vec<&str> = result_pattern
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

	if patterns.len() != 5 {
		return Err("expected 5 guess entries".into());
	}

	let mut result: GuessResult = vec![];

	for patt in patterns.iter() {
		let captures = match regex.captures(patt) {
			Some(cap) => cap,
			None => return Err(format!("invalid pattern {}", patt)),
		};

		let char = match captures.get(2) {
			Some(c) => c.as_str(),
			None => return Err(format!("no value in pattern {}", patt)),
		};

		let op = captures.get(1).map(|m| m.as_str()).unwrap_or("");

		match op {
			"!" => result.push(GuessResultChar::Wrong(char.to_string())),
			"?" => result.push(GuessResultChar::WrongPosition(char.to_string())),
			_ => result.push(GuessResultChar::Right(char.to_string())),
		}
	}

	Ok(result)
}

fn get_match_args_from_results(guess_results: &[GuessResult]) -> (MatchPattern, String, String) {
	let mut include = "".to_string();
	let mut exclude = "".to_string();
	let mut match_pattern: MatchPattern = vec![];

	for result in guess_results {
		for (i, result_char) in result.iter().enumerate() {
			match result_char {
				GuessResultChar::Right(c) | GuessResultChar::WrongPosition(c) => {
					include.push_str(c);
				}
				GuessResultChar::Wrong(c) => {
					exclude.push_str(c);
				}
			}

			let resolved_op = match result_char {
				GuessResultChar::Right(c) => MatchOperation::MatchAnyIn(c.to_string()),
				GuessResultChar::WrongPosition(c) | GuessResultChar::Wrong(c) => {
					let candidate_op = MatchOperation::ExcludeAllIn(c.to_string());
					let current_op = match_pattern.get(i);

					match (&candidate_op, current_op) {
						(
							MatchOperation::ExcludeAllIn(a),
							Some(MatchOperation::ExcludeAllIn(b)),
						) => {
							let mut joined = b.to_owned();

							joined.push_str(&a.clone());
							MatchOperation::ExcludeAllIn(unique_string(&joined))
						}
						_ => candidate_op,
					}
				}
			};

			if i < match_pattern.len() {
				match_pattern[i] = resolved_op;
			} else {
				match_pattern.push(resolved_op);
			}
		}
	}

	(
		match_pattern,
		unique_string(&include),
		unique_string(&exclude),
	)
}

#[cfg(test)]
mod tests {
	use crate::matcher::MatchOperation;

	use super::*;

	#[test]
	fn should_error_if_incorrect_guess_parts() {
		let four = match parse_result_pattern("p q r s") {
			Ok(_) => panic!("should not pass"),
			Err(message) => message,
		};

		let six = match parse_result_pattern("p q r s") {
			Ok(_) => panic!("should not pass"),
			Err(message) => message,
		};

		assert_eq!(four, "expected 5 guess entries");
		assert_eq!(six, "expected 5 guess entries");
	}

	#[test]
	fn should_error_on_incorrect_pattern() {
		let result = match parse_result_pattern("p ?q !r s aa") {
			Ok(_) => panic!("should not pass"),
			Err(message) => message,
		};

		assert_eq!(result, "invalid pattern aa");
	}

	#[test]
	fn should_parse_guess_patterns() -> Result<(), String> {
		let result = parse_result_pattern("p ?l !a t !e")?;

		assert_eq!(
			result,
			vec![
				GuessResultChar::Right("p".to_string()),
				GuessResultChar::WrongPosition("l".to_string()),
				GuessResultChar::Wrong("a".to_string()),
				GuessResultChar::Right("t".to_string()),
				GuessResultChar::Wrong("e".to_string()),
			]
		);

		Ok(())
	}

	#[test]
	fn should_build_matcher_pattern_from_guesses() -> Result<(), String> {
		// word is pilot

		// plate
		let first_guess: GuessResult = vec![
			GuessResultChar::Right("p".to_string()),
			GuessResultChar::WrongPosition("l".to_string()),
			GuessResultChar::Wrong("a".to_string()),
			GuessResultChar::WrongPosition("t".to_string()),
			GuessResultChar::Wrong("e".to_string()),
		];

		let guesses = vec![first_guess.clone()];
		let (pattern, include, exclude) = get_match_args_from_results(&guesses);

		assert_eq!(include, "plt".to_string());
		assert_eq!(exclude, "ae".to_string());
		assert_eq!(
			pattern,
			vec![
				MatchOperation::MatchAnyIn("p".to_string()),
				MatchOperation::ExcludeAllIn("l".to_string()),
				MatchOperation::ExcludeAllIn("a".to_string()),
				MatchOperation::ExcludeAllIn("t".to_string()),
				MatchOperation::ExcludeAllIn("e".to_string()),
			]
		);

		// polit (whatever)
		let second_guess = vec![
			GuessResultChar::Right("p".to_string()),
			GuessResultChar::WrongPosition("o".to_string()),
			GuessResultChar::Right("l".to_string()),
			GuessResultChar::WrongPosition("i".to_string()),
			GuessResultChar::Right("t".to_string()),
		];

		let guesses: Vec<GuessResult> = vec![first_guess, second_guess];
		let (pattern, include, exclude) = get_match_args_from_results(&guesses);

		assert_eq!(include, "pltoi".to_string());
		assert_eq!(exclude, "ae".to_string());
		assert_eq!(
			pattern,
			vec![
				MatchOperation::MatchAnyIn("p".to_string()),
				MatchOperation::ExcludeAllIn("lo".to_string()),
				MatchOperation::MatchAnyIn("l".to_string()),
				MatchOperation::ExcludeAllIn("ti".to_string()),
				MatchOperation::MatchAnyIn("t".to_string()),
			]
		);

		Ok(())
	}
}
