mod data;
mod match_words;
mod notwordle;
mod util;

pub use crate::data::{Dictionary, get_dictionary};
pub use crate::match_words::match_words;
pub use crate::notwordle::{GuessResultToken, Notwordle};
