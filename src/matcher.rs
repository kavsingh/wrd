use crate::data::get_words;

#[derive(Clone, Debug)]
pub enum MatchOperation {
	MatchAny,
	MatchAnyIn(String),
	ExcludeAllIn(String),
}

impl PartialEq for MatchOperation {
	fn eq(&self, other: &Self) -> bool {
		match (self, other) {
			(Self::MatchAny, Self::MatchAny) => true,
			(Self::MatchAnyIn(a), Self::MatchAnyIn(b)) => a == b,
			(Self::ExcludeAllIn(a), Self::ExcludeAllIn(b)) => a == b,
			_ => false,
		}
	}
}

pub type MatchPattern = Vec<MatchOperation>;

pub fn parse_pattern(descriptor: &str) -> MatchPattern {
	descriptor
		.split(" ")
		.map(|desc| {
			if desc.contains("*") {
				return MatchOperation::MatchAny;
			}

			let letters: String = desc.chars().filter(|c| c.is_ascii() && *c != '!').collect();

			if letters.is_empty() {
				panic!("empty or non-ascii descriptors not allowed")
			}

			if desc.starts_with("!") {
				MatchOperation::ExcludeAllIn(letters)
			} else {
				MatchOperation::MatchAnyIn(letters)
			}
		})
		.collect()
}

pub fn match_from_pattern(
	pattern: &MatchPattern,
	include: &str,
	exclude: &str,
	within: &str,
) -> Vec<String> {
	get_words()
		.iter()
		.filter(|word| {
			if word.len() != pattern.len() {
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
				let matcher = match pattern.get(i) {
					Some(op) => op,
					None => continue,
				};

				match matcher {
					MatchOperation::MatchAny => continue,
					MatchOperation::MatchAnyIn(letters) => {
						if !letters.chars().any(|l| l == char) {
							return false;
						}
					}
					MatchOperation::ExcludeAllIn(letters) => {
						if letters.chars().any(|l| l == char) {
							return false;
						}
					}
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
			MatchOperation::MatchAny,
			MatchOperation::MatchAny,
			MatchOperation::MatchAny,
			MatchOperation::MatchAny,
		];

		assert_eq!(
			match_from_pattern(&pattern, "", "", ""),
			vec!["jjkk".to_string(), "kkll".to_string()]
		);
	}

	#[test]
	fn should_match_on_pattern() {
		let pattern = vec![
			MatchOperation::MatchAny,
			MatchOperation::MatchAnyIn("ab".to_string()),
			MatchOperation::ExcludeAllIn("cd".to_string()),
			MatchOperation::MatchAny,
			MatchOperation::MatchAny,
			MatchOperation::MatchAny,
		];

		assert_eq!(
			match_from_pattern(&pattern, "", "", ""),
			vec!["aaabbb".to_string(), "bbbccc".to_string()]
		);
	}

	#[test]
	fn should_match_on_pattern_within_pool() {
		let pattern = vec![
			MatchOperation::MatchAny,
			MatchOperation::MatchAny,
			MatchOperation::MatchAny,
			MatchOperation::MatchAny,
			MatchOperation::MatchAny,
		];

		assert_eq!(
			match_from_pattern(&pattern, "t", "", "ytanpem"),
			vec!["yenta".to_string()]
		);
	}
}
