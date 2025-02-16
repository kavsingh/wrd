use std::sync::LazyLock;

use regex_lite::Regex;

use crate::data::WORDS;

#[derive(Clone, Debug)]
pub enum MatchCharsToken {
	MatchAnyChar,
	MatchAnyCharIn(String),
	ExcludeAllCharsIn(String),
}

impl PartialEq for MatchCharsToken {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::MatchAnyChar, Self::MatchAnyChar) => true,
			(Self::MatchAnyCharIn(a), Self::MatchAnyCharIn(b)) => a == b,
			(Self::ExcludeAllCharsIn(a), Self::ExcludeAllCharsIn(b)) => a == b,
			_ => false,
		}
	}
}

#[derive(Clone, Debug)]
pub enum MatcherToken {
	MatchAnyWord,
	MatchOnChars((Vec<MatchCharsToken>, Option<usize>)),
}

impl PartialEq for MatcherToken {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::MatchAnyWord, Self::MatchAnyWord) => true,
			(Self::MatchOnChars(a), Self::MatchOnChars(b)) => a == b,
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
	let token = tokenize_pattern(pattern)?;
	let result = match_words_from_tokenized(&token, include, exclude, within, words);

	Ok(result)
}

pub fn match_words_from_tokenized<'a>(
	token: &MatcherToken,
	include: &str,
	exclude: &str,
	within: &str,
	words: Option<&[&'a str]>,
) -> Vec<&'a str> {
	words
		.unwrap_or_else(|| &WORDS)
		.iter()
		.filter(|word| match_word(word, token, include, exclude, within))
		.cloned()
		.collect()
}

fn match_word(
	word: &str,
	token: &MatcherToken,
	include: &str,
	exclude: &str,
	within: &str,
) -> bool {
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

	match token {
		MatcherToken::MatchAnyWord => true,
		MatcherToken::MatchOnChars((tokens, end)) => match_word_chars(word, tokens, end),
	}
}

fn match_word_chars(word: &str, tokens: &[MatchCharsToken], end: &Option<usize>) -> bool {
	let matchable = match end {
		Some(index) => {
			if *index < word.len() {
				&word[..*index]
			} else {
				""
			}
		}
		None => word,
	};

	if matchable.len() != tokens.len() {
		return false;
	}

	for (i, char) in matchable.chars().enumerate() {
		match tokens.get(i) {
			None | Some(MatchCharsToken::MatchAnyChar) => continue,
			Some(MatchCharsToken::MatchAnyCharIn(chars)) => {
				if !chars.chars().any(|l| l == char) {
					return false;
				}
			}
			Some(MatchCharsToken::ExcludeAllCharsIn(chars)) => {
				if chars.chars().any(|l| l == char) {
					return false;
				}
			}
		}
	}

	true
}

fn tokenize_pattern(input: &str) -> Result<MatcherToken, String> {
	if input == "**" {
		return Ok(MatcherToken::MatchAnyWord);
	}

	let parts = input
		.split(" ")
		.filter_map(|line| {
			let trimmed = line.trim();

			if trimmed.is_empty() {
				None
			} else {
				Some(trimmed)
			}
		})
		.collect::<Vec<_>>();

	if parts.is_empty() {
		return Err("invalid empty input".to_string());
	}

	let end = match parts.last() {
		Some(&"**") => Some(parts.len() - 1),
		None | Some(_) => None,
	};
	let tokenizable = match end {
		Some(num) => &parts[..num],
		None => &parts[..],
	};

	let chars_tokens = tokenizable
		.iter()
		.map(|part| tokenize(part))
		.collect::<Result<Vec<_>, _>>()?;

	Ok(MatcherToken::MatchOnChars((chars_tokens, end)))
}

static MATCH_CHARS_TOKEN_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^(\!)?([a-z]+)$").expect("invalid match token regex"));

fn tokenize(input: &str) -> Result<MatchCharsToken, String> {
	if input == "*" {
		return Ok(MatchCharsToken::MatchAnyChar);
	}

	let captures = match MATCH_CHARS_TOKEN_REGEX.captures(input) {
		Some(c) => c,
		None => return Err(format!("invalid input {input}")),
	};

	match (
		captures.get(1).map(|c| c.as_str()),
		captures.get(2).map(|c| c.as_str().to_owned()),
	) {
		(Some("!"), Some(letters)) => Ok(MatchCharsToken::ExcludeAllCharsIn(letters)),
		(None, Some(letters)) => Ok(MatchCharsToken::MatchAnyCharIn(letters)),
		_ => Err(format!("invalid input {input}")),
	}
}

#[cfg(test)]
mod tokenize_tests {
	use super::*;

	#[test]
	fn should_error_on_invalid_pattern() {
		let message = match tokenize_pattern("") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid empty input");

		let message = match tokenize_pattern("** y") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid input **");

		let message = match tokenize_pattern("* abc !def ghi!de") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid input ghi!de");

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

		let message = match tokenize_pattern("***") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid input ***");

		let message = match tokenize_pattern("ABC !def") {
			Ok(_) => panic!("should not pass"),
			Err(e) => e,
		};

		assert_eq!(message, "invalid input ABC");
	}

	#[test]
	fn should_tokenize_pattern() -> Result<(), String> {
		assert_eq!(tokenize_pattern("**")?, MatcherToken::MatchAnyWord);

		assert_eq!(
			tokenize_pattern("*")?,
			MatcherToken::MatchOnChars((vec![MatchCharsToken::MatchAnyChar], None))
		);

		assert_eq!(
			tokenize_pattern("* a !bcd ef *")?,
			MatcherToken::MatchOnChars((
				vec![
					MatchCharsToken::MatchAnyChar,
					MatchCharsToken::MatchAnyCharIn("a".to_string()),
					MatchCharsToken::ExcludeAllCharsIn("bcd".to_string()),
					MatchCharsToken::MatchAnyCharIn("ef".to_string()),
					MatchCharsToken::MatchAnyChar,
				],
				None
			))
		);

		assert_eq!(
			tokenize_pattern("y e **")?,
			MatcherToken::MatchOnChars((
				vec![
					MatchCharsToken::MatchAnyCharIn("y".to_string()),
					MatchCharsToken::MatchAnyCharIn("e".to_string()),
				],
				Some(2)
			))
		);

		Ok(())
	}
}

#[cfg(test)]
mod match_words_tests {
	use super::*;

	static TEST_WORDS: [&str; 12] = [
		"aaabbb", "bbbccc", "cccddd", "dddeee", "eeefff", "fffggg", "gghhii", "iijjkk", "jjkk",
		"kkll", "yenta", "yes",
	];

	#[test]
	fn should_match_all_words() {
		let token = MatcherToken::MatchAnyWord;

		assert_eq!(
			match_words_from_tokenized(&token, "", "", "", Some(&TEST_WORDS)),
			&TEST_WORDS
		);
	}

	#[test]
	fn should_match_all_words_respecting_globals() {
		let token = MatcherToken::MatchAnyWord;

		assert_eq!(
			match_words_from_tokenized(&token, "", "", "gfjk", Some(&TEST_WORDS)),
			vec!["fffggg", "jjkk"]
		);

		assert_eq!(
			match_words_from_tokenized(&token, "f", "", "gfjk", Some(&TEST_WORDS)),
			vec!["fffggg"]
		);
	}

	#[test]
	fn should_constrain_chars_match_to_tokens_length() {
		let token = MatcherToken::MatchOnChars((
			vec![
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyChar,
			],
			None,
		));

		assert_eq!(
			match_words_from_tokenized(&token, "", "", "", Some(&TEST_WORDS)),
			vec!["jjkk".to_string(), "kkll".to_string()]
		);
	}

	#[test]
	fn should_match_chars_on_tokens() {
		let token = MatcherToken::MatchOnChars((
			vec![
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyCharIn("ab".to_string()),
				MatchCharsToken::ExcludeAllCharsIn("cd".to_string()),
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyChar,
			],
			None,
		));

		assert_eq!(
			match_words_from_tokenized(&token, "", "", "", Some(&TEST_WORDS)),
			vec!["aaabbb".to_string(), "bbbccc".to_string()]
		);

		let token = MatcherToken::MatchOnChars((
			vec![
				MatchCharsToken::MatchAnyCharIn("y".to_string()),
				MatchCharsToken::MatchAnyCharIn("e".to_string()),
			],
			Some(2),
		));

		assert_eq!(
			match_words_from_tokenized(&token, "", "", "", Some(&TEST_WORDS)),
			vec!["yenta".to_string(), "yes".to_string()]
		);

		let token = MatcherToken::MatchOnChars((
			vec![
				MatchCharsToken::MatchAnyCharIn("f".to_string()),
				MatchCharsToken::MatchAnyCharIn("f".to_string()),
				MatchCharsToken::MatchAnyCharIn("f".to_string()),
				MatchCharsToken::MatchAnyCharIn("g".to_string()),
			],
			Some(4),
		));

		assert_eq!(
			match_words_from_tokenized(&token, "", "", "", Some(&TEST_WORDS)),
			vec!["fffggg".to_string()]
		);
	}

	#[test]
	fn should_match_chars_on_tokens_within_globals() {
		let tokens = MatcherToken::MatchOnChars((
			vec![
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyChar,
				MatchCharsToken::MatchAnyChar,
			],
			None,
		));

		assert_eq!(
			match_words_from_tokenized(&tokens, "t", "", "ytanpem", Some(&TEST_WORDS)),
			vec!["yenta".to_string()]
		);
	}
}
