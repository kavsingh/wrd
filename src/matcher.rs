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

pub fn match_from_pattern(pattern: &MatchPattern, include: &str, exclude: &str) -> Vec<String> {
	get_words()
		.iter()
		.filter(|word| {
			if word.len() != pattern.len() {
				return false;
			}

			if !include.is_empty() && include.chars().any(|c| !word.contains(c)) {
				return false;
			}

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
			match_from_pattern(&pattern, "", ""),
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
			match_from_pattern(&pattern, "", ""),
			vec!["aaabbb".to_string(), "bbbccc".to_string()]
		);
	}
}
