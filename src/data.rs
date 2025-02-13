use rust_embed::Embed;
use serde_json::Value;
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

	let words = Assets::get("words.json").expect("could not get dictionary");
	let words_string =
		std::str::from_utf8(words.data.as_ref()).expect("could not convert dictionary to string");
	let json = serde_json::from_str::<Value>(words_string).expect("invalid dictionary json");
	let values = json
		.as_array()
		.expect("expected dictionary to be array of strings");

	let mut strings: Vec<String> = values
		.iter()
		.filter_map(|val| val.as_str().map(|s| s.to_string()))
		.collect();

	strings.sort_unstable();

	strings
}
