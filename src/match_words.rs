use std::sync::LazyLock;

use regex_lite::Regex;

use crate::data::WORDS;

#[derive(Clone, Debug)]
pub enum MatcherToken {
	MatchAny,
	MatchAnyIn(String),
	ExcludeAllIn(String),
}

impl PartialEq for MatcherToken {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::MatchAny, Self::MatchAny) => true,
			(Self::MatchAnyIn(a), Self::MatchAnyIn(b)) => a == b,
			(Self::ExcludeAllIn(a), Self::ExcludeAllIn(b)) => a == b,
			_ => false,
		}
	}
}

pub fn match_words<'a>(
	pattern: &str,
	include: &str,
	exclude: &str,
	within: &str,
	words: Option<&[&'a str]>,
) -> Result<Vec<&'a str>, String> {
	let tokens = tokenize_pattern(pattern)?;
	let result = match_words_from_tokens(&tokens, include, exclude, within, words);

	Ok(result)
}

pub fn match_words_from_tokens<'a>(
	tokens: &[MatcherToken],
	include: &str,
	exclude: &str,
	within: &str,
	words: Option<&[&'a str]>,
) -> Vec<&'a str> {
	words
		.unwrap_or_else(|| &WORDS)
		.iter()
		.filter(|word| match_word(word, tokens, include, exclude, within))
		.cloned()
		.collect()
}

fn match_word(
	word: &str,
	tokens: &[MatcherToken],
	include: &str,
	exclude: &str,
	within: &str,
) -> bool {
	if word.len() != tokens.len() {
		return false;
	}

	// word can only contain letters within this group
	if !within.is_empty() && word.chars().any(|c| !within.contains(c)) {
		return false;
	}

	// word must include all of these letters
	if !include.is_empty() && include.chars().any(|c| !word.contains(c)) {
		return false;
	}

	// word must not include any of these letters
	if !exclude.is_empty() && exclude.chars().any(|c| word.contains(c)) {
		return false;
	}

	for (i, char) in word.chars().enumerate() {
		match tokens.get(i) {
			Some(MatcherToken::MatchAny) => continue,
			Some(MatcherToken::MatchAnyIn(chars)) => {
				if !chars.chars().any(|l| l == char) {
					return false;
				}
			}
			Some(MatcherToken::ExcludeAllIn(chars)) => {
				if chars.chars().any(|l| l == char) {
					return false;
				}
			}
			None => continue,
		}
	}

	true
}

fn tokenize_pattern(input: &str) -> Result<Vec<MatcherToken>, String> {
	input
		.split(" ")
		.map(tokenize)
		.collect::<Result<Vec<_>, _>>()
}

static MATCH_TOKEN_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^(\!)?([a-z]+)$").expect("invalid match token regex"));

fn tokenize(input: &str) -> Result<MatcherToken, String> {
	if input == "*" {
		return Ok(MatcherToken::MatchAny);
	}

	let regex = &MATCH_TOKEN_REGEX;
	let captures = match regex.captures(input) {
		Some(c) => c,
		None => return Err(format!("invalid input {input}")),
	};

	match (
		captures.get(1).map(|c| c.as_str()),
		captures.get(2).map(|c| c.as_str()),
	) {
		(Some("!"), Some(letters)) => Ok(MatcherToken::ExcludeAllIn(letters.to_owned())),
		(None, Some(letters)) => Ok(MatcherToken::MatchAnyIn(letters.to_owned())),
		_ => Err(format!("invalid input {input}")),
	}
}

#[cfg(test)]
mod tokenize_tests {
	use super::*;

	#[test]
	fn should_error_on_invalid_pattern() {
		let message = match tokenize_pattern("* abc !def ghi!de") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid input ghi!de");

		let message = match tokenize_pattern(" a") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid input ");

		let message = match tokenize_pattern("45 ") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid input 45");

		let message = match tokenize_pattern("** ABC !def") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid input **");

		let message = match tokenize_pattern("ABC !def") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid input ABC");
	}

	#[test]
	fn should_tokenize_pattern() -> Result<(), String> {
		assert_eq!(
			tokenize_pattern("* a !bcd ef *")?,
			vec![
				MatcherToken::MatchAny,
				MatcherToken::MatchAnyIn("a".to_string()),
				MatcherToken::ExcludeAllIn("bcd".to_string()),
				MatcherToken::MatchAnyIn("ef".to_string()),
				MatcherToken::MatchAny,
			]
		);

		Ok(())
	}
}

#[cfg(test)]
mod match_words_tests {
	use super::*;

	static TEST_WORDS: [&str; 11] = [
		"aaabbb", "bbbccc", "cccddd", "dddeee", "eeefff", "fffggg", "gghhii", "iijjkk", "jjkk",
		"kkll", "yenta",
	];

	#[test]
	fn should_constrain_to_tokens_length() {
		let tokens = vec![
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS)),
			vec!["jjkk".to_string(), "kkll".to_string()]
		);
	}

	#[test]
	fn should_match_on_tokens() {
		let tokens = vec![
			MatcherToken::MatchAny,
			MatcherToken::MatchAnyIn("ab".to_string()),
			MatcherToken::ExcludeAllIn("cd".to_string()),
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS)),
			vec!["aaabbb".to_string(), "bbbccc".to_string()]
		);
	}

	#[test]
	fn should_match_on_tokens_within_pool() {
		let tokens = vec![
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "t", "", "ytanpem", Some(&TEST_WORDS)),
			vec!["yenta".to_string()]
		);
	}
}
