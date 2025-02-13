use crate::data::get_words;

#[derive(Debug)]
pub enum MatchOperation {
	MatchAnything,
	MatchOneOf(Vec<String>),
	ExcludeAllOf(Vec<String>),
}

pub type MatchPattern = Vec<MatchOperation>;

pub fn match_from_pattern(
	pattern: &MatchPattern,
	include: &Vec<&str>,
	exclude: &Vec<&str>,
) -> Vec<String> {
	get_words()
		.iter()
		.filter(|word| {
			if word.len() != pattern.len() {
				return false;
			}

			if !include.is_empty() && include.iter().any(|c| !word.contains(c)) {
				return false;
			}

			if !exclude.is_empty() && exclude.iter().any(|c| word.contains(c)) {
				return false;
			}

			let chars: Vec<&str> = word.split("").filter(|c| !c.is_empty()).collect();

			for (i, char) in chars.iter().enumerate() {
				let matcher = match pattern.get(i) {
					Some(op) => op,
					None => continue,
				};

				match matcher {
					MatchOperation::MatchAnything => continue,
					MatchOperation::MatchOneOf(letters) => {
						if !letters.iter().any(|l| l == char) {
							return false;
						}

						continue;
					}
					MatchOperation::ExcludeAllOf(letters) => {
						if letters.iter().any(|l| l == char) {
							return false;
						}

						continue;
					}
				}
			}

			true
		})
		.cloned()
		.collect()
}
