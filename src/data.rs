use rust_embed::Embed;
use std::sync::LazyLock;

#[derive(Embed)]
#[folder = "data"]
struct Assets;

// is this is a good idea? not sure if this is a good idea. seems like a lot
// of work for a first init.
pub static WORD_LIST: LazyLock<Vec<String>> = LazyLock::new(load_word_list);

fn load_word_list() -> Vec<String> {
	let dict = Assets::get("words.txt").expect("could not get word list");
	let content = std::str::from_utf8(dict.data.as_ref()).expect("could not load word list");
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
