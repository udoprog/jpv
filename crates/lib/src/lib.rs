#![allow(clippy::large_enum_variant)]

#[macro_use]
pub mod inflection;
pub use self::inflection::{Form, Inflection, Inflections, OwnedInflections};

pub mod api;

mod concat;
pub use self::concat::Concat;

pub mod jmdict;
pub mod kanjidic2;

pub mod entities;
pub use self::entities::PartOfSpeech;

mod furigana;
pub use self::furigana::{Furigana, FuriganaGroup};

pub mod romaji;

pub mod kana;

mod priority;
pub use self::priority::Priority;

pub mod database;

mod musli;

#[doc(hidden)]
pub mod macro_support {
    pub use fixed_map;
}
