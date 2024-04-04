use super::*;

use FuriganaGroup::Kana as Kn;
use FuriganaGroup::Kanji as K;

macro_rules! test_case {
    ($kanji:expr, $kana:expr, $expected:expr) => {
        test_case!($kanji, $kana, $expected, "");
    };

    ($kanji:expr, $kana:expr, $expected:expr, $suffix:expr) => {
        assert_eq!(
            furigana2($kanji, $kana, $suffix).collect::<Vec<_>>(),
            $expected
        );
    };
}

#[test]
fn test_kana_prefix() {
    test_case!("お金", "おかね", [Kn("お"), K("金", "かね")]);

    test_case!(
        "私はお金がない星",
        "わたしはおかねがないほし",
        [
            K("私", "わたし"),
            Kn("はお"),
            K("金", "かね"),
            Kn("がない"),
            K("星", "ほし")
        ]
    );

    test_case!(
        "お金がない星",
        "おかねがないほし",
        [Kn("お"), K("金", "かね"), Kn("がない"), K("星", "ほし")]
    );

    test_case!(
        "私はお金がない",
        "わたしはおかねがない",
        [K("私", "わたし"), Kn("はお"), K("金", "かね"), Kn("がない")]
    );

    test_case!("十八禁", "じゅうはちきん", [K("十八禁", "じゅうはちきん")]);
    test_case!("18禁", "じゅうはちきん", [K("18禁", "じゅうはちきん")]);
    test_case!("見失", "みうしな", [K("見失", "みうしな"), Kn("う")], "う");
    test_case!("見失う", "みうしなう", [K("見失", "みうしな"), Kn("う")]);

    test_case!("愛する", "あいする", [K("愛", "あい"), Kn("する")]);
    test_case!("愛", "あい", [K("愛", "あい"), Kn("する")], "する");

    test_case!(
        "兄たり難く弟たり難し",
        "けいたりがたくていたりがたし",
        [
            K("兄", "けい"),
            Kn("たり"),
            K("難", "がた"),
            Kn("く"),
            K("弟", "てい"),
            Kn("たり"),
            K("難", "がた"),
            Kn("し")
        ]
    );

    test_case!(
        "月とすっぽん",
        "つきとすっぽん",
        [K("月", "つき"), Kn("とすっぽん")]
    );
}
