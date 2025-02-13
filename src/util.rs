pub fn unique_string(str: &str) -> String {
	let mut result: String = "".to_string();

	for char in str.chars() {
		if !result.contains(char) {
			result.push(char);
		}
	}

	result
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn should_return_a_unique_string_keeping_order() {
		assert_eq!(unique_string("abcdeacbede"), "abcde".to_string());
	}
}
