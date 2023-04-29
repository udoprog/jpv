#![allow(clippy::large_enum_variant)]

#[macro_use]
mod inflection;
pub use self::inflection::{Form, Inflection, Inflections};

pub mod adjective;

mod concat;
pub use self::concat::Concat;

pub mod elements;

pub mod entities;
pub use self::entities::PartOfSpeech;

mod furigana;
pub use self::furigana::{Furigana, FuriganaGroup};

pub mod romaji;

pub mod kana;

pub mod verb;

mod parser;

mod priority;

pub mod database;

mod musli;
mod serde;

#[doc(hidden)]
pub mod macro_support {
    pub use fixed_map;
}
