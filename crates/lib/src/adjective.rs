use crate::elements::Entry;
use crate::inflection::Inflections;
use crate::kana::Full;
use crate::verb::Reading;
use crate::PartOfSpeech;

/// Try to conjugate the given entry as an adjective.
pub fn conjugate<'a>(entry: &Entry<'a>) -> Vec<(Reading, Inflections<'a>)> {
    let mut output = Vec::new();

    let readings = crate::verb::reading_permutations(entry);

    for pos in crate::verb::parts_of_speech(entry) {
        for &(kanji, reading) in &readings {
            let (_, kanji_text) = kanji.unwrap_or(reading);
            let (_, reading_text) = reading;

            let inflections = match pos {
                PartOfSpeech::AdjectiveI => {
                    let (Some(k), Some(r)) = (
                        kanji_text.strip_suffix('い'),
                        reading_text.strip_suffix('い'),
                    ) else {
                        continue;
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
                PartOfSpeech::AdjectiveIx => {
                    let (Some(k), Some(r)) = (
                        kanji_text.strip_suffix("いい"),
                        reading_text.strip_suffix("いい"),
                    ) else {
                        continue;
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
                PartOfSpeech::AdjectiveNa => {
                    inflections! {
                        kanji_text, reading_text,
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
                _ => continue,
            };

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
    }

    output
}
