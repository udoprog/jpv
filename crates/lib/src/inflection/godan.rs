#![allow(unused)]

#[derive(Debug, Clone, Copy)]
pub(crate) struct Godan {
    pub(super) a: &'static str,
    pub(super) i: &'static str,
    pub(super) u: &'static str,
    pub(super) e: &'static str,
    pub(super) o: &'static str,
    pub(super) te: &'static str,
    pub(super) te_stem: &'static str,
    pub(super) past: &'static str,
    pub(super) past_conditional: &'static str,
    // If で verb, else て.
    pub(super) de: bool,
}

impl Godan {
    const fn new(
        a: &'static str,
        i: &'static str,
        u: &'static str,
        e: &'static str,
        o: &'static str,
        te: &'static str,
        te_stem: &'static str,
        past: &'static str,
        past_conditional: &'static str,
        de: bool,
    ) -> Self {
        Self {
            a,
            i,
            u,
            e,
            o,
            te,
            te_stem,
            past,
            past_conditional,
            de,
        }
    }
}

/// The U godan table.
pub(super) static U: &Godan = &Godan::new(
    "わ",
    "い",
    "う",
    "え",
    "お",
    "って",
    "っ",
    "った",
    "ったら",
    false,
);
/// The TSU godan table.
pub(super) static TSU: &Godan = &Godan::new(
    "た",
    "ち",
    "つ",
    "て",
    "と",
    "って",
    "っ",
    "った",
    "ったら",
    false,
);
/// The RU godan table.
pub(super) static RU: &Godan = &Godan::new(
    "ら",
    "り",
    "る",
    "れ",
    "ろ",
    "って",
    "っ",
    "った",
    "ったら",
    false,
);
/// The KU godan table.
pub(super) static KU: &Godan = &Godan::new(
    "か",
    "き",
    "く",
    "け",
    "こ",
    "いて",
    "い",
    "いた",
    "いたら",
    false,
);
/// The GU godan table.
pub(super) static GU: &Godan = &Godan::new(
    "が",
    "ぎ",
    "ぐ",
    "げ",
    "ご",
    "いで",
    "い",
    "いだ",
    "いだら",
    true,
);
/// The MU godan table.
pub(super) static MU: &Godan = &Godan::new(
    "ま",
    "み",
    "む",
    "め",
    "も",
    "んで",
    "ん",
    "んだ",
    "んだら",
    true,
);
/// The BU godan table.
pub(super) static BU: &Godan = &Godan::new(
    "ば",
    "び",
    "ぶ",
    "べ",
    "ぼ",
    "んで",
    "ん",
    "んだ",
    "んだら",
    true,
);
/// The NU godan table.
pub(super) static NU: &Godan = &Godan::new(
    "な",
    "に",
    "ぬ",
    "ね",
    "の",
    "んで",
    "ん",
    "んだ",
    "んだら",
    true,
);
/// The SU godan table.
pub(super) static SU: &Godan = &Godan::new(
    "さ",
    "し",
    "す",
    "せ",
    "そ",
    "して",
    "し",
    "した",
    "したら",
    false,
);
/// The IKU/YUKU godan table.
pub(super) static IKU: &Godan = &Godan::new(
    "か",
    "き",
    "く",
    "け",
    "こ",
    "って",
    "っ",
    "った",
    "ったら",
    false,
);
