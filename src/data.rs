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
	let dict = Assets::get("words.txt").expect("could not get dictionary");
	let content =
		std::str::from_utf8(dict.data.as_ref()).expect("could not convert dictionary to string");
	let mut strings: Vec<String> = content
		.lines()
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
