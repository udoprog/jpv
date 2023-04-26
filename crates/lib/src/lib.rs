#![allow(clippy::large_enum_variant)]

pub mod adjective;

mod concat;
pub use self::concat::Concat;

pub mod elements;

mod entities;
pub use self::entities::PartOfSpeech;

mod furigana;
pub use self::furigana::Furigana;

mod kana;

pub mod verb;

mod parser;

mod priority;

pub mod database;

mod musli;
