use std::{str, sync::LazyLock};

use rust_embed::Embed;

#[derive(Embed)]
#[folder = "data"]
struct Assets;

// is this is a good idea? not sure if this is a good idea. feels very hacky.
// TODO: just paste the word list in to a rust file?
static WORD_DATA: LazyLock<Vec<u8>> = LazyLock::new(load_word_data);
pub static WORDS: LazyLock<Vec<&str>> = LazyLock::new(parse_word_list);

fn load_word_data() -> Vec<u8> {
	Assets::get("words.txt")
		.expect("could not load word list")
		.data
		.into_owned()
}

fn parse_word_list() -> Vec<&'static str> {
	str::from_utf8(&WORD_DATA)
		.expect("could not read word list")
		.lines()
		.filter_map(|line| {
			let trimmed = line.trim();

			if trimmed.is_empty() {
				None
			} else {
				Some(trimmed)
			}
		})
		.collect()
}
