use crate::elements::Entry;
use crate::inflection::Inflections;
use crate::kana::Full;
use crate::PartOfSpeech;

/// Try to conjugate the given entry as an adjective.
pub fn conjugate<'a>(entry: &Entry<'a>) -> Option<Inflections<'a>> {
    let (Some(kind), [kanji, ..], [reading, ..]) = (
        as_adjective_kind(entry),
        &entry.kanji_elements[..],
        &entry.reading_elements[..],
    ) else {
        return None;
    };

    let inflections = match kind {
        AdjectiveKind::I => {
            let (Some(k), Some(r)) = (
                kanji.text.strip_suffix('い'),
                reading.text.strip_suffix('い'),
            ) else {
                return None;
            };

            inflections! {
                k, r,
                ("い"),
                Polite ("いです"),
                Past ("かった"),
                Past, Polite ("かったです"),
                Negative ("くない"),
                Negative, Polite ("くないです"),
                Past, Negative ("なかった"),
                Past, Negative, Polite ("なかったです"),
            }
        }
        AdjectiveKind::Yoi => {
            let (Some(k), Some(r)) = (
                kanji.text.strip_suffix("いい"),
                reading.text.strip_suffix("いい"),
            ) else {
                return None;
            };

            inflections! {
                k, r,
                ("いい"),
                Polite ("いいです"),
                Past ("よかった"),
                Past, Polite ("よかったです"),
                Negative ("よくない"),
                Negative, Polite ("よくないです"),
                Past, Negative ("よなかった"),
                Past, Negative, Polite ("よなかったです"),
            }
        }
        AdjectiveKind::Na => {
            inflections! {
                kanji.text, reading.text,
                ("だ"),
                Polite ("です"),
                Past ("だった"),
                Past, Polite ("でした"),
                Negative ("ではない"),
                Negative, Polite ("ではありません"),
                Past, Negative ("ではなかった"),
                Past, Negative, Polite ("ではありませんでした"),
            }
        }
    };

    Some(Inflections {
        dictionary: Full::new(kanji.text, reading.text, ""),
        inflections,
    })
}

enum AdjectiveKind {
    /// An i-adjective.
    I,
    /// Special yoi / ii class.
    Yoi,
    /// Na-adjective.
    Na,
}

/// If the entry is an adjective, figure out the adjective kind.
fn as_adjective_kind(entry: &Entry<'_>) -> Option<AdjectiveKind> {
    for sense in &entry.senses {
        for pos in sense.pos.iter() {
            let kind = match pos {
                PartOfSpeech::AdjectiveI => AdjectiveKind::I,
                PartOfSpeech::AdjectiveIx => AdjectiveKind::Yoi,
                PartOfSpeech::AdjectiveNa => AdjectiveKind::Na,
                _ => continue,
            };

            return Some(kind);
        }
    }

    None
}
