//! Module which performs verb inflection, based on a words class.

use fixed_map::Set;
use musli::mode::{Binary, Text};
use musli::{Decode, Encode};
use musli_zerocopy::ZeroCopy;
use serde::{Deserialize, Serialize};

use crate::inflection::godan;
use crate::inflection::macros;
use crate::inflection::Inflections;
use crate::jmdict::Entry;
use crate::kana::{Fragments, Full};
use crate::PartOfSpeech;

use crate::inflection::Form;
use Form::*;

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
#[musli(mode = Binary, bound = {T: Encode<Binary>}, decode_bound<'de, A> = {T: Decode<'de, Binary, A>})]
#[musli(mode = Text, bound = {T: Encode<Text>}, decode_bound<'de, A> = {T: Decode<'de, Text, A>})]
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
#[repr(C, packed)]
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

    for &(kanji, reading, pos) in &readings {
        for pos in pos.iter() {
            let (_, kanji_text) = kanji.unwrap_or(reading);
            let (_, reading_text) = reading;

            let mut inflections = Inflections::new(Full::new(kanji_text, reading_text, ""));

            let kind;
            let chau_stem: Option<Fragments<'_>>;

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

                    macros::ichidan_te(|suffix, inflect| {
                        inflections.insert(inflect, &[], Fragments::new([k], [r], [suffix]));
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["っちゃ"]));
                }
                PartOfSpeech::VerbGodanKS => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'く') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::IKU, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["っちゃ"]));
                }
                PartOfSpeech::VerbGodanUS => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'う') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::US, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["うちゃ"]));
                }
                PartOfSpeech::VerbGodanU => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'う') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::U, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["っちゃ"]));
                }
                PartOfSpeech::VerbGodanT => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'つ') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::TSU, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["っちゃ"]));
                }
                PartOfSpeech::VerbGodanR
                | PartOfSpeech::VerbGodanRI
                | PartOfSpeech::VerbGodanAru
                | PartOfSpeech::VerbGodanUru => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'る') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::RU, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["っちゃ"]));
                }
                PartOfSpeech::VerbGodanK => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'く') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::KU, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["いちゃ"]));
                }
                PartOfSpeech::VerbGodanG => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'ぐ') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::GU, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["いじゃ"]));
                }
                PartOfSpeech::VerbGodanM => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'む') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::MU, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["んじゃ"]));
                }
                PartOfSpeech::VerbGodanB => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'ぶ') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::BU, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["んじゃ"]));
                }
                PartOfSpeech::VerbGodanN => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'ぬ') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::NU, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["んじゃ"]));
                }
                PartOfSpeech::VerbGodanS => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'す') else {
                        allowlist!();
                        continue;
                    };

                    macros::godan_base(godan::SU, |prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([k], [r], [prefix, suffix]),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([k], [r], ["しちゃ"]));
                }
                PartOfSpeech::VerbSuruSpecial | PartOfSpeech::VerbSuruIncluded => {
                    let Some((mode, kanji_stem, reading_stem)) =
                        extract_suru(kanji_text, reading_text)
                    else {
                        allowlist!();
                        continue;
                    };

                    macros::suru_base(|prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new(
                                [kanji_stem, mode.apply(prefix)],
                                [reading_stem, prefix],
                                [suffix],
                            ),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = Some(Fragments::new([kanji_stem], [reading_stem], ["しちゃ"]));
                }
                PartOfSpeech::VerbKuru => {
                    let Some((mode, kanji_stem, reading_prefix)) =
                        extract_kuru(kanji_text, reading_text)
                    else {
                        allowlist!();
                        continue;
                    };

                    macros::kuru_base(|prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new(
                                [kanji_stem, mode.apply(prefix)],
                                [reading_prefix, prefix],
                                [suffix],
                            ),
                        );
                    });

                    kind = Kind::Verb;
                    chau_stem = None;
                }
                PartOfSpeech::AdjectiveI => {
                    let Some((k, r)) = match_char(kanji_text, reading_text, 'い') else {
                        allowlist!("弱っちぃ");
                        continue;
                    };

                    macros::adjective_i(|suffix, inflect| {
                        inflections.insert(inflect, &[], Fragments::new([k], [r], [suffix]));
                    });

                    kind = Kind::Adjective;
                    chau_stem = None;
                }
                PartOfSpeech::AdjectiveIx => {
                    let Some((mode, kanji_stem, reading_prefix)) =
                        extract_ii(kanji_text, reading_text)
                    else {
                        allowlist!();
                        continue;
                    };

                    macros::adjective_ii(|prefix, suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new(
                                [kanji_stem, mode.apply(prefix)],
                                [reading_prefix, prefix],
                                [suffix],
                            ),
                        );
                    });

                    kind = Kind::Adjective;
                    chau_stem = None;
                }
                PartOfSpeech::AdjectiveNa => {
                    macros::adjective_na(|suffix, inflect| {
                        inflections.insert(
                            inflect,
                            &[],
                            Fragments::new([kanji_text], [reading_text], [suffix]),
                        );
                    });

                    kind = Kind::Adjective;
                    chau_stem = None;
                }
                _ => {
                    continue;
                }
            };

            if let Some(stem) = inflections.get(inflect!(Stem)).cloned() {
                macros::godan(godan::RU, |prefix, suffix, inflect| {
                    inflections.insert(inflect, &[TaGaRu], stem.concat(["たが", prefix, suffix]));
                });

                macros::adjective_i(|suffix, inflect| {
                    inflections.insert(inflect, &[Tai], stem.concat(["た", suffix]));
                    inflections.insert(inflect, &[EasyTo], stem.concat(["やす", suffix]));
                    inflections.insert(inflect, &[HardTo], stem.concat(["にく", suffix]));
                });
            }

            if let Some(te) = inflections.get(inflect!(Te)).cloned() {
                inflections.insert(&[TeIru, Te, Short], &[], te.concat(["る"]));
                inflections.insert(&[TeIru, Te, Short, Honorific], &[], te.concat(["ます"]));
                inflections.insert(&[TeIru, Te, Past, Short], &[], te.concat(["た"]));

                macros::ichidan(|suffix, inflect| {
                    inflections.insert(inflect, &[TeIru, Te], te.concat(["い", suffix]))
                });

                macros::godan(godan::RU, |prefix, suffix, inflect| {
                    inflections.insert(inflect, &[TeAru, Te], te.concat(["あ", prefix, suffix]));
                });

                macros::godan(godan::IKU, |prefix, suffix, inflect| {
                    inflections.insert(inflect, &[TeIku, Te], te.concat(["い", prefix, suffix]));
                });

                macros::godan(godan::U, |prefix, suffix, inflect| {
                    inflections.insert(
                        inflect,
                        &[TeShimau, Te],
                        te.concat(["しま", prefix, suffix]),
                    );
                });

                macros::godan(godan::KU, |prefix, suffix, inflect| {
                    inflections.insert(inflect, &[TeOku, Te], te.concat(["お", prefix, suffix]));
                });

                inflections.insert(&[Te, TeOku, Short], &[], te.concat(["く"]));

                macros::kuru(|prefix, suffix, inflect| {
                    inflections.insert(inflect, &[TeKuru, Te], te.concat([prefix, suffix]));
                });
            }

            if let Some(stem) = chau_stem {
                macros::godan(godan::U, |prefix, suffix, inflect| {
                    inflections.insert(inflect, &[Chau], stem.concat([prefix, suffix]));
                });
            }

            if let Some(kanji) = kanji {
                if kanji.0 >= u8::MAX as usize {
                    log::warn!("Kanji index too large: {}", kanji.0);
                    continue;
                }
            }

            if reading.0 >= u8::MAX as usize {
                log::warn!("Reading index too large: {}", reading.0);
                continue;
            }

            let reading = Reading {
                kanji: kanji.map(|(i, _)| i as u8).unwrap_or(u8::MAX),
                reading: reading.0 as u8,
            };

            output.push((reading, inflections, kind));
        }
    }

    output
}

fn s(s: &str) -> Option<(char, char, &str)> {
    let mut it = s.chars();
    let c1 = it.next_back()?;
    let c2 = it.next_back()?;
    Some((c2, c1, it.as_str()))
}

fn s2(s: &str) -> Option<(char, char, &str, &str)> {
    let mut it = s.chars();
    let c1 = it.next_back()?;
    let s1 = it.as_str();
    let c2 = it.next_back()?;
    Some((c2, c1, s1, it.as_str()))
}

/// Whether or not a kanji text should have a conjugation suffix added to it or
/// not. If set to excluded, this implies that the kanji itself represents the
/// conjugation suffix.
#[derive(Debug, Clone, Copy)]
enum SuffixMode {
    /// Exclude suffix.
    Excluded,
    /// Include suffix.
    Included,
}

impl SuffixMode {
    fn apply(self, prefix: &str) -> &str {
        match self {
            SuffixMode::Excluded => "",
            SuffixMode::Included => prefix,
        }
    }
}

fn extract_ii<'a>(kanji: &'a str, reading: &'a str) -> Option<(SuffixMode, &'a str, &'a str)> {
    let ('い' | 'よ', 'い', r) = s(reading)? else {
        return None;
    };

    match s2(kanji)? {
        ('好' | '良', 'い', k, _) => Some((SuffixMode::Excluded, k, r)),
        ('い' | 'よ', 'い', _, k) => Some((SuffixMode::Included, k, r)),
        _ => None,
    }
}

fn extract_suru<'a>(kanji: &'a str, reading: &'a str) -> Option<(SuffixMode, &'a str, &'a str)> {
    let ('す', 'る', r) = s(reading)? else {
        return None;
    };

    match s2(kanji)? {
        ('為', 'る', k, _) => Some((SuffixMode::Excluded, k, r)),
        ('す', 'る', _, k) => Some((SuffixMode::Included, k, r)),
        _ => None,
    }
}

fn extract_kuru<'a>(kanji: &'a str, reading: &'a str) -> Option<(SuffixMode, &'a str, &'a str)> {
    let ('く', 'る', r) = s(reading)? else {
        return None;
    };

    match s2(kanji)? {
        ('来' | '來', 'る', k, _) => Some((SuffixMode::Excluded, k, r)),
        ('く', 'る', _, k) => Some((SuffixMode::Included, k, r)),
        _ => None,
    }
}

fn match_char<'a>(
    kanji_text: &'a str,
    reading_text: &'a str,
    suffix: char,
) -> Option<(&'a str, &'a str)> {
    Some((
        kanji_text.strip_suffix(suffix)?,
        reading_text.strip_suffix(suffix)?,
    ))
}

/// Get all reading permutations.
pub fn reading_permutations<'a>(
    entry: &Entry<'a>,
) -> Vec<(
    Option<(usize, &'a str)>,
    (usize, &'a str),
    Set<PartOfSpeech>,
)> {
    let mut readings = Vec::new();

    for (reading_index, reading) in entry.reading_elements.iter().enumerate() {
        if reading.is_search_only() {
            continue;
        }

        if reading.no_kanji || entry.kanji_elements.is_empty() {
            let mut pos = Set::new();

            for sense in &entry.senses {
                if !sense.applies_to(None, reading.text) {
                    continue;
                }

                for p in sense.pos.iter() {
                    pos.insert(p);
                }
            }

            readings.push((
                None,
                (reading_index, reading.text),
                build_pos(entry, None, reading.text),
            ));
            continue;
        }

        for (kanji_index, kanji) in entry.kanji_elements.iter().enumerate() {
            if kanji.is_search_only() {
                continue;
            }

            if !reading.applies_to(kanji.text) {
                continue;
            }

            readings.push((
                Some((kanji_index, kanji.text)),
                (reading_index, reading.text),
                build_pos(entry, Some(kanji.text), reading.text),
            ));
        }
    }

    readings
}

fn build_pos(entry: &Entry<'_>, kanji: Option<&str>, reading: &str) -> Set<PartOfSpeech> {
    let mut pos = Set::new();

    for sense in &entry.senses {
        if !sense.applies_to(kanji, reading) {
            continue;
        }

        for p in sense.pos.iter() {
            pos.insert(p);
        }
    }

    pos
}
