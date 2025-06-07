use std::borrow::Cow;
use std::collections::HashMap;
use std::str;
use std::sync::LazyLock;

use rust_embed::Embed;

use crate::util::non_empty_str;

#[derive(Embed)]
#[folder = "data"]
struct Assets;

// is this is a good idea? not sure if this is a good idea. feels very hacky.
// TODO: just paste paste dictionaries in as rust files?
static DICT_DATA: LazyLock<HashMap<&'static str, Cow<'static, [u8]>>> =
	LazyLock::new(load_dict_data);
static DICTIONARIES: LazyLock<HashMap<&'static str, Vec<&'static str>>> =
	LazyLock::new(parse_dict_data);

#[derive(Debug, Clone, PartialEq)]
pub enum Dictionary {
	Moby,
	Gwicks,
}

impl Dictionary {
	pub fn name(&self) -> &'static str {
		match *self {
			Self::Moby => "moby",
			Self::Gwicks => "gwicks",
		}
	}

	fn asset(&self) -> &'static str {
		match *self {
			Self::Moby => "words-moby.txt",
			Self::Gwicks => "words-gwicks-usa2.txt",
		}
	}
}

pub fn get_dictionary(dict: &Dictionary) -> &'static Vec<&'static str> {
	let name = dict.name();

	DICTIONARIES
		.get(name)
		.unwrap_or_else(|| panic!("{name} data not found"))
}

fn load_dict_data() -> HashMap<&'static str, Cow<'static, [u8]>> {
	let mut data = HashMap::new();

	for dict in [Dictionary::Moby, Dictionary::Gwicks] {
		data.insert(dict.name(), load_data(dict.asset()));
	}

	data
}

fn parse_dict_data() -> HashMap<&'static str, Vec<&'static str>> {
	let mut dicts = HashMap::new();

	for dict in [Dictionary::Moby, Dictionary::Gwicks] {
		dicts.insert(dict.name(), parse_data(dict.name()));
	}

	dicts
}

fn load_data(name: &str) -> Cow<'static, [u8]> {
	Assets::get(name)
		.unwrap_or_else(|| panic!("could not load {name}"))
		.data
}

fn parse_data(name: &str) -> Vec<&'static str> {
	str::from_utf8(
		DICT_DATA
			.get(name)
			.unwrap_or_else(|| panic!("{name} data not found")),
	)
	.unwrap_or_else(|e| panic!("could not read {name}: {}", e))
	.lines()
	.filter_map(non_empty_str)
	.collect()
}
