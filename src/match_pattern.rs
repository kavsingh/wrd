use crate::data::get_words;

#[derive(Clone, Debug)]
pub enum MatchPatternToken {
	MatchAny,
	MatchAnyIn(String),
	ExcludeAllIn(String),
}

impl PartialEq for MatchPatternToken {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::MatchAny, Self::MatchAny) => true,
			(Self::MatchAnyIn(a), Self::MatchAnyIn(b)) => a == b,
			(Self::ExcludeAllIn(a), Self::ExcludeAllIn(b)) => a == b,
			_ => false,
		}
	}
}

pub fn tokenize_pattern(input: &str) -> Vec<MatchPatternToken> {
	input
		.split(" ")
		.map(|desc| {
			if desc.contains("*") {
				return MatchPatternToken::MatchAny;
			}

			let letters: String = desc.chars().filter(|c| c.is_ascii() && *c != '!').collect();

			if letters.is_empty() {
				panic!("empty or non-ascii descriptors not allowed")
			}

			if desc.starts_with("!") {
				MatchPatternToken::ExcludeAllIn(letters)
			} else {
				MatchPatternToken::MatchAnyIn(letters)
			}
		})
		.collect()
}

pub fn match_words(
	tokens: &[MatchPatternToken],
	include: &str,
	exclude: &str,
	within: &str,
	words: Option<&Vec<String>>,
) -> Vec<String> {
	words
		.unwrap_or(get_words())
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
					Some(MatchPatternToken::MatchAny) => continue,
					Some(MatchPatternToken::MatchAnyIn(chars)) => {
						if !chars.chars().any(|l| l == char) {
							return false;
						}
					}
					Some(MatchPatternToken::ExcludeAllIn(chars)) => {
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_match_on_pattern_length() {
		let pattern = vec![
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAny,
		];

		assert_eq!(
			match_words(&pattern, "", "", "", None),
			vec!["jjkk".to_string(), "kkll".to_string()]
		);
	}

	#[test]
	fn should_match_on_pattern() {
		let pattern = vec![
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAnyIn("ab".to_string()),
			MatchPatternToken::ExcludeAllIn("cd".to_string()),
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAny,
		];

		assert_eq!(
			match_words(&pattern, "", "", "", None),
			vec!["aaabbb".to_string(), "bbbccc".to_string()]
		);
	}

	#[test]
	fn should_match_on_pattern_within_pool() {
		let pattern = vec![
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAny,
			MatchPatternToken::MatchAny,
		];

		assert_eq!(
			match_words(&pattern, "t", "", "ytanpem", None),
			vec!["yenta".to_string()]
		);
	}
}
