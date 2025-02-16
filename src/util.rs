pub fn unique_string(str: &str) -> String {
	str.chars().fold("".to_owned(), |mut acc: String, c| {
		if !acc.contains(c) {
			acc.push(c)
		}

		acc
	})
}

pub fn non_empty_str(str: &str) -> Option<&str> {
	let trimmed = str.trim();

	if trimmed.is_empty() {
		None
	} else {
		Some(trimmed)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_return_a_unique_string_keeping_order() {
		assert_eq!(unique_string("abcdeacbede"), "abcde".to_string());
	}
}
