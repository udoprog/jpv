//! Module which performs verb inflection, based on a words class.

mod godan;
#[macro_use]
mod macros;

use std::collections::BTreeMap;

use musli::{Decode, Encode};
use musli_zerocopy::ZeroCopy;
use serde::{Deserialize, Serialize};

use crate::elements::Entry;
use crate::inflection::Inflections;
use crate::kana::{Fragments, Full};
use crate::PartOfSpeech;

#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    ZeroCopy,
)]
#[repr(u8)]
#[zero_copy(bounds = {T: ZeroCopy})]
#[musli(bound = {T: Encode<M>}, decode_bound = {T: Decode<'de, M>})]
pub enum ReadingOption<T> {
    None,
    Some(T),
}

/// The reading which this set of inflections belong to.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    Serialize,
    Deserialize,
    Encode,
    Decode,
    ZeroCopy,
)]
#[repr(C)]
pub struct Reading {
    /// Index of the kanji that the reading matches, if one is present.
    pub kanji: u8,
    /// Index of the reading used.
    pub reading: u8,
}

/// Try to conjugate the given entry as a verb.
pub fn conjugate<'a>(entry: &Entry<'a>) -> Vec<(Reading, Inflections<'a>)> {
    let Some(kind) = as_verb_kind(entry) else {
        return Vec::new();
    };

    let mut output = Vec::new();

    let mut readings = Vec::new();

    for (reading_index, reading) in entry.reading_elements.iter().enumerate() {
        if reading.no_kanji {
            readings.push((None, (reading_index, reading.text)));
            continue;
        }

        for (kanji_index, kanji) in entry.kanji_elements.iter().enumerate() {
            if reading.applies_to(&kanji.text) {
                readings.push((
                    Some((kanji_index, kanji.text)),
                    (reading_index, reading.text),
                ));
            }
        }
    }

    for (kanji, reading) in readings {
        let (_, kanji_text) = kanji.unwrap_or(reading);
        let (_, reading_text) = reading;

        let (mut inflections, stem, de) = match kind {
            VerbKind::Ichidan => {
                let (Some(k), Some(r)) = (
                    kanji_text.strip_suffix('る'),
                    reading_text.strip_suffix('る'),
                ) else {
                    continue;
                };

                let mut inflections = inflections! {
                    k, r,
                    Te ("て"),
                };

                macro_rules! populate {
                    ($suffix:expr $(, $inflect:ident)*) => {
                        inflections.insert(inflect!($($inflect),*), Fragments::new([k], [r], [$suffix]));
                    }
                }

                ichidan!(populate);
                (inflections, Fragments::new([k], [r], ["っ"]), false)
            }
            VerbKind::GodanIku => {
                let (Some(k), Some(r)) = (
                    kanji_text.strip_suffix('く'),
                    reading_text.strip_suffix('く'),
                ) else {
                    continue;
                };

                let g = godan::IKU;

                let mut inflections = BTreeMap::new();
                inflections.insert(inflect!(Te), Fragments::new([k], [r], [g.te]));

                macro_rules! populate {
                    ($suffix:expr $(, $inflect:ident)*) => {
                        inflections.insert(inflect!($($inflect),*), Fragments::new([k], [r], $suffix));
                    }
                }

                godan!(populate, g);
                (inflections, Fragments::new([k], [r], [g.te_stem]), g.de)
            }
            VerbKind::Godan => {
                let mut k = kanji_text.chars();
                let mut r = reading_text.chars();

                let g = match k.next_back() {
                    Some('う') => godan::U,
                    Some('つ') => godan::TSU,
                    Some('る') => godan::RU,
                    Some('く') => godan::KU,
                    Some('ぐ') => godan::GU,
                    Some('む') => godan::MU,
                    Some('ぶ') => godan::BU,
                    Some('ぬ') => godan::NU,
                    Some('す') => godan::SU,
                    _ => continue,
                };

                r.next_back();

                let k = k.as_str();
                let r = r.as_str();

                let mut inflections = BTreeMap::new();
                inflections.insert(inflect!(Te), Fragments::new([k], [r], [g.te]));

                macro_rules! populate {
                    ($suffix:expr $(, $inflect:ident)*) => {
                        inflections.insert(inflect!($($inflect),*), Fragments::new([k], [r], $suffix));
                    }
                }

                godan!(populate, g);
                (inflections, Fragments::new([k], [r], [g.te_stem]), g.de)
            }
            VerbKind::Suru => {
                let (k, r) = if reading_text == "する" {
                    // TODO: actualy fix suru conjugations to cope with the
                    // irregular kanji.
                    (reading_text, reading_text)
                } else {
                    let (Some(k), Some(r)) = (
                        kanji_text.strip_suffix("する"),
                        reading_text.strip_suffix("する"),
                    ) else {
                        continue;
                    };

                    (k, r)
                };

                let mut inflections = BTreeMap::new();
                inflections.insert(inflect!(Te), Fragments::new([k], [r], ["して"]));

                macro_rules! populate {
                    ($suffix:expr $(, $inflect:ident)*) => {
                        inflections.insert(inflect!($($inflect),*), Fragments::new([k], [r], [$suffix]));
                    }
                }

                suru!(populate);
                (inflections, Fragments::new([k], [r], []), false)
            }
            VerbKind::Kuru => {
                let (Some(k), Some(r)) = (
                    kanji_text.strip_suffix("来る"),
                    reading_text.strip_suffix("くる"),
                ) else {
                    continue;
                };

                let mut inflections = BTreeMap::new();
                inflections.insert(inflect!(Te), Fragments::new([k, "来"], [r, "き"], ["て"]));

                macro_rules! populate {
                    ($r:expr, $suffix:expr $(, $inflect:ident)*) => {
                        inflections.insert(inflect!($($inflect),*), Fragments::new([k, "来"], [r, $r], [$suffix]));
                    }
                }

                kuru!(populate);
                (inflections, Fragments::new([k], [r], []), false)
            }
        };

        if let Some(p) = inflections.get(&inflect!(Te)).cloned() {
            macro_rules! populate {
                ($suffix:expr $(, $inflect:ident)*) => {
                    inflections.insert(inflect!(TeIru, Te $(, $inflect)*), p.concat([concat!("い", $suffix)]));
                }
            }

            inflections.insert(inflect!(TeIru, Te, Short), p.concat(["る"]));
            ichidan!(populate);

            macro_rules! populate {
                ($suffix:expr $(, $inflect:ident)*) => {
                    inflections.insert(inflect!(TeAru, Te $(, $inflect)*), p.concat($suffix));
                }
            }

            godan!(populate, godan::RU, "あ");

            macro_rules! populate {
                ($suffix:expr $(, $inflect:ident)*) => {
                    inflections.insert(inflect!(TeIku, Te $(, $inflect)*), p.concat($suffix));
                }
            }

            godan!(populate, godan::IKU, "い");

            macro_rules! populate {
                ($suffix:expr $(, $inflect:ident)*) => {
                    inflections.insert(inflect!(TeShimau, Te $(, $inflect)*), p.concat($suffix));
                }
            }

            godan!(populate, godan::U, "しま");

            macro_rules! populate {
                ($suffix:expr $(, $inflect:ident)*) => {
                    inflections.insert(inflect!(TeOku, Te $(, $inflect)*), p.concat($suffix));
                }
            }

            godan!(populate, godan::KU, "お");
            inflections.insert(inflect!(Te, TeOku, Short), p.concat(["く"]));

            macro_rules! populate {
                ($r:expr, $suffix:expr $(, $inflect:ident)*) => {
                    inflections.insert(inflect!(TeKuru, Te $(, $inflect)*), p.concat([concat!($r, $suffix)]));
                }
            }

            kuru!(populate);
        }

        if !stem.is_empty() {
            macro_rules! populate {
                ($suffix:expr $(, $inflect:ident)*) => {
                    inflections.insert(inflect!(Chau $(, $inflect)*), stem.concat($suffix));
                }
            }

            if de {
                godan!(populate, godan::U, "じゃ");
            } else {
                godan!(populate, godan::U, "ちゃ");
            }
        }

        let reading = Reading {
            kanji: kanji.map(|(i, _)| i as u8).unwrap_or(u8::MAX),
            reading: reading.0 as u8,
        };

        let inflections = Inflections {
            dictionary: Full::new(kanji_text, reading_text, ""),
            inflections,
        };

        output.push((reading, inflections));
    }

    output
}

#[derive(Debug, Clone, Copy)]
enum VerbKind {
    /// Ichidan verb.
    Ichidan,
    /// Godan verb.
    Godan,
    /// Special godan verb.
    GodanIku,
    /// Suru irregular suru verb.
    Suru,
    /// Special irregular kuru verb.
    Kuru,
}

/// If the entry is a verb, figure out the verb kind.
fn as_verb_kind(entry: &Entry<'_>) -> Option<VerbKind> {
    for sense in &entry.senses {
        for pos in sense.pos.iter() {
            let kind = match pos {
                PartOfSpeech::VerbIchidan => VerbKind::Ichidan,
                PartOfSpeech::VerbIchidanS => VerbKind::Ichidan,
                PartOfSpeech::VerbGodanAru => VerbKind::Godan,
                PartOfSpeech::VerbGodanB => VerbKind::Godan,
                PartOfSpeech::VerbGodanG => VerbKind::Godan,
                PartOfSpeech::VerbGodanK => VerbKind::Godan,
                PartOfSpeech::VerbGodanKS => VerbKind::GodanIku,
                PartOfSpeech::VerbGodanM => VerbKind::Godan,
                PartOfSpeech::VerbGodanN => VerbKind::Godan,
                PartOfSpeech::VerbGodanR => VerbKind::Godan,
                PartOfSpeech::VerbGodanRI => VerbKind::Godan,
                PartOfSpeech::VerbGodanS => VerbKind::Godan,
                PartOfSpeech::VerbGodanT => VerbKind::Godan,
                PartOfSpeech::VerbGodanU => VerbKind::Godan,
                PartOfSpeech::VerbGodanUS => VerbKind::Godan,
                PartOfSpeech::VerbGodanUru => VerbKind::Godan,
                PartOfSpeech::VerbKuru => VerbKind::Kuru,
                PartOfSpeech::VerbSuru => VerbKind::Suru,
                PartOfSpeech::VerbSuruSpecial => VerbKind::Suru,
                PartOfSpeech::VerbSuruIncluded => VerbKind::Suru,
                _ => continue,
            };

            return Some(kind);
        }
    }

    None
}
