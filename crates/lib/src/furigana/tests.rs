use crate::furigana::FuriganaGroup;

use super::Furigana;

#[test]
fn test_mixed_furigana() {
    let furigana = Furigana::new("私はお金がない星", "わたしはおかねがないほし");
    assert_eq!(furigana.to_string(), "私[わたし]はお金[かね]がない星[ほし]");

    assert_eq!(
        furigana.iter().collect::<Vec<_>>(),
        &[
            FuriganaGroup::Kanji("私", "わたし"),
            FuriganaGroup::Kana("はお"),
            FuriganaGroup::Kanji("金", "かね"),
            FuriganaGroup::Kana("がない"),
            FuriganaGroup::Kanji("星", "ほし"),
        ]
    );
}

#[test]
fn test_heading_furigana() {
    let furigana = Furigana::new("お金がない星", "おかねがないほし");

    assert_eq!(
        furigana.iter().collect::<Vec<_>>(),
        &[
            FuriganaGroup::Kana("お"),
            FuriganaGroup::Kanji("金", "かね"),
            FuriganaGroup::Kana("がない"),
            FuriganaGroup::Kanji("星", "ほし"),
        ]
    );

    assert_eq!(furigana.to_string(), "お金[かね]がない星[ほし]");
}

#[test]
fn test_trailing_kana() {
    let furigana = Furigana::new("私はお金がない", "わたしはおかねがない");
    assert_eq!(furigana.to_string(), "私[わたし]はお金[かね]がない");

    assert_eq!(
        furigana.iter().collect::<Vec<_>>(),
        &[
            FuriganaGroup::Kanji("私", "わたし"),
            FuriganaGroup::Kana("はお"),
            FuriganaGroup::Kanji("金", "かね"),
            FuriganaGroup::Kana("がない"),
        ]
    );
}
