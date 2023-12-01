#[macro_use]
mod tools;
use self::tools::{colon, comma, iter, ruby, seq, spacing};

pub(crate) mod entry;
pub(crate) use self::entry::Entry;

pub(crate) mod name;
pub(crate) use self::name::Name;

pub(crate) mod character;
pub(crate) use self::character::Character;

pub(crate) mod prompt;
pub(crate) use self::prompt::Prompt;
