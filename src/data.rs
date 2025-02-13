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
	dbg!("collecting words.json to vec");

	let words = Assets::get("words.json").expect("no words.json found");
	let words_string =
		std::str::from_utf8(words.data.as_ref()).expect("could not convert words.json to string");
	let json = serde_json::from_str::<Value>(words_string).expect("invalid words.json");
	let values = json
		.as_array()
		.expect("expected words.json to be array of strings");

	values
		.iter()
		.filter_map(|val| val.as_str().map(|s| s.to_string()))
		.collect()
}
