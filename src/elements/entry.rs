use core::fmt;
use core::mem;
use std::collections::BTreeMap;

use anyhow::{Context, Result};

use crate::composite::{comp, Composite};
use crate::elements::{kanji_element, reading_element, sense, text};
use crate::elements::{KanjiElement, ReadingElement, Sense};
use crate::entities::{KanjiInfo, PartOfSpeech};
use crate::parser::{Output, Poll};

pub struct Word<'a> {
    /// Verb stem.
    pub text: &'a str,
    /// Furigana reading of verb stem.
    pub reading: &'a str,
}

/// A reading pair.
#[derive(Clone)]
pub struct Pair<'a> {
    pub kanji: Composite<'a>,
    pub reading: Composite<'a>,
}

/// Construct a kanji/reading pair.
fn pair<'a, A, B>(kanji: A, reading: B) -> Pair<'a>
where
    A: IntoIterator<Item = &'a str>,
    B: IntoIterator<Item = &'a str>,
{
    Pair {
        kanji: comp(kanji),
        reading: comp(reading),
    }
}

impl<'a> IntoIterator for Pair<'a> {
    type Item = Composite<'a>;
    type IntoIter = std::array::IntoIter<Composite<'a>, 2>;

    #[inline]
    fn into_iter(self) -> Self::IntoIter {
        [self.kanji, self.reading].into_iter()
    }
}

impl fmt::Display for Pair<'_> {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.kanji != self.reading {
            write!(f, "{} ({})", self.kanji, self.reading)
        } else {
            self.kanji.fmt(f)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Polite {
    Plain,
    Polite,
}

impl fmt::Display for Polite {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Polite::Plain => write!(f, "Plain"),
            Polite::Polite => write!(f, "Polite"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Conjugation {
    Causative,
    Command,
    CommandAlt,
    Conditional,
    Hypothetical,
    Indicative,
    Negative,
    Passive,
    Past,
    PastNegative,
    Potential,
    PotentialAlt,
    Tai,
    Te,
    Volitional,
}

pub struct VerbConjugations<'a> {
    pub dictionary: Word<'a>,
    pub plain: BTreeMap<Conjugation, Pair<'a>>,
    pub polite: BTreeMap<Conjugation, Pair<'a>>,
}

impl<'a> VerbConjugations<'a> {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (Polite, Conjugation, Composite<'a>)> + '_ {
        let plain = self
            .plain
            .iter()
            .flat_map(|(k, p)| p.clone().into_iter().map(|p| (Polite::Plain, *k, p)));

        let polite = self
            .polite
            .iter()
            .flat_map(|(k, p)| p.clone().into_iter().map(|p| (Polite::Polite, *k, p)));

        plain.chain(polite)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum VerbKind {
    /// Ichidan verb.
    Ichidan,
    /// Godan verb.
    Godan,
    /// Special godan verb.
    GodanSpecial,
    /// Suru irregular suru verb.
    Suru,
    /// Special irregular kuru verb.
    Kuru,
}

#[derive(Debug)]
pub struct Entry<'a> {
    pub sequence: &'a str,
    pub reading_elements: Vec<ReadingElement<'a>>,
    pub kanji_elements: Vec<KanjiElement<'a>>,
    pub senses: Vec<Sense<'a>>,
}

impl<'a> Entry<'a> {
    /// If the entry is a verb, figure out the verb kind.
    pub(crate) fn as_verb_kind(&self) -> Option<VerbKind> {
        for sense in &self.senses {
            for pos in sense.part_of_speech.iter() {
                let kind = match pos {
                    PartOfSpeech::AdjectiveF => continue,
                    PartOfSpeech::AdjectiveI => continue,
                    PartOfSpeech::AdjectiveIx => continue,
                    PartOfSpeech::AdjectiveKari => continue,
                    PartOfSpeech::AdjectiveKu => continue,
                    PartOfSpeech::AdjectiveNa => continue,
                    PartOfSpeech::AdjectiveNari => continue,
                    PartOfSpeech::AdjectiveNo => continue,
                    PartOfSpeech::AdjectivePn => continue,
                    PartOfSpeech::AdjectiveShiku => continue,
                    PartOfSpeech::AdjectiveT => continue,
                    PartOfSpeech::Adverb => continue,
                    PartOfSpeech::AdverbTo => continue,
                    PartOfSpeech::Auxiliary => continue,
                    PartOfSpeech::AuxiliaryAdjective => continue,
                    PartOfSpeech::AuxiliaryVerb => continue,
                    PartOfSpeech::Conjunction => continue,
                    PartOfSpeech::Copular => continue,
                    PartOfSpeech::Counter => continue,
                    PartOfSpeech::Expression => continue,
                    PartOfSpeech::Interjection => continue,
                    PartOfSpeech::Noun => continue,
                    PartOfSpeech::NounAdverbial => continue,
                    PartOfSpeech::NounProper => continue,
                    PartOfSpeech::NounPrefix => continue,
                    PartOfSpeech::NounSuffix => continue,
                    PartOfSpeech::NounTemporal => continue,
                    PartOfSpeech::Numeric => continue,
                    PartOfSpeech::Pronoun => continue,
                    PartOfSpeech::Prefix => continue,
                    PartOfSpeech::Particle => continue,
                    PartOfSpeech::Suffix => continue,
                    PartOfSpeech::Unclassified => continue,
                    PartOfSpeech::VerbUnspecified => continue,
                    PartOfSpeech::VerbIchidan => VerbKind::Ichidan,
                    PartOfSpeech::VerbIchidanS => VerbKind::Ichidan,
                    PartOfSpeech::VerbNidanAS => continue,
                    PartOfSpeech::VerbNidanBK => continue,
                    PartOfSpeech::VerbNidanBS => continue,
                    PartOfSpeech::VerbNidanDK => continue,
                    PartOfSpeech::VerbNidanDS => continue,
                    PartOfSpeech::VerbNidanGK => continue,
                    PartOfSpeech::VerbNidanGS => continue,
                    PartOfSpeech::VerbNidanHK => continue,
                    PartOfSpeech::VerbNidanHS => continue,
                    PartOfSpeech::VerbNidanKK => continue,
                    PartOfSpeech::VerbNidanKS => continue,
                    PartOfSpeech::VerbNidanMK => continue,
                    PartOfSpeech::VerbNidanMS => continue,
                    PartOfSpeech::VerbNidanNS => continue,
                    PartOfSpeech::VerbNidanRK => continue,
                    PartOfSpeech::VerbNidanRS => continue,
                    PartOfSpeech::VerbNidanSS => continue,
                    PartOfSpeech::VerbNidanTK => continue,
                    PartOfSpeech::VerbNidanTS => continue,
                    PartOfSpeech::VerbNidanWS => continue,
                    PartOfSpeech::VerbNidanYK => continue,
                    PartOfSpeech::VerbNidanYS => continue,
                    PartOfSpeech::VerbNidanZS => continue,
                    PartOfSpeech::VerbYodanB => continue,
                    PartOfSpeech::VerbYodanG => continue,
                    PartOfSpeech::VerbYodanH => continue,
                    PartOfSpeech::VerbYodanK => continue,
                    PartOfSpeech::VerbYodanM => continue,
                    PartOfSpeech::VerbYodanN => continue,
                    PartOfSpeech::VerbYodanR => continue,
                    PartOfSpeech::VerbYodanS => continue,
                    PartOfSpeech::VerbYodanT => continue,
                    PartOfSpeech::VerbGodanAru => VerbKind::Godan,
                    PartOfSpeech::VerbGodanB => VerbKind::Godan,
                    PartOfSpeech::VerbGodanG => VerbKind::Godan,
                    PartOfSpeech::VerbGodanK => VerbKind::Godan,
                    PartOfSpeech::VerbGodanKS => VerbKind::GodanSpecial,
                    PartOfSpeech::VerbGodanM => VerbKind::Godan,
                    PartOfSpeech::VerbGodanN => VerbKind::Godan,
                    PartOfSpeech::VerbGodanR => VerbKind::Godan,
                    PartOfSpeech::VerbGodanRI => VerbKind::Godan,
                    PartOfSpeech::VerbGodanS => VerbKind::Godan,
                    PartOfSpeech::VerbGodanT => VerbKind::Godan,
                    PartOfSpeech::VerbGodanU => VerbKind::Godan,
                    PartOfSpeech::VerbGodanUS => VerbKind::Godan,
                    PartOfSpeech::VerbGodanUru => VerbKind::Godan,
                    PartOfSpeech::VerbIntransitive => continue,
                    PartOfSpeech::VerbKuru => VerbKind::Kuru,
                    PartOfSpeech::VerbNu => continue,
                    PartOfSpeech::VerbRu => continue,
                    PartOfSpeech::VerbSuru => continue,
                    PartOfSpeech::VerbSuC => continue,
                    PartOfSpeech::VerbSuruIncluded => VerbKind::Suru,
                    PartOfSpeech::VerbSuruSpecial => continue,
                    PartOfSpeech::VerbTransitive => continue,
                    PartOfSpeech::VerbZuru => continue,
                };

                return Some(kind);
            }
        }

        None
    }

    pub(crate) fn as_verb_conjugation(&self) -> Option<VerbConjugations<'a>> {
        macro_rules! pair {
            ($k:expr, $r:expr, [$($same:expr),* $(,)?]) => {
                pair([$k, $($same),*], [$r, $($same),*])
            };

            ($k:expr, $r:expr, [$($kanji:expr),* $(,)?], [$($reading:expr),* $(,)?]) => {
                pair([$k, $($kanji),*], [$r, $($reading),*])
            };

            ($k:expr, $r:expr, $($same:expr),* $(,)?) => {
                pair([$k, $($same),*], [$r, $($same),*])
            };
        }

        macro_rules! conjugations {
            ($k:expr, $r:expr, $(
                $kind:ident ( $($tt:tt)* )
            ),* $(,)?) => {{
                let mut tree = BTreeMap::new();
                $(tree.insert(Conjugation::$kind, pair!($k, $r, $($tt)*));)*
                tree
            }};
        }

        let (Some(kind), [kanji, ..], [reading, ..]) = (self.as_verb_kind(), &self.kanji_elements[..], &self.reading_elements[..]) else {
            return None;
        };

        let kanji_text = if kanji.info.contains(KanjiInfo::RareKanji) {
            reading.text
        } else {
            kanji.text
        };

        match kind {
            VerbKind::Ichidan => {
                let mut k = kanji_text.chars();
                let mut r = reading.text.chars();

                let (Some('る'), Some('る')) = (k.next_back(), r.next_back()) else {
                    return None;
                };

                let k = k.as_str();
                let r = r.as_str();

                let plain = conjugations! {
                    k, r,
                    Causative("させる"),
                    Command("ろ"),
                    CommandAlt("よ"),
                    Conditional("たら"),
                    Hypothetical("ば"),
                    Negative("ない"),
                    Passive("られる"),
                    Past("た"),
                    PastNegative("なかった"),
                    Potential("られる"),
                    PotentialAlt("れる"),
                    Tai("たい"),
                    Te("て"),
                    Volitional("よう"),
                };

                let polite = conjugations! {
                    k, r,
                    Indicative("ます"),
                    Negative("ません"),
                    Past("ました"),
                    PastNegative("ませんでした"),
                };

                Some(VerbConjugations {
                    dictionary: Word {
                        text: kanji_text,
                        reading: reading.text,
                    },
                    plain,
                    polite,
                })
            }
            VerbKind::Godan | VerbKind::GodanSpecial => {
                let mut k = kanji_text.chars();
                let mut r = reading.text.chars();

                let ([a, i, e, o], te, past) = match k.next_back() {
                    Some('う') => (["わ", "い", "え", "お"], "って", "った"),
                    Some('つ') => (["た", "ち", "て", "と"], "って", "った"),
                    Some('る') => (["ら", "り", "れ", "ろ"], "って", "った"),
                    Some('く') => (["か", "き", "け", "こ"], "いて", "いた"),
                    Some('ぐ') => (["が", "ぎ", "げ", "ご"], "いで", "いだ"),
                    Some('む') => (["ま", "み", "め", "も"], "んで", "んだ"),
                    Some('ぶ') => (["ば", "び", "べ", "ぼ"], "んで", "んだ"),
                    Some('ぬ') => (["な", "に", "ね", "の"], "んで", "んだ"),
                    Some('す') => (["さ", "し", "せ", "そ"], "して", "した"),
                    _ => return None,
                };

                // Special te-conjugation.
                let (te, past) = match kind {
                    VerbKind::GodanSpecial => ("って", "った"),
                    _ => (te, past),
                };

                r.next_back();

                let k = k.as_str();
                let r = r.as_str();

                let plain = conjugations! {
                    k, r,
                    Causative(a, "せる"),
                    Command(e),
                    Conditional(past, "ら"),
                    Hypothetical(e, "ば"),
                    Negative(a, "ない"),
                    Passive(a, "れる"),
                    Past(past),
                    PastNegative(a, "なかった"),
                    Potential(e, "る"),
                    Tai(i, "たい"),
                    Te(te),
                    Volitional(o, "う"),
                };

                let polite = conjugations! {
                    k, r,
                    Indicative(i, "ます"),
                    Negative(i, "ません"),
                    Past(i, "ました"),
                    PastNegative(i, "ませんでした"),
                };

                Some(VerbConjugations {
                    dictionary: Word {
                        text: kanji_text,
                        reading: reading.text,
                    },
                    plain,
                    polite,
                })
            }
            VerbKind::Suru => {
                let (Some(k), Some(r)) = (kanji_text.strip_suffix("する"), reading.text.strip_suffix("する")) else {
                    return None;
                };

                let plain = conjugations! {
                    k, r,
                    Causative("させる"),
                    Command("しろ"),
                    Conditional("したら"),
                    Hypothetical("すれば"),
                    Negative("しない"),
                    Passive("される"),
                    Past("した"),
                    PastNegative("しなかった"),
                    Potential("できる"),
                    Tai("したい"),
                    Te("して"),
                    Volitional("しよう"),
                };

                let polite = conjugations! {
                    k, r,
                    Indicative("します"),
                    Negative("しません"),
                    Past("しました"),
                    PastNegative("しませんでした"),
                };

                Some(VerbConjugations {
                    dictionary: Word {
                        text: kanji_text,
                        reading: reading.text,
                    },
                    plain,
                    polite,
                })
            }
            VerbKind::Kuru => {
                let (Some(k), Some(r)) = (kanji_text.strip_suffix("来る"), reading.text.strip_suffix("くる")) else {
                    return None;
                };

                let plain = conjugations! {
                    k, r,
                    Causative(["来させる"], ["こさせる"]),
                    Command(["来い"], ["こい"]),
                    Conditional(["来たら"], ["きたら"]),
                    Hypothetical(["来れば"], ["くれば"]),
                    Negative(["来ない"], ["こない"]),
                    Passive(["来られる"], ["こられる"]),
                    Past(["来た"], ["きた"]),
                    PastNegative(["来なかった"], ["こなかった"]),
                    Potential(["来られる"], ["こられる"]),
                    Tai(["来たい"], ["きたい"]),
                    Te(["来て"], ["きて"]),
                    Volitional(["来よう"], ["こよう"]),
                };

                let polite = conjugations! {
                    k, r,
                    Indicative(["来ます"], ["きます"]),
                    Negative(["来ません"], ["きません"]),
                    Past(["来ました"], ["きました"]),
                    PastNegative(["来ませんでした"], ["きませんでした"]),
                };

                Some(VerbConjugations {
                    dictionary: Word {
                        text: kanji_text,
                        reading: reading.text,
                    },
                    plain,
                    polite,
                })
            }
        }
    }
}

#[derive(Debug, Default)]
enum State<'a> {
    #[default]
    Root,
    EntrySequence(text::Builder<'a>),
    ReadingElement(reading_element::Builder<'a>),
    KanjiElement(kanji_element::Builder<'a>),
    Sense(sense::Builder<'a>),
}

#[derive(Default)]
pub(crate) struct Builder<'a> {
    state: State<'a>,
    sequence: Option<&'a str>,
    reading_elements: Vec<ReadingElement<'a>>,
    kanji_elements: Vec<KanjiElement<'a>>,
    senses: Vec<Sense<'a>>,
}

impl<'a> Builder<'a> {
    builder! {
        self => Entry<'a> {
            "ent_seq", EntrySequence, value => {
                self.sequence = Some(value);
            },
            "r_ele", ReadingElement, value => {
                self.reading_elements.push(value);
            },
            "k_ele", KanjiElement, value => {
                self.kanji_elements.push(value);
            },
            "sense", Sense, value => {
                self.senses.push(value);
            },
        }
    }

    /// Build an [`Entry`].
    fn build(&mut self) -> Result<Entry<'a>> {
        let sequence = self.sequence.take().context("missing entry sequence")?;
        let reading_elements = mem::take(&mut self.reading_elements);
        let kanji_elements = mem::take(&mut self.kanji_elements);
        let senses = mem::take(&mut self.senses);

        Ok(Entry {
            sequence,
            reading_elements,
            kanji_elements,
            senses,
        })
    }
}
