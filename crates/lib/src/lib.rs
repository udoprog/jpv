#![allow(clippy::large_enum_variant)]
#![allow(clippy::type_complexity)]
#![allow(clippy::match_like_matches_macro)]

/// Dictionary magic.
pub const DICTIONARY_MAGIC: u32 = 0x4a_50_56_44; // "JPVD";
/// Current database version in use.
pub const DICTIONARY_VERSION: u32 = 1;

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
