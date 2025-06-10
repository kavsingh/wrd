use wrd_lib::Dictionary;

#[derive(Debug)]
pub struct AppState {
	pub dictionary: Dictionary,
	pub cursor_position: Option<(u16, u16)>,
}

impl Default for AppState {
	fn default() -> Self {
		Self {
			dictionary: Dictionary::Moby,
			cursor_position: None,
		}
	}
}
