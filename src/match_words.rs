/*
'* * q u e s' >  /^[a-z]{1}[a-z]{1}ques$/
'** qr !sed * **' > /^[a-z]*[qr]{1}(?![sed])[a-z]{1}[a-z]*$/
 */

use std::sync::LazyLock;

use regex::Regex;

use crate::{data::WORDS, util::non_empty_str};

#[derive(Clone, Debug)]
pub enum MatcherToken {
	// **
	MatchAnyChars,
	// *
	MatchAnyChar,
	// a-z
	MatchAnyCharIn(String),
	// !a-z
	ExcludeAllCharsIn(String),
}

impl PartialEq for MatcherToken {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::MatchAnyChars, Self::MatchAnyChars) => true,
			(Self::MatchAnyChar, Self::MatchAnyChar) => true,
			(Self::MatchAnyCharIn(a), Self::MatchAnyCharIn(b)) => a == b,
			(Self::ExcludeAllCharsIn(a), Self::ExcludeAllCharsIn(b)) => a == b,
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
	let result = match_words_from_tokens(&tokens, include, exclude, within, words)?;

	Ok(result)
}

pub fn match_words_from_tokens<'a>(
	tokens: &[MatcherToken],
	include: &str,
	exclude: &str,
	within: &str,
	words: Option<&[&'a str]>,
) -> Result<Vec<&'a str>, String> {
	let regex = regex_from_tokens(tokens)?;
	let result: Vec<&str> = words
		.unwrap_or_else(|| &WORDS)
		.iter()
		.filter(|word| match_word(word, &regex, include, exclude, within))
		.cloned()
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

fn regex_from_tokens(tokens: &[MatcherToken]) -> Result<Regex, String> {
	let pattern = tokens
		.iter()
		.map(|token| match token {
			MatcherToken::MatchAnyChars => r"[a-z]*".to_string(),
			MatcherToken::MatchAnyChar => r"[a-z]{1}".to_string(),
			MatcherToken::MatchAnyCharIn(chars) => format!("[{chars}]{{1}}"),
			MatcherToken::ExcludeAllCharsIn(chars) => format!("[a-z[^{chars}]]{{1}}"),
		})
		.collect::<String>();
	let bounded = format!("^{pattern}$");

	Regex::new(&bounded).map_err(|err| format!("invalid regex {pattern}: {err}"))
}

fn tokenize_pattern(input: &str) -> Result<Vec<MatcherToken>, String> {
	let parts: Vec<_> = input.split(" ").filter_map(non_empty_str).collect();

	if parts.is_empty() {
		return Err("invalid empty input".to_string());
	}

	let tokens = parts
		.iter()
		.fold(vec![], |mut acc: Vec<&str>, part| {
			if let Some(last) = acc.last() {
				if *last == "**" && *part == "**" {
					return acc;
				}
			}

			acc.push(part);
			acc
		})
		.iter()
		.map(|part| tokenize(part))
		.collect::<Result<Vec<_>, _>>()?;

	Ok(tokens)
}

static MATCH_CHARS_TOKEN_REGEX: LazyLock<Regex> =
	LazyLock::new(|| Regex::new(r"^(\!)?([a-z]+)$").expect("invalid match token regex"));

fn tokenize(input: &str) -> Result<MatcherToken, String> {
	if input == "**" {
		return Ok(MatcherToken::MatchAnyChars);
	}

	if input == "*" {
		return Ok(MatcherToken::MatchAnyChar);
	}

	let captures = match MATCH_CHARS_TOKEN_REGEX.captures(input) {
		Some(c) => c,
		None => return Err(format!("invalid input {input}")),
	};

	match (
		captures.get(1).map(|c| c.as_str()),
		captures.get(2).map(|c| c.as_str().to_owned()),
	) {
		(Some("!"), Some(letters)) => Ok(MatcherToken::ExcludeAllCharsIn(letters)),
		(None, Some(letters)) => Ok(MatcherToken::MatchAnyCharIn(letters)),
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
		assert_eq!(tokenize_pattern("**")?, vec![MatcherToken::MatchAnyChars]);

		assert_eq!(
			tokenize_pattern("* ** **")?,
			vec![MatcherToken::MatchAnyChar, MatcherToken::MatchAnyChars,]
		);

		assert_eq!(
			tokenize_pattern("* a !bcd ** ** ** ef * **")?,
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
	fn should_match_all_words() -> Result<(), String> {
		let tokens = vec![MatcherToken::MatchAnyChars];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS))?,
			&TEST_WORDS
		);

		Ok(())
	}

	#[test]
	fn should_match_all_words_respecting_globals() -> Result<(), String> {
		let tokens = vec![MatcherToken::MatchAnyChars];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "gfjk", Some(&TEST_WORDS))?,
			vec!["fffggg", "jjkk"]
		);

		assert_eq!(
			match_words_from_tokens(&tokens, "f", "", "gfjk", Some(&TEST_WORDS))?,
			vec!["fffggg"]
		);

		Ok(())
	}

	#[test]
	fn should_constrain_chars_match_to_tokens_length() -> Result<(), String> {
		let tokens = vec![
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS))?,
			vec!["jjkk".to_string(), "kkll".to_string()]
		);

		Ok(())
	}

	#[test]
	fn should_match_chars_on_tokens() -> Result<(), String> {
		let tokens = vec![
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyCharIn("ab".to_string()),
			MatcherToken::ExcludeAllCharsIn("cd".to_string()),
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS))?,
			vec!["aaabbb".to_string(), "bbbccc".to_string()]
		);

		let tokens = vec![
			MatcherToken::MatchAnyCharIn("y".to_string()),
			MatcherToken::MatchAnyCharIn("e".to_string()),
			MatcherToken::MatchAnyChars,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "", "", "", Some(&TEST_WORDS))?,
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
			match_words_from_tokens(&token, "", "", "", Some(&TEST_WORDS))?,
			vec!["fffggg".to_string()]
		);

		Ok(())
	}

	#[test]
	fn should_match_chars_on_tokens_within_globals() -> Result<(), String> {
		let tokens = vec![
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
			MatcherToken::MatchAnyChar,
		];

		assert_eq!(
			match_words_from_tokens(&tokens, "t", "", "ytanpem", Some(&TEST_WORDS))?,
			vec!["yenta".to_string()]
		);

		Ok(())
	}
}
