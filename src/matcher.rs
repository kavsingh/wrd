use crate::data::get_words;

#[derive(Debug)]
pub enum MatchOperation {
	MatchAny,
	MatchAnyIn(String),
	ExcludeAllIn(String),
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
