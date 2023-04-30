//! Module which performs verb inflection, based on a words class.

mod godan;
#[macro_use]
mod macros;

use std::collections::BTreeMap;

use crate::elements::Entry;
use crate::entities::KanjiInfo;
use crate::inflection::Inflections;
use crate::kana::{Fragments, Full};
use crate::PartOfSpeech;

/// Try to conjugate the given entry as a verb.
pub fn conjugate<'a>(entry: &Entry<'a>) -> Option<Inflections<'a>> {
    let (Some(kind), [kanji, ..], [reading, ..]) = (as_verb_kind(entry), &entry.kanji_elements[..], &entry.reading_elements[..]) else {
        return None;
    };

    let kanji_text = if kanji.info.contains(KanjiInfo::RareKanji) {
        reading.text
    } else {
        kanji.text
    };

    let (mut inflections, stem, de) = match kind {
        VerbKind::Ichidan => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix('る'), reading.text.strip_suffix('る')) else {
                return None;
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
            (inflections, Fragments::new([k], [r], []), false)
        }
        VerbKind::GodanIku => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix('く'), reading.text.strip_suffix('く')) else {
                return None;
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
            (inflections, Fragments::new([k], [r], []), g.de)
        }
        VerbKind::Godan => {
            let mut k = kanji_text.chars();
            let mut r = reading.text.chars();

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
                _ => return None,
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
            (inflections, Fragments::new([k], [r], []), g.de)
        }
        VerbKind::Suru => {
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("する"), reading.text.strip_suffix("する")) else {
                return None;
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
            let (Some(k), Some(r)) = (kanji_text.strip_suffix("来る"), reading.text.strip_suffix("くる")) else {
                return None;
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

        inflections.insert(inflect!(TeIru, Te, Alternate), p.concat(["る"]));
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
        inflections.insert(inflect!(Te, TeOku, Alternate), p.concat(["く"]));

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

    Some(Inflections {
        dictionary: Full::new(kanji_text, reading.text, ""),
        inflections,
    })
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
                PartOfSpeech::VerbSuruIncluded => VerbKind::Suru,
                PartOfSpeech::VerbSuruSpecial => VerbKind::Suru,
                _ => continue,
            };

            return Some(kind);
        }
    }

    None
}
