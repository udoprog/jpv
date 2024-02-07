use crate::furigana::FuriganaGroup;

use super::Furigana;

#[test]
fn test_mixed_furigana() {
    let furigana = Furigana::new("私はお金がない星", "わたしはおかねがないほし", "");
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
    let furigana = Furigana::new("お金がない星", "おかねがないほし", "");

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
    let furigana = Furigana::new("私はお金がない", "わたしはおかねがない", "");
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

#[test]
fn no_matching_hiragana() {
    let furigana = Furigana::new("十八禁", "じゅうはちきん", "");
    assert_eq!(furigana.to_string(), "十八禁[じゅうはちきん]");

    assert_eq!(
        furigana.iter().collect::<Vec<_>>(),
        &[FuriganaGroup::Kanji("十八禁", "じゅうはちきん")]
    );

    let furigana = Furigana::new("18禁", "じゅうはちきん", "");
    assert_eq!(furigana.to_string(), "18禁[じゅうはちきん]");

    assert_eq!(
        furigana.iter().collect::<Vec<_>>(),
        &[FuriganaGroup::Kanji("18禁", "じゅうはちきん")]
    );
}

#[test]
fn test_common_suffix() {
    // 見失う -> 見失[みうしな]う
    let furigana = Furigana::new("見失", "みうしな", "う");

    assert_eq!(
        furigana.iter().collect::<Vec<_>>(),
        [
            FuriganaGroup::Kanji("見失", "みうしな"),
            FuriganaGroup::Kana("う"),
        ]
    );

    assert_eq!(furigana.to_string(), "見失[みうしな]う");

    let furigana = Furigana::new("見失う", "みうしなう", "");

    assert_eq!(
        furigana.iter().collect::<Vec<_>>(),
        [
            FuriganaGroup::Kanji("見失", "みうしな"),
            FuriganaGroup::Kana("う"),
        ]
    );

    assert_eq!(furigana.to_string(), "見失[みうしな]う");
}

#[test]
fn test_long_suffix() {
    let furigana = Furigana::new("愛する", "あいする", "");

    assert_eq!(
        furigana.iter().collect::<Vec<_>>(),
        [
            FuriganaGroup::Kanji("愛", "あい"),
            FuriganaGroup::Kana("する"),
        ]
    );

    let furigana = Furigana::new("愛", "あい", "する");

    assert_eq!(
        furigana.iter().collect::<Vec<_>>(),
        [
            FuriganaGroup::Kanji("愛", "あい"),
            FuriganaGroup::Kana("する"),
        ]
    );
}
