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
	tokens: &[MatcherToken],
	include: &str,
	exclude: &str,
	within: &str,
	words: Option<&[&'a str]>,
) -> Vec<&'a str> {
	words
		.unwrap_or_else(|| &WORDS)
		.iter()
		.filter(|word| {
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
		})
		.cloned()
		.collect()
}

pub fn tokenize_pattern(input: &str) -> Result<Vec<MatcherToken>, &'static str> {
	input
		.split(" ")
		.map(|part| tokenize(part))
		.collect::<Result<Vec<_>, _>>()
}

fn tokenize(input: &str) -> Result<MatcherToken, &'static str> {
	if input.contains("*") {
		return Ok(MatcherToken::MatchAny);
	}

	let letters: String = input
		.chars()
		.filter(|c| c.is_ascii() && *c != '!')
		.collect();

	if letters.is_empty() {
		return Err("empty or non-ascii matchers not allowed");
	}

	if input.starts_with("!") {
		Ok(MatcherToken::ExcludeAllIn(letters))
	} else {
		Ok(MatcherToken::MatchAnyIn(letters))
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
		let pattern = vec![
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
		];

		assert_eq!(
			match_words(&pattern, "", "", "", Some(&TEST_WORDS)),
			vec!["jjkk".to_string(), "kkll".to_string()]
		);
	}

	#[test]
	fn should_match_on_tokens() {
		let pattern = vec![
			MatcherToken::MatchAny,
			MatcherToken::MatchAnyIn("ab".to_string()),
			MatcherToken::ExcludeAllIn("cd".to_string()),
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
		];

		assert_eq!(
			match_words(&pattern, "", "", "", Some(&TEST_WORDS)),
			vec!["aaabbb".to_string(), "bbbccc".to_string()]
		);
	}

	#[test]
	fn should_match_on_tokens_within_pool() {
		let pattern = vec![
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
			MatcherToken::MatchAny,
		];

		assert_eq!(
			match_words(&pattern, "t", "", "ytanpem", Some(&TEST_WORDS)),
			vec!["yenta".to_string()]
		);
	}
}
