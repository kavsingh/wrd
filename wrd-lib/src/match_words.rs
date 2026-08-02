use std::sync::LazyLock;

use regex::Regex;

use crate::data::{Dictionary, get_dictionary};
use crate::util::non_empty_str;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum MatchWordsError {
	#[error("unexpected empty pattern")]
	EmptyPattern,
	#[error("invalid pattern {0}")]
	InvalidPattern(String),
	#[error("could not create matcher {pattern}: {cause}")]
	CreateMatcherFailed { pattern: String, cause: String },
}

#[derive(Clone, Debug)]
pub enum MatcherToken {
	MatchAnyChars,             // **
	MatchAnyChar,              // *
	MatchAnyCharIn(String),    // a-z
	ExcludeAllCharsIn(String), // !a-z
}

impl PartialEq for MatcherToken {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(
				Self::MatchAnyChar | Self::MatchAnyChars,
				Self::MatchAnyChar | Self::MatchAnyChars,
			) => true,

			(
				Self::MatchAnyCharIn(a) | Self::ExcludeAllCharsIn(a),
				Self::MatchAnyCharIn(b) | Self::ExcludeAllCharsIn(b),
			) => a == b,

			_ => false,
		}
	}
}

/// # Errors
/// Propagates errors from `tokenize_pattern` and `match_words_from_tokens`.
pub fn match_words<'a>(
	pattern: &str,
	include: &str,
	exclude: &str,
	within: &str,
	haystack: Option<&[&'a str]>,
) -> Result<Vec<&'a str>, MatchWordsError> {
	let tokens = tokenize_pattern(pattern)?;
	let result = match_words_from_tokens(&tokens, include, exclude, within, haystack)?;

	Ok(result)
}

const EMPTY_WORDS: Vec<&'static str> = vec![];

pub fn match_words_from_tokens<'a>(
	tokens: &[MatcherToken],
	include: &str,
	exclude: &str,
	within: &str,
	haystack: Option<&[&'a str]>,
) -> Result<Vec<&'a str>, MatchWordsError> {
	let regex = regex_from_tokens(tokens)?;
	let empty = &EMPTY_WORDS;
	let result: Vec<&str> = haystack
		.unwrap_or_else(|| get_dictionary(&Dictionary::Moby).unwrap_or(empty))
		.iter()
		.filter(|word| match_word(word, &regex, include, exclude, within))
		.copied()
		.collect();

	Ok(result)
}

fn match_word(word: &str, matcher: &Regex, include: &str, exclude: &str, within: &str) -> bool {
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

	matcher.is_match(word)
}

fn regex_from_tokens(tokens: &[MatcherToken]) -> Result<Regex, MatchWordsError> {
	let pattern = tokens
		.iter()
		.map(|token| match token {
			MatcherToken::MatchAnyChars => r"[a-z]*".to_string(),
			MatcherToken::MatchAnyChar => r"[a-z]".to_string(),
			MatcherToken::MatchAnyCharIn(chars) => format!("[{chars}]"),
			MatcherToken::ExcludeAllCharsIn(chars) => format!("([[a-z]--[{chars}]])"),
		})
		.collect::<String>();
	let bounded = format!("^{pattern}$");

	Regex::new(&bounded).map_err(|e| MatchWordsError::CreateMatcherFailed {
		pattern: bounded,
		cause: e.to_string(),
	})
}

fn tokenize_pattern(input: &str) -> Result<Vec<MatcherToken>, MatchWordsError> {
	let parts: Vec<_> = input.split(' ').filter_map(non_empty_str).collect();

	if parts.is_empty() {
		return Err(MatchWordsError::EmptyPattern);
	}

	let tokens = parts
		.iter()
		.fold(vec![], |mut acc: Vec<&str>, part| {
			if let Some(last) = acc.last()
				&& *last == "**"
				&& *part == "**"
			{
				return acc;
			}

			acc.push(part);
			acc
		})
		.iter()
		.map(|part| tokenize(part))
		.collect::<Result<Vec<_>, _>>()?;

	Ok(tokens)
}

#[allow(clippy::expect_used)]
static MATCH_CHARS_TOKEN_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^(\!)?([a-z]+)$").expect("invalid match token regex"));

fn tokenize(input: &str) -> Result<MatcherToken, MatchWordsError> {
	if input == "**" {
		return Ok(MatcherToken::MatchAnyChars);
	}

	if input == "*" {
		return Ok(MatcherToken::MatchAnyChar);
	}

	let Some(captures) = MATCH_CHARS_TOKEN_REGEX.captures(input) else {
		return Err(MatchWordsError::InvalidPattern(input.to_string()));
	};

	match (
		captures.get(1).map(|c| c.as_str()),
		captures.get(2).map(|c| c.as_str().to_owned()),
	) {
		(Some("!"), Some(letters)) => Ok(MatcherToken::ExcludeAllCharsIn(letters)),
		(None, Some(letters)) => Ok(MatcherToken::MatchAnyCharIn(letters)),
		_ => Err(MatchWordsError::InvalidPattern(input.to_string())),
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tokenize_tests {
	use super::*;

	#[test]
	#[allow(clippy::panic)]
	fn should_error_on_invalid_pattern() {
		assert_eq!(
			tokenize_pattern("").unwrap_err(),
			MatchWordsError::EmptyPattern
		);
		assert_eq!(
			tokenize_pattern("* abc !def ghi!de").unwrap_err(),
			MatchWordsError::InvalidPattern("ghi!de".to_string())
		);
		assert_eq!(
			tokenize_pattern("45 ").unwrap_err(),
			MatchWordsError::InvalidPattern("45".to_string())
		);
		assert_eq!(
			tokenize_pattern("***").unwrap_err(),
			MatchWordsError::InvalidPattern("***".to_string())
		);
		assert_eq!(
			tokenize_pattern("ABC !def").unwrap_err(),
			MatchWordsError::InvalidPattern("ABC".to_string())
		);
	}

	#[test]
	fn should_tokenize_pattern() {
		assert_eq!(
			tokenize_pattern("**").unwrap(),
			vec![MatcherToken::MatchAnyChars]
		);

		assert_eq!(
			tokenize_pattern("* ** **").unwrap(),
			vec![MatcherToken::MatchAnyChar, MatcherToken::MatchAnyChars,]
		);

		assert_eq!(
			tokenize_pattern("* a !bcd ** ** ** ef * **").unwrap(),
			vec![
				MatcherToken::MatchAnyChar,
				MatcherToken::MatchAnyCharIn("a".to_string()),
				MatcherToken::ExcludeAllCharsIn("bcd".to_string()),
				MatcherToken::MatchAnyChars,
				MatcherToken::MatchAnyCharIn("ef".to_string()),
				MatcherToken::MatchAnyChar,
				MatcherToken::MatchAnyChars,
			]
		);
	}
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod match_words_tests {
	use super::*;

	static TEST_WORDS: [&str; 12] = [
		"aaabbb", "bbbccc", "cccddd", "dddeee", "eeefff", "fffggg", "gghhii", "iijjkk", "jjkk",
		"kkll", "yenta", "yes",
	];

	#[test]
	fn should_match_all_words() {
		let tokens = vec![MatcherToken::MatchAnyChars];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS)).unwrap(),
			&TEST_WORDS
		);
	}

	#[test]
	fn should_match_all_words_respecting_globals() {
		let tokens = vec![MatcherToken::MatchAnyChars];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "gfjk", Some(&TEST_WORDS)).unwrap(),
			vec!["fffggg", "jjkk"]
		);

		assert_eq!(
			match_words_from_tokens(&tokens, "f", "", "gfjk", Some(&TEST_WORDS)).unwrap(),
			vec!["fffggg"]
		);
	}

	#[test]
	fn should_constrain_chars_match_to_tokens_length() {
		let tokens = vec![
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS)).unwrap(),
			vec!["jjkk".to_string(), "kkll".to_string()]
		);
	}

	#[test]
	fn should_match_chars_on_tokens() {
		let tokens = vec![
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyCharIn("ab".to_string()),
			MatcherToken::ExcludeAllCharsIn("cd".to_string()),
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS)).unwrap(),
			vec!["aaabbb".to_string(), "bbbccc".to_string()]
		);

		let tokens = vec![
			MatcherToken::MatchAnyCharIn("y".to_string()),
			MatcherToken::MatchAnyCharIn("e".to_string()),
			MatcherToken::MatchAnyChars,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS)).unwrap(),
			vec!["yenta".to_string(), "yes".to_string()]
		);

		let token = vec![
			MatcherToken::MatchAnyCharIn("f".to_string()),
			MatcherToken::MatchAnyCharIn("f".to_string()),
			MatcherToken::MatchAnyCharIn("f".to_string()),
			MatcherToken::MatchAnyCharIn("g".to_string()),
			MatcherToken::MatchAnyChars,
		];

		assert_eq!(
			match_words_from_tokens(&token, "", "", "", Some(&TEST_WORDS)).unwrap(),
			vec!["fffggg".to_string()]
		);
	}

	#[test]
	fn should_match_chars_on_tokens_within_globals() {
		let tokens = vec![
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "t", "", "ytanpem", Some(&TEST_WORDS)).unwrap(),
			vec!["yenta".to_string()]
		);

		let tokens = [
			MatcherToken::ExcludeAllCharsIn("ps".to_string()),
			MatcherToken::ExcludeAllCharsIn("lt".to_string()),
			MatcherToken::MatchAnyCharIn("a".to_string()),
			MatcherToken::ExcludeAllCharsIn("tl".to_string()),
			MatcherToken::ExcludeAllCharsIn("ek".to_string()),
		];
		let test_words = [
			"blast", "flats", "loath", "slant", "slats", "stalk", "stall", "trail", "trawl",
		];
		let result =
			match_words_from_tokens(&tokens, "lat", "pesk", "", Some(&test_words)).unwrap();

		assert_eq!(result, vec!["trail", "trawl"]);
	}
}
