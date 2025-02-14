use rust_embed::Embed;
use std::sync::OnceLock;

#[derive(Embed)]
#[folder = "data"]
struct Assets;

static WORDS_LIST: OnceLock<Vec<String>> = OnceLock::new();

pub fn get_words() -> &'static Vec<String> {
	WORDS_LIST.get_or_init(parse_words_json)
}

fn parse_words_json() -> Vec<String> {
	if cfg!(test) {
		return vec![
			"aaabbb".to_string(),
			"bbbccc".to_string(),
			"cccddd".to_string(),
			"dddeee".to_string(),
			"eeefff".to_string(),
			"fffggg".to_string(),
			"gghhii".to_string(),
			"iijjkk".to_string(),
			"jjkk".to_string(),
			"kkll".to_string(),
		];
	}

	let dict = Assets::get("words.txt").expect("could not get dictionary");
	let content =
		std::str::from_utf8(dict.data.as_ref()).expect("could not convert dictionary to string");
	let mut strings: Vec<String> = content
		.split("\n")
		.filter_map(|line| {
			let trimmed = line.trim();

			if trimmed.is_empty() {
				None
			} else {
				Some(trimmed.to_string())
			}
		})
		.collect();

	strings.sort_unstable();

	strings
}
