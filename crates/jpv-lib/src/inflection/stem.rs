//! Reverse inflection (stemming) for Japanese verbs and adjectives.

use std::collections::{BTreeSet, HashMap};
use std::sync::LazyLock;

use fixed_map::Set;

use crate::inflection::conjugate::{
    extract_ii, extract_kuru, extract_suru, match_char, reading_permutations, Kind, Reading,
};
use crate::inflection::godan::{self, Godan};
use crate::inflection::macros;
use crate::inflection::{Form, Inflection};
use crate::jmdict::Entry;
use crate::PartOfSpeech;

use Form::*;

/// Conjugable word class identified by the stemmer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WordClass {
    Ichidan,
    GodanKS,
    GodanUS,
    GodanU,
    GodanT,
    GodanR,
    GodanK,
    GodanG,
    GodanM,
    GodanB,
    GodanN,
    GodanS,
    Suru,
    Kuru,
    AdjectiveI,
    AdjectiveIx,
    AdjectiveNa,
}

impl WordClass {
    /// Return whether this word class is a verb or an adjective.
    pub fn kind(self) -> Kind {
        match self {
            WordClass::AdjectiveI | WordClass::AdjectiveIx | WordClass::AdjectiveNa => {
                Kind::Adjective
            }
            _ => Kind::Verb,
        }
    }

    /// Test whether a JMdict [`PartOfSpeech`] belongs to this word class.
    pub fn matches_pos(self, pos: PartOfSpeech) -> bool {
        match self {
            WordClass::Ichidan => {
                matches!(pos, PartOfSpeech::VerbIchidan | PartOfSpeech::VerbIchidanS)
            }
            WordClass::GodanKS => pos == PartOfSpeech::VerbGodanKS,
            WordClass::GodanUS => pos == PartOfSpeech::VerbGodanUS,
            WordClass::GodanU => pos == PartOfSpeech::VerbGodanU,
            WordClass::GodanT => pos == PartOfSpeech::VerbGodanT,
            WordClass::GodanR => matches!(
                pos,
                PartOfSpeech::VerbGodanR
                    | PartOfSpeech::VerbGodanRI
                    | PartOfSpeech::VerbGodanAru
                    | PartOfSpeech::VerbGodanUru
            ),
            WordClass::GodanK => pos == PartOfSpeech::VerbGodanK,
            WordClass::GodanG => pos == PartOfSpeech::VerbGodanG,
            WordClass::GodanM => pos == PartOfSpeech::VerbGodanM,
            WordClass::GodanB => pos == PartOfSpeech::VerbGodanB,
            WordClass::GodanN => pos == PartOfSpeech::VerbGodanN,
            WordClass::GodanS => pos == PartOfSpeech::VerbGodanS,
            WordClass::Suru => matches!(
                pos,
                PartOfSpeech::VerbSuruSpecial | PartOfSpeech::VerbSuruIncluded
            ),
            WordClass::Kuru => pos == PartOfSpeech::VerbKuru,
            WordClass::AdjectiveI => pos == PartOfSpeech::AdjectiveI,
            WordClass::AdjectiveIx => pos == PartOfSpeech::AdjectiveIx,
            WordClass::AdjectiveNa => pos == PartOfSpeech::AdjectiveNa,
        }
    }

    /// Return the set of JMdict [`PartOfSpeech`] values belonging to this class.
    pub fn pos_set(self) -> Set<PartOfSpeech> {
        let mut set = Set::new();

        match self {
            WordClass::Ichidan => {
                set.insert(PartOfSpeech::VerbIchidan);
                set.insert(PartOfSpeech::VerbIchidanS);
            }
            WordClass::GodanKS => {
                set.insert(PartOfSpeech::VerbGodanKS);
            }
            WordClass::GodanUS => {
                set.insert(PartOfSpeech::VerbGodanUS);
            }
            WordClass::GodanU => {
                set.insert(PartOfSpeech::VerbGodanU);
            }
            WordClass::GodanT => {
                set.insert(PartOfSpeech::VerbGodanT);
            }
            WordClass::GodanR => {
                set.insert(PartOfSpeech::VerbGodanR);
                set.insert(PartOfSpeech::VerbGodanRI);
                set.insert(PartOfSpeech::VerbGodanAru);
                set.insert(PartOfSpeech::VerbGodanUru);
            }
            WordClass::GodanK => {
                set.insert(PartOfSpeech::VerbGodanK);
            }
            WordClass::GodanG => {
                set.insert(PartOfSpeech::VerbGodanG);
            }
            WordClass::GodanM => {
                set.insert(PartOfSpeech::VerbGodanM);
            }
            WordClass::GodanB => {
                set.insert(PartOfSpeech::VerbGodanB);
            }
            WordClass::GodanN => {
                set.insert(PartOfSpeech::VerbGodanN);
            }
            WordClass::GodanS => {
                set.insert(PartOfSpeech::VerbGodanS);
            }
            WordClass::Suru => {
                set.insert(PartOfSpeech::VerbSuruSpecial);
                set.insert(PartOfSpeech::VerbSuruIncluded);
            }
            WordClass::Kuru => {
                set.insert(PartOfSpeech::VerbKuru);
            }
            WordClass::AdjectiveI => {
                set.insert(PartOfSpeech::AdjectiveI);
            }
            WordClass::AdjectiveIx => {
                set.insert(PartOfSpeech::AdjectiveIx);
            }
            WordClass::AdjectiveNa => {
                set.insert(PartOfSpeech::AdjectiveNa);
            }
        }

        set
    }

    /// Test whether a `(kanji_text, reading_text)` pair satisfies this class's
    /// dictionary-form constraints, matching [`conjugate`](super::conjugate).
    fn valid_pair(self, kanji_text: &str, reading_text: &str) -> bool {
        match self {
            WordClass::Ichidan | WordClass::GodanR => {
                match_char(kanji_text, reading_text, 'る').is_some()
            }
            WordClass::GodanKS | WordClass::GodanK => {
                match_char(kanji_text, reading_text, 'く').is_some()
            }
            WordClass::GodanUS | WordClass::GodanU => {
                match_char(kanji_text, reading_text, 'う').is_some()
            }
            WordClass::GodanT => match_char(kanji_text, reading_text, 'つ').is_some(),
            WordClass::GodanG => match_char(kanji_text, reading_text, 'ぐ').is_some(),
            WordClass::GodanM => match_char(kanji_text, reading_text, 'む').is_some(),
            WordClass::GodanB => match_char(kanji_text, reading_text, 'ぶ').is_some(),
            WordClass::GodanN => match_char(kanji_text, reading_text, 'ぬ').is_some(),
            WordClass::GodanS => match_char(kanji_text, reading_text, 'す').is_some(),
            WordClass::Suru => extract_suru(kanji_text, reading_text).is_some(),
            WordClass::Kuru => extract_kuru(kanji_text, reading_text).is_some(),
            WordClass::AdjectiveI => match_char(kanji_text, reading_text, 'い').is_some(),
            WordClass::AdjectiveIx => extract_ii(kanji_text, reading_text).is_some(),
            WordClass::AdjectiveNa => true,
        }
    }
}

/// A candidate dictionary form produced by stemming a surface string.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Stem {
    /// Candidate dictionary form (kanji+okurigana or kana).
    pub dictionary: String,
    /// Exact inflection forms identified.
    pub inflection: Inflection,
    /// Conjugable word class that produced this candidate.
    pub class: WordClass,
}

impl Stem {
    /// Test whether a JMdict [`PartOfSpeech`] is compatible with this stem.
    pub fn matches_pos(&self, pos: PartOfSpeech) -> bool {
        self.class.matches_pos(pos)
    }

    /// Match this stemmed candidate against a JMdict entry, returning every
    /// [`Reading`] whose kanji or reading form and part of speech match.
    pub fn match_entry(&self, entry: &Entry<'_>) -> Vec<Reading> {
        let readings = reading_permutations(entry);
        let mut out = Vec::new();

        for &(kanji, reading, pos) in &readings {
            if let Some(r) = match_permutation(self, kanji, reading, pos) {
                out.push(r);
            }
        }

        out
    }
}

fn match_permutation(
    stem: &Stem,
    kanji: Option<(usize, &str)>,
    reading: (usize, &str),
    pos: Set<PartOfSpeech>,
) -> Option<Reading> {
    if !pos.iter().any(|p| stem.class.matches_pos(p)) {
        return None;
    }

    let (_, kanji_text) = kanji.unwrap_or(reading);
    let (_, reading_text) = reading;

    if !stem.class.valid_pair(kanji_text, reading_text) {
        return None;
    }

    if kanji_text != stem.dictionary && reading_text != stem.dictionary {
        return None;
    }

    if let Some((i, _)) = kanji {
        if i >= u8::MAX as usize {
            return None;
        }
    }

    if reading.0 >= u8::MAX as usize {
        return None;
    }

    Some(Reading {
        kanji: kanji.map(|(i, _)| i as u8).unwrap_or(u8::MAX),
        reading: reading.0 as u8,
    })
}

/// Match a collection of stemmed candidates against a JMdict entry, returning
/// unique `(Reading, Inflection)` pairs.
pub fn match_stems<'a>(
    stems: impl IntoIterator<Item = &'a Stem>,
    entry: &Entry<'_>,
) -> Vec<(Reading, Inflection)> {
    let readings = reading_permutations(entry);
    let mut matches = BTreeSet::new();

    for stem in stems {
        for &(kanji, reading, pos) in &readings {
            if let Some(r) = match_permutation(stem, kanji, reading, pos) {
                matches.insert((r, stem.inflection));
            }
        }
    }

    matches.into_iter().collect()
}

/// Stem a surface string and match all candidates against a JMdict entry.
pub fn match_entry(surface: &str, entry: &Entry<'_>) -> Vec<(Reading, Inflection)> {
    let stems = stem(surface);
    match_stems(&stems, entry)
}

#[derive(Debug, Clone, Copy)]
enum BaseConstraint {
    NonEmpty,
    Any,
    EndsWithSuruKanji,
    EndsWithKuruKanji,
    EndsWithIiKanji,
}

impl BaseConstraint {
    fn check(self, base: &str) -> bool {
        match self {
            BaseConstraint::NonEmpty => !base.is_empty(),
            BaseConstraint::Any => true,
            BaseConstraint::EndsWithSuruKanji => base.ends_with('為'),
            BaseConstraint::EndsWithKuruKanji => base.ends_with('来') || base.ends_with('來'),
            BaseConstraint::EndsWithIiKanji => base.ends_with('良') || base.ends_with('好'),
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct DirectRule {
    constraint: BaseConstraint,
    dict_endings: &'static [&'static str],
    class: WordClass,
    inflection: Inflection,
}

struct SuffixTable<T> {
    map: HashMap<String, Vec<T>>,
    max_bytes: usize,
}

impl<T> SuffixTable<T> {
    fn new() -> Self {
        Self {
            map: HashMap::new(),
            max_bytes: 0,
        }
    }

    fn insert(&mut self, suffix: String, value: T) {
        self.max_bytes = self.max_bytes.max(suffix.len());
        self.map.entry(suffix).or_default().push(value);
    }
}

struct Tables {
    direct: SuffixTable<DirectRule>,
    stem_compounds: SuffixTable<Inflection>,
    te_compounds: SuffixTable<Inflection>,
    chau_compounds: SuffixTable<Inflection>,
}

const GODAN_CLASSES: &[(WordClass, &Godan, &str)] = &[
    (WordClass::GodanKS, godan::IKU, "っちゃ"),
    (WordClass::GodanUS, godan::US, "うちゃ"),
    (WordClass::GodanU, godan::U, "っちゃ"),
    (WordClass::GodanT, godan::TSU, "っちゃ"),
    (WordClass::GodanR, godan::RU, "っちゃ"),
    (WordClass::GodanK, godan::KU, "いちゃ"),
    (WordClass::GodanG, godan::GU, "いじゃ"),
    (WordClass::GodanM, godan::MU, "んじゃ"),
    (WordClass::GodanB, godan::BU, "んじゃ"),
    (WordClass::GodanN, godan::NU, "んじゃ"),
    (WordClass::GodanS, godan::SU, "しちゃ"),
];

fn make_inflection(a: &[Form], b: &[Form]) -> Inflection {
    let mut set = Set::new();

    for &f in a {
        set.insert(f);
    }

    for &f in b {
        set.insert(f);
    }

    Inflection::new(set)
}

static TABLES: LazyLock<Tables> = LazyLock::new(build_tables);

fn build_tables() -> Tables {
    let mut direct = SuffixTable::new();
    let mut stem_compounds = SuffixTable::new();
    let mut te_compounds = SuffixTable::new();
    let mut chau_compounds = SuffixTable::new();

    macros::ichidan_te(|suffix, forms| {
        direct.insert(
            suffix.to_owned(),
            DirectRule {
                constraint: BaseConstraint::NonEmpty,
                dict_endings: &["る"],
                class: WordClass::Ichidan,
                inflection: make_inflection(forms, &[]),
            },
        );
    });

    for &(class, g, _) in GODAN_CLASSES {
        let dict_endings: &'static [&'static str] = match g.u {
            "く" => &["く"],
            "う" => &["う"],
            "つ" => &["つ"],
            "る" => &["る"],
            "ぐ" => &["ぐ"],
            "む" => &["む"],
            "ぶ" => &["ぶ"],
            "ぬ" => &["ぬ"],
            "す" => &["す"],
            _ => unreachable!(),
        };

        macros::godan_base(g, |prefix, suffix, forms| {
            let mut full = String::with_capacity(prefix.len() + suffix.len());
            full.push_str(prefix);
            full.push_str(suffix);

            direct.insert(
                full,
                DirectRule {
                    constraint: BaseConstraint::NonEmpty,
                    dict_endings,
                    class,
                    inflection: make_inflection(forms, &[]),
                },
            );
        });
    }

    macros::suru_base(|prefix, suffix, forms| {
        let inflection = make_inflection(forms, &[]);

        let mut full = String::with_capacity(prefix.len() + suffix.len());
        full.push_str(prefix);
        full.push_str(suffix);

        direct.insert(
            full,
            DirectRule {
                constraint: BaseConstraint::Any,
                dict_endings: &["する"],
                class: WordClass::Suru,
                inflection,
            },
        );

        direct.insert(
            suffix.to_owned(),
            DirectRule {
                constraint: BaseConstraint::EndsWithSuruKanji,
                dict_endings: &["る"],
                class: WordClass::Suru,
                inflection,
            },
        );
    });

    macros::kuru_base(|prefix, suffix, forms| {
        let inflection = make_inflection(forms, &[]);

        let mut full = String::with_capacity(prefix.len() + suffix.len());
        full.push_str(prefix);
        full.push_str(suffix);

        direct.insert(
            full,
            DirectRule {
                constraint: BaseConstraint::Any,
                dict_endings: &["くる"],
                class: WordClass::Kuru,
                inflection,
            },
        );

        direct.insert(
            suffix.to_owned(),
            DirectRule {
                constraint: BaseConstraint::EndsWithKuruKanji,
                dict_endings: &["る"],
                class: WordClass::Kuru,
                inflection,
            },
        );
    });

    macros::adjective_i(|suffix, forms| {
        direct.insert(
            suffix.to_owned(),
            DirectRule {
                constraint: BaseConstraint::NonEmpty,
                dict_endings: &["い"],
                class: WordClass::AdjectiveI,
                inflection: make_inflection(forms, &[]),
            },
        );
    });

    macros::adjective_ii(|prefix, suffix, forms| {
        let inflection = make_inflection(forms, &[]);

        let mut full = String::with_capacity(prefix.len() + suffix.len());
        full.push_str(prefix);
        full.push_str(suffix);

        direct.insert(
            full,
            DirectRule {
                constraint: BaseConstraint::Any,
                dict_endings: &["いい", "よい"],
                class: WordClass::AdjectiveIx,
                inflection,
            },
        );

        direct.insert(
            suffix.to_owned(),
            DirectRule {
                constraint: BaseConstraint::EndsWithIiKanji,
                dict_endings: &["い"],
                class: WordClass::AdjectiveIx,
                inflection,
            },
        );
    });

    macros::adjective_na(|suffix, forms| {
        direct.insert(
            suffix.to_owned(),
            DirectRule {
                constraint: BaseConstraint::NonEmpty,
                dict_endings: &[""],
                class: WordClass::AdjectiveNa,
                inflection: make_inflection(forms, &[]),
            },
        );
    });

    // Stem compounds
    macros::godan(godan::RU, |prefix, suffix, forms| {
        stem_compounds.insert(
            format!("たが{prefix}{suffix}"),
            make_inflection(forms, &[TaGaRu]),
        );
    });

    macros::adjective_i(|suffix, forms| {
        stem_compounds.insert(format!("た{suffix}"), make_inflection(forms, &[Tai]));
        stem_compounds.insert(format!("やす{suffix}"), make_inflection(forms, &[EasyTo]));
        stem_compounds.insert(format!("にく{suffix}"), make_inflection(forms, &[HardTo]));
    });

    // Te compounds
    te_compounds.insert("る".to_owned(), make_inflection(&[TeIru, Te, Short], &[]));
    te_compounds.insert(
        "ます".to_owned(),
        make_inflection(&[TeIru, Te, Short, Honorific], &[]),
    );
    te_compounds.insert(
        "た".to_owned(),
        make_inflection(&[TeIru, Te, Past, Short], &[]),
    );

    macros::ichidan(|suffix, forms| {
        te_compounds.insert(format!("い{suffix}"), make_inflection(forms, &[TeIru, Te]));
    });

    macros::godan(godan::RU, |prefix, suffix, forms| {
        te_compounds.insert(
            format!("あ{prefix}{suffix}"),
            make_inflection(forms, &[TeAru, Te]),
        );
    });

    macros::godan(godan::IKU, |prefix, suffix, forms| {
        te_compounds.insert(
            format!("い{prefix}{suffix}"),
            make_inflection(forms, &[TeIku, Te]),
        );
    });

    macros::godan(godan::U, |prefix, suffix, forms| {
        te_compounds.insert(
            format!("しま{prefix}{suffix}"),
            make_inflection(forms, &[TeShimau, Te]),
        );
    });

    macros::godan(godan::KU, |prefix, suffix, forms| {
        te_compounds.insert(
            format!("お{prefix}{suffix}"),
            make_inflection(forms, &[TeOku, Te]),
        );
    });

    te_compounds.insert("く".to_owned(), make_inflection(&[Te, TeOku, Short], &[]));

    macros::kuru(|prefix, suffix, forms| {
        te_compounds.insert(
            format!("{prefix}{suffix}"),
            make_inflection(forms, &[TeKuru, Te]),
        );
    });

    // Chau compounds
    macros::godan(godan::U, |prefix, suffix, forms| {
        chau_compounds.insert(format!("{prefix}{suffix}"), make_inflection(forms, &[Chau]));
    });

    Tables {
        direct,
        stem_compounds,
        te_compounds,
        chau_compounds,
    }
}

fn suffix_splits(s: &str, max_bytes: usize) -> impl Iterator<Item = (&str, &str)> {
    let min_start = s.len().saturating_sub(max_bytes);
    s.char_indices()
        .map(|(i, _)| i)
        .chain(std::iter::once(s.len()))
        .filter(move |&i| i >= min_start)
        .map(move |i| (&s[..i], &s[i..]))
}

fn push_stem(
    out: &mut Vec<Stem>,
    base: &str,
    ending: &str,
    inflection: Inflection,
    class: WordClass,
) {
    let mut dictionary = String::with_capacity(base.len() + ending.len());
    dictionary.push_str(base);
    dictionary.push_str(ending);
    out.push(Stem {
        dictionary,
        inflection,
        class,
    });
}

fn expand_stem_base(stem_surface: &str, inflection: Inflection, out: &mut Vec<Stem>) {
    if !stem_surface.is_empty() {
        push_stem(out, stem_surface, "る", inflection, WordClass::Ichidan);
    }

    for &(class, g, _) in GODAN_CLASSES {
        if let Some(base) = stem_surface.strip_suffix(g.i) {
            if !base.is_empty() {
                push_stem(out, base, g.u, inflection, class);
            }
        }
    }

    if let Some(base) = stem_surface.strip_suffix('し') {
        push_stem(out, base, "する", inflection, WordClass::Suru);
    }

    if BaseConstraint::EndsWithSuruKanji.check(stem_surface) {
        push_stem(out, stem_surface, "る", inflection, WordClass::Suru);
    }

    if let Some(base) = stem_surface.strip_suffix('き') {
        push_stem(out, base, "くる", inflection, WordClass::Kuru);
    }

    if BaseConstraint::EndsWithKuruKanji.check(stem_surface) {
        push_stem(out, stem_surface, "る", inflection, WordClass::Kuru);
    }
}

fn expand_te_base(te_surface: &str, inflection: Inflection, out: &mut Vec<Stem>) {
    if let Some(base) = te_surface.strip_suffix('て') {
        if !base.is_empty() {
            push_stem(out, base, "る", inflection, WordClass::Ichidan);
        }

        if BaseConstraint::EndsWithSuruKanji.check(base) {
            push_stem(out, base, "る", inflection, WordClass::Suru);
        }

        if BaseConstraint::EndsWithKuruKanji.check(base) {
            push_stem(out, base, "る", inflection, WordClass::Kuru);
        }
    }

    for &(class, g, _) in GODAN_CLASSES {
        if let Some(base) = te_surface.strip_suffix(g.te) {
            if !base.is_empty() {
                push_stem(out, base, g.u, inflection, class);
            }
        }
    }

    if let Some(base) = te_surface.strip_suffix("して") {
        push_stem(out, base, "する", inflection, WordClass::Suru);
    }

    if let Some(base) = te_surface.strip_suffix("きて") {
        push_stem(out, base, "くる", inflection, WordClass::Kuru);
    }
}

fn expand_chau_base(chau_surface: &str, inflection: Inflection, out: &mut Vec<Stem>) {
    if let Some(base) = chau_surface.strip_suffix("っちゃ") {
        if !base.is_empty() {
            push_stem(out, base, "る", inflection, WordClass::Ichidan);
        }
    }

    for &(class, g, chau_ending) in GODAN_CLASSES {
        if let Some(base) = chau_surface.strip_suffix(chau_ending) {
            if !base.is_empty() {
                push_stem(out, base, g.u, inflection, class);
            }
        }
    }

    if let Some(base) = chau_surface.strip_suffix("しちゃ") {
        push_stem(out, base, "する", inflection, WordClass::Suru);

        if BaseConstraint::EndsWithSuruKanji.check(base) {
            push_stem(out, base, "る", inflection, WordClass::Suru);
        }
    }
}

/// Stem a Japanese surface string (kanji+okurigana or kana) into all candidate
/// dictionary forms, along with their exact [`Inflection`] and [`WordClass`].
pub fn stem(surface: &str) -> Vec<Stem> {
    if surface.is_empty() {
        return Vec::new();
    }

    let tables = &*TABLES;
    let mut out = Vec::new();

    for (base, suffix) in suffix_splits(surface, tables.direct.max_bytes) {
        if let Some(rules) = tables.direct.map.get(suffix) {
            for rule in rules {
                if !rule.constraint.check(base) {
                    continue;
                }

                for &ending in rule.dict_endings {
                    push_stem(&mut out, base, ending, rule.inflection, rule.class);
                }
            }
        }
    }

    for (stem_surface, suffix) in suffix_splits(surface, tables.stem_compounds.max_bytes) {
        if let Some(inflections) = tables.stem_compounds.map.get(suffix) {
            for &inflection in inflections {
                expand_stem_base(stem_surface, inflection, &mut out);
            }
        }
    }

    for (te_surface, suffix) in suffix_splits(surface, tables.te_compounds.max_bytes) {
        if let Some(inflections) = tables.te_compounds.map.get(suffix) {
            for &inflection in inflections {
                expand_te_base(te_surface, inflection, &mut out);
            }
        }
    }

    for (chau_surface, suffix) in suffix_splits(surface, tables.chau_compounds.max_bytes) {
        if let Some(inflections) = tables.chau_compounds.map.get(suffix) {
            for &inflection in inflections {
                expand_chau_base(chau_surface, inflection, &mut out);
            }
        }
    }

    out.sort();
    out.dedup();
    out
}

#[cfg(test)]
mod tests {
    use std::collections::{BTreeSet, HashSet};

    use fixed_map::Set;

    use crate::inflection::conjugate;
    use crate::jmdict::{Entry, KanjiElement, ReadingElement, Sense};
    use crate::PartOfSpeech;

    use super::{match_entry, stem, WordClass};

    fn make_entry<'a>(kanji: &[&'a str], readings: &[&'a str], pos: &[PartOfSpeech]) -> Entry<'a> {
        let mut pos_set = Set::new();

        for &p in pos {
            pos_set.insert(p);
        }

        Entry {
            sequence: 1,
            kanji_elements: kanji
                .iter()
                .map(|&text| KanjiElement {
                    text,
                    priority: Vec::new(),
                    info: Set::new(),
                })
                .collect(),
            reading_elements: readings
                .iter()
                .map(|&text| ReadingElement {
                    text,
                    no_kanji: false,
                    reading_string: HashSet::new(),
                    priority: Vec::new(),
                    info: Set::new(),
                })
                .collect(),
            senses: vec![Sense {
                pos: pos_set,
                stagk: Vec::new(),
                stagr: Vec::new(),
                xref: Vec::new(),
                antonym: Vec::new(),
                field: Set::new(),
                misc: Set::new(),
                info: None,
                dialect: Set::new(),
                gloss: Vec::new(),
                source_language: Vec::new(),
                examples: Vec::new(),
            }],
        }
    }

    fn assert_round_trip(entry: &Entry<'_>) {
        let conjugated = conjugate(entry);
        assert!(
            !conjugated.is_empty(),
            "expected conjugations for {:?}",
            entry
        );

        let mut total_checked = 0usize;

        for (reading, inflections, _) in &conjugated {
            assert!(
                !inflections.inflections.is_empty(),
                "expected non-empty inflections for {:?}",
                reading
            );

            for (&inflection, pair) in inflections.iter() {
                let kanji_surface = format!("{}{}", pair.text(), pair.suffix());
                let reading_surface = format!("{}{}", pair.reading(), pair.suffix());

                let mut surfaces = BTreeSet::new();
                surfaces.insert(kanji_surface);
                surfaces.insert(reading_surface);

                for surface in surfaces {
                    let matched = match_entry(&surface, entry);
                    assert!(
                        matched.contains(&(*reading, inflection)),
                        "surface `{surface}` did not stem back to ({reading:?}, {inflection:?}); got: {matched:?}"
                    );
                    total_checked += 1;
                }
            }
        }

        assert!(total_checked > 0);
    }

    #[test]
    fn round_trip_ichidan() {
        assert_round_trip(&make_entry(
            &["食べる"],
            &["たべる"],
            &[PartOfSpeech::VerbIchidan],
        ));
        assert_round_trip(&make_entry(&[], &["たべる"], &[PartOfSpeech::VerbIchidan]));
        assert_round_trip(&make_entry(
            &["呉れる"],
            &["くれる"],
            &[PartOfSpeech::VerbIchidanS],
        ));
    }

    #[test]
    fn round_trip_all_godan_classes() {
        let cases: &[(&[&str], &[&str], PartOfSpeech)] = &[
            (&["行く"], &["いく"], PartOfSpeech::VerbGodanKS),
            (&[], &["いく"], PartOfSpeech::VerbGodanKS),
            (&["問う"], &["とう"], PartOfSpeech::VerbGodanUS),
            (&[], &["とう"], PartOfSpeech::VerbGodanUS),
            (&["買う"], &["かう"], PartOfSpeech::VerbGodanU),
            (&["待つ"], &["まつ"], PartOfSpeech::VerbGodanT),
            (&["走る"], &["はしる"], PartOfSpeech::VerbGodanR),
            (&["有る"], &["ある"], PartOfSpeech::VerbGodanRI),
            (&["下さる"], &["くださる"], PartOfSpeech::VerbGodanAru),
            (&["得る"], &["うる"], PartOfSpeech::VerbGodanUru),
            (&["書く"], &["かく"], PartOfSpeech::VerbGodanK),
            (&["泳ぐ"], &["およぐ"], PartOfSpeech::VerbGodanG),
            (&["読む"], &["よむ"], PartOfSpeech::VerbGodanM),
            (&["遊ぶ"], &["あそぶ"], PartOfSpeech::VerbGodanB),
            (&["死ぬ"], &["しぬ"], PartOfSpeech::VerbGodanN),
            (&["話す"], &["はなす"], PartOfSpeech::VerbGodanS),
        ];

        for &(kanji, readings, pos) in cases {
            assert_round_trip(&make_entry(kanji, readings, &[pos]));
            if !kanji.is_empty() {
                assert_round_trip(&make_entry(&[], readings, &[pos]));
            }
        }
    }

    #[test]
    fn round_trip_irregular_verbs() {
        assert_round_trip(&make_entry(
            &[],
            &["する"],
            &[PartOfSpeech::VerbSuruSpecial],
        ));
        assert_round_trip(&make_entry(
            &["為る"],
            &["する"],
            &[PartOfSpeech::VerbSuruSpecial],
        ));
        assert_round_trip(&make_entry(
            &["勉強する"],
            &["べんきょうする"],
            &[PartOfSpeech::VerbSuruIncluded],
        ));
        assert_round_trip(&make_entry(
            &["来る", "來る"],
            &["くる"],
            &[PartOfSpeech::VerbKuru],
        ));
        assert_round_trip(&make_entry(&[], &["くる"], &[PartOfSpeech::VerbKuru]));
        assert_round_trip(&make_entry(
            &["持って来る"],
            &["もってくる"],
            &[PartOfSpeech::VerbKuru],
        ));
    }

    #[test]
    fn round_trip_adjectives() {
        assert_round_trip(&make_entry(
            &["高い"],
            &["たかい"],
            &[PartOfSpeech::AdjectiveI],
        ));
        assert_round_trip(&make_entry(&[], &["たかい"], &[PartOfSpeech::AdjectiveI]));

        assert_round_trip(&make_entry(
            &["良い", "好い"],
            &["いい", "よい"],
            &[PartOfSpeech::AdjectiveIx],
        ));
        assert_round_trip(&make_entry(&[], &["いい"], &[PartOfSpeech::AdjectiveIx]));
        assert_round_trip(&make_entry(&[], &["よい"], &[PartOfSpeech::AdjectiveIx]));
        assert_round_trip(&make_entry(
            &["格好良い"],
            &["かっこういい"],
            &[PartOfSpeech::AdjectiveIx],
        ));

        assert_round_trip(&make_entry(
            &["静か"],
            &["しずか"],
            &[PartOfSpeech::AdjectiveNa],
        ));
        assert_round_trip(&make_entry(&[], &["しずか"], &[PartOfSpeech::AdjectiveNa]));
    }

    #[test]
    fn negative_pos_and_bogus_stems() {
        // Non-matching POS: noun should never match conjugated forms.
        let noun = make_entry(&["食べる"], &["たべる"], &[PartOfSpeech::Noun]);
        assert!(match_entry("食べた", &noun).is_empty());
        assert!(match_entry("食べていた", &noun).is_empty());

        // Wrong verb class: ichidan conjugation on a godan-r verb.
        let godan_r = make_entry(&["走る"], &["はしる"], &[PartOfSpeech::VerbGodanR]);
        assert!(match_entry("走た", &godan_r).is_empty());
        assert!(match_entry("走ない", &godan_r).is_empty());
        assert!(!match_entry("走った", &godan_r).is_empty());

        // Godan-k vs godan-ks (行く): 行いた is not valid for VerbGodanKS.
        let iku = make_entry(&["行く"], &["いく"], &[PartOfSpeech::VerbGodanKS]);
        assert!(match_entry("行いた", &iku).is_empty());
        assert!(!match_entry("行った", &iku).is_empty());

        // Normal godan-k (書く): 書った is not valid for VerbGodanK.
        let kaku = make_entry(&["書く"], &["かく"], &[PartOfSpeech::VerbGodanK]);
        assert!(match_entry("書った", &kaku).is_empty());
        assert!(!match_entry("書いた", &kaku).is_empty());

        // Allowlisted non-matching ending (e.g. 買い増す tagged ichidan).
        let mismatch = make_entry(&["買い増す"], &["かいます"], &[PartOfSpeech::VerbIchidan]);
        assert!(match_entry("買い増した", &mismatch).is_empty());

        // Empty input produces no stems.
        assert!(stem("").is_empty());

        // Bogus empty-stem candidates are not emitted for regular classes.
        let ta_stems = stem("た");
        assert!(!ta_stems.iter().any(|s| s.dictionary == "る"));
        assert!(!ta_stems.iter().any(|s| s.dictionary.is_empty()));
        assert!(!ta_stems
            .iter()
            .any(|s| s.class == WordClass::Ichidan && s.inflection == crate::inflect!(Past)));

        // Bogus kanji-excluded forms do not match.
        let kuru = make_entry(&["来る"], &["くる"], &[PartOfSpeech::VerbKuru]);
        assert!(match_entry("来きた", &kuru).is_empty());

        let ii = make_entry(&["良い"], &["よい"], &[PartOfSpeech::AdjectiveIx]);
        assert!(match_entry("良いかった", &ii).is_empty());
        assert!(match_entry("よい", &ii).is_empty());

        let suru = make_entry(&["為る"], &["する"], &[PartOfSpeech::VerbSuruSpecial]);
        assert!(match_entry("為した", &suru).is_empty());
    }
}
