//! Module which performs verb inflection, based on a words class.

#![cfg_attr(fake, allow(dead_code, unused, unused_variables, unused_macros))]

use std::collections::BTreeMap;

use fixed_map::Set;
use musli::{Decode, Encode};
use musli_zerocopy::ZeroCopy;
use serde::{Deserialize, Serialize};

use crate::elements::Entry;
use crate::inflection::{Inflection, Inflections};
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

/// The kind of word.
pub enum Kind {
    Verb,
    Adjective,
}

/// Try to conjugate the given entry as a verb.
pub fn conjugate<'a>(entry: &Entry<'a>) -> Vec<(Reading, Inflections<'a>, Kind)> {
    let mut output = Vec::new();

    let readings = reading_permutations(entry);

    for pos in parts_of_speech(entry) {
        for &(kanji, reading) in &readings {
            let (_, kanji_text) = kanji.unwrap_or(reading);
            let (_, reading_text) = reading;

            let mut inflections = BTreeMap::<Inflection, Fragments<'_, 3, 4>>::new();

            macro_rules! insert {
                (($($form:ident),* $(,)?), $word:expr) => {
                    inflections.insert(inflect!($($form),*), $word);
                }
            }

            let kind;
            let chau_stem: Option<(Fragments<'_, 3, 4>, bool)>;

            macro_rules! allowlist {
                ($($expected:literal),*) => {
                    if let Some((_, kanji_text)) = kanji {
                        if !(false $(|| kanji_text == $expected)*) {
                            let alts: Vec<String> = vec![$(format!("'{}'", $expected)),*];
                            let alts = alts.join(" / ");
                            tracing::warn!("Expected to end in {alts}: {kanji_text} / {reading_text}: {:?}", entry);
                        } else {
                            tracing::info!("{:?}", entry);
                        }
                    }
                }
            }

            match pos {
                PartOfSpeech::VerbIchidan | PartOfSpeech::VerbIchidanS => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'る') else {
                        allowlist!("買い増す");
                        continue;
                    };

                    macro_rules! populate {
                        ($suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [$suffix]));
                        }
                    }

                    ichidan!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["っ"]), false));
                }
                PartOfSpeech::VerbGodanKS => {
                    let Some((kanji_stem, reading_prefix)) =
                        extract_stem(kanji_text, reading_text, 'く')
                    else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([kanji_stem], [reading_prefix, $prefix], [$suffix]));
                        }
                    }

                    godan_iku!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((
                        Fragments::new([kanji_stem], [reading_prefix, "き"], ["っ"]),
                        false,
                    ));
                }
                PartOfSpeech::VerbGodanU | PartOfSpeech::VerbGodanUS => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'う') else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [concat!($prefix, $suffix)]));
                        }
                    }

                    godan_u!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["っ"]), false));
                }
                PartOfSpeech::VerbGodanT => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'つ') else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [concat!($prefix, $suffix)]));
                        }
                    }

                    godan_tsu!(populate, te);

                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["っ"]), false));
                }
                PartOfSpeech::VerbGodanR
                | PartOfSpeech::VerbGodanRI
                | PartOfSpeech::VerbGodanAru
                | PartOfSpeech::VerbGodanUru => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'る') else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [concat!($prefix, $suffix)]));
                        }
                    }

                    godan_ru!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["っ"]), false));
                }
                PartOfSpeech::VerbGodanK => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'く') else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [concat!($prefix, $suffix)]));
                        }
                    }

                    godan_ku!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["い"]), false));
                }
                PartOfSpeech::VerbGodanG => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'ぐ') else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [concat!($prefix, $suffix)]));
                        }
                    }

                    godan_gu!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["い"]), true));
                }
                PartOfSpeech::VerbGodanM => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'む') else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [concat!($prefix, $suffix)]));
                        }
                    }

                    godan_mu!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["ん"]), true));
                }
                PartOfSpeech::VerbGodanB => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'ぶ') else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [concat!($prefix, $suffix)]));
                        }
                    }

                    godan_bu!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["ん"]), true));
                }
                PartOfSpeech::VerbGodanN => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'ぬ') else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [concat!($prefix, $suffix)]));
                        }
                    }

                    godan_nu!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["ん"]), true));
                }
                PartOfSpeech::VerbGodanS => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'す') else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [concat!($prefix, $suffix)]));
                        }
                    }

                    godan_su!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = Some((Fragments::new([k], [r], ["し"]), false));
                }
                PartOfSpeech::VerbSuruSpecial | PartOfSpeech::VerbSuruIncluded => {
                    let Some((kanji_stem, reading_prefix)) =
                        extract_stem(kanji_text, reading_text, 'る')
                    else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([kanji_stem], [reading_prefix, $prefix], [$suffix]));
                        }
                    }

                    suru!(populate, te);
                    chau_stem = Some((
                        Fragments::new([kanji_stem], [reading_prefix, "し"], []),
                        false,
                    ));
                    kind = Kind::Verb;
                }
                PartOfSpeech::VerbKuru => {
                    let Some((kanji_stem, reading_prefix)) =
                        extract_stem(kanji_text, reading_text, 'る')
                    else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([kanji_stem], [reading_prefix, $prefix], [$suffix]));
                        }
                    }

                    kuru!(populate, te);
                    kind = Kind::Verb;
                    chau_stem = None;
                }
                PartOfSpeech::AdjectiveI => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'い') else {
                        allowlist!("弱っちぃ");
                        continue;
                    };

                    macro_rules! populate {
                        ($suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([k], [r], [$suffix]));
                        }
                    }

                    adjective_i!(populate);
                    kind = Kind::Adjective;
                    chau_stem = None;
                }
                PartOfSpeech::AdjectiveIx => {
                    let Some((kanji_stem, reading_prefix)) =
                        extract_stem(kanji_text, reading_text, 'い')
                    else {
                        allowlist!();
                        continue;
                    };

                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([kanji_stem], [reading_prefix, $prefix], [$suffix]));
                        }
                    }

                    adjective_ii!(populate);
                    kind = Kind::Adjective;
                    chau_stem = None;
                }
                PartOfSpeech::AdjectiveNa => {
                    macro_rules! populate {
                        ($suffix:expr $(, $inflect:ident)*) => {
                            insert!(($($inflect),*), Fragments::new([kanji_text], [reading_text], [$suffix]));
                        }
                    }

                    adjective_na!(populate);
                    kind = Kind::Adjective;
                    chau_stem = None;
                }
                _ => {
                    continue;
                }
            };

            if let Some(te) = inflections.get(&inflect!(Te)).cloned() {
                macro_rules! populate {
                    ($suffix:expr $(, $inflect:ident)*) => {
                        insert!((TeIru, Te $(, $inflect)*), te.concat([concat!("い", $suffix)]));
                    }
                }

                insert!((TeIru, Te, Short), te.concat(["る"]));
                ichidan!(populate);

                macro_rules! populate {
                    ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                        insert!((TeAru, Te $(, $inflect)*), te.concat([concat!("あ", $prefix, $suffix)]));
                    }
                }

                godan_ru!(populate);

                macro_rules! populate {
                    ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                        insert!((TeIku, Te $(, $inflect)*), te.concat([concat!("い", $prefix, $suffix)]));
                    }
                }

                godan_iku!(populate);

                macro_rules! populate {
                    ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                        insert!((TeShimau, Te $(, $inflect)*), te.concat([concat!("しま", $prefix, $suffix)]));
                    }
                }

                godan_u!(populate);

                macro_rules! populate {
                    ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                        insert!((TeOku, Te $(, $inflect)*), te.concat([concat!("お", $prefix, $suffix)]));
                    }
                }

                godan_ku!(populate);
                insert!((Te, TeOku, Short), te.concat(["く"]));

                macro_rules! populate {
                    ($r:expr, $suffix:expr $(, $inflect:ident)*) => {
                        insert!((TeKuru, Te $(, $inflect)*), te.concat([concat!($r, $suffix)]));
                    }
                }

                kuru!(populate);
            }

            if let Some((stem, de)) = chau_stem {
                if de {
                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!((Chau $(, $inflect)*), stem.concat([concat!("じゃ", $prefix, $suffix)]));
                        }
                    }

                    godan_u!(populate);
                } else {
                    macro_rules! populate {
                        ($prefix:expr, $suffix:expr $(, $inflect:ident)*) => {
                            insert!((Chau $(, $inflect)*), stem.concat([concat!("ちゃ", $prefix, $suffix)]));
                        }
                    }

                    godan_u!(populate);
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

            output.push((reading, inflections, kind));
        }
    }

    output
}

fn extract_stem<'a>(
    kanji_text: &'a str,
    reading_text: &'a str,
    c: char,
) -> Option<(&'a str, &'a str)> {
    let mut k = kanji_text.char_indices();
    let mut r = reading_text.char_indices();

    let (k_e, _) = k.next_back()?;
    let (_, reading_char) = r.next_back()?;

    if reading_char != c {
        return None;
    }

    r.next_back();
    Some((&kanji_text[..k_e], r.as_str()))
}

fn match_char<'a>(
    kanji_text: &'a str,
    reading_text: &'a str,
    expected: char,
) -> Option<(&'a str, &'a str)> {
    let mut k = kanji_text.chars();
    let mut r = reading_text.chars();

    if k.next_back() != Some(expected) || r.next_back() != Some(expected) {
        return None;
    }

    Some((k.as_str(), r.as_str()))
}

pub(crate) fn reading_permutations<'a>(
    entry: &Entry<'a>,
) -> Vec<(Option<(usize, &'a str)>, (usize, &'a str))> {
    let mut readings = Vec::new();

    for (reading_index, reading) in entry.reading_elements.iter().enumerate() {
        if reading.is_search_only() {
            continue;
        }

        if reading.no_kanji || entry.kanji_elements.is_empty() {
            readings.push((None, (reading_index, reading.text)));
            continue;
        }

        for (kanji_index, kanji) in entry.kanji_elements.iter().enumerate() {
            if kanji.is_search_only() {
                continue;
            }

            if reading.applies_to(&kanji.text) {
                readings.push((
                    Some((kanji_index, kanji.text)),
                    (reading_index, reading.text),
                ));
            }
        }
    }

    readings
}

/// If the entry is a verb, figure out the verb kind.
pub(crate) fn parts_of_speech(entry: &Entry<'_>) -> Set<PartOfSpeech> {
    let mut pos = Set::new();

    for sense in &entry.senses {
        for p in sense.pos {
            pos.insert(p);
        }
    }

    pos
}
