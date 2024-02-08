//! [<img alt="github" src="https://img.shields.io/badge/github-udoprog/jpv-8da0cb?style=for-the-badge&logo=github" height="20">](https://github.com/udoprog/jpv)
//! [<img alt="crates.io" src="https://img.shields.io/crates/v/jpv-lib.svg?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/jpv-lib)
//! [<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-jpv--lib-66c2a5?style=for-the-badge&logoColor=white&logo=data:image/svg+xml;base64,PHN2ZyByb2xlPSJpbWciIHhtbG5zPSJodHRwOi8vd3d3LnczLm9yZy8yMDAwL3N2ZyIgdmlld0JveD0iMCAwIDUxMiA1MTIiPjxwYXRoIGZpbGw9IiNmNWY1ZjUiIGQ9Ik00ODguNiAyNTAuMkwzOTIgMjE0VjEwNS41YzAtMTUtOS4zLTI4LjQtMjMuNC0zMy43bC0xMDAtMzcuNWMtOC4xLTMuMS0xNy4xLTMuMS0yNS4zIDBsLTEwMCAzNy41Yy0xNC4xIDUuMy0yMy40IDE4LjctMjMuNCAzMy43VjIxNGwtOTYuNiAzNi4yQzkuMyAyNTUuNSAwIDI2OC45IDAgMjgzLjlWMzk0YzAgMTMuNiA3LjcgMjYuMSAxOS45IDMyLjJsMTAwIDUwYzEwLjEgNS4xIDIyLjEgNS4xIDMyLjIgMGwxMDMuOS01MiAxMDMuOSA1MmMxMC4xIDUuMSAyMi4xIDUuMSAzMi4yIDBsMTAwLTUwYzEyLjItNi4xIDE5LjktMTguNiAxOS45LTMyLjJWMjgzLjljMC0xNS05LjMtMjguNC0yMy40LTMzLjd6TTM1OCAyMTQuOGwtODUgMzEuOXYtNjguMmw4NS0zN3Y3My4zek0xNTQgMTA0LjFsMTAyLTM4LjIgMTAyIDM4LjJ2LjZsLTEwMiA0MS40LTEwMi00MS40di0uNnptODQgMjkxLjFsLTg1IDQyLjV2LTc5LjFsODUtMzguOHY3NS40em0wLTExMmwtMTAyIDQxLjQtMTAyLTQxLjR2LS42bDEwMi0zOC4yIDEwMiAzOC4ydi42em0yNDAgMTEybC04NSA0Mi41di03OS4xbDg1LTM4Ljh2NzUuNHptMC0xMTJsLTEwMiA0MS40LTEwMi00MS40di0uNmwxMDItMzguMiAxMDIgMzguMnYuNnoiPjwvcGF0aD48L3N2Zz4K" height="20">](https://docs.rs/jpv-lib)
#![allow(clippy::large_enum_variant)]
#![allow(clippy::type_complexity)]
#![allow(clippy::match_like_matches_macro)]

/// Dictionary magic `JPVD`.
pub const DICTIONARY_MAGIC: u32 = 0x4a_50_56_44;
/// Current database version in use.
pub const DICTIONARY_VERSION: u32 = 5;

/// Helper to convert a type to its owned variant.
pub use ::borrowme::to_owned;

/// Helper to convert a type to its borrowed variant.
pub use ::borrowme::borrow;

/// Re-export toml support.
pub use ::toml;

#[macro_use]
pub mod reporter;

pub mod token;

#[macro_use]
pub mod inflection;
pub use self::inflection::{Form, Inflection, Inflections, OwnedInflections};

pub mod config;

pub mod data;

pub mod api;

pub use self::dirs::Dirs;
mod dirs;

mod concat;
pub use self::concat::Concat;

pub use self::sort_key::{Key, Weight};
mod sort_key;

pub mod jmdict;
pub mod jmnedict;
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
