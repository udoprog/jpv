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
    pub(super) hypothetical_conversational: Option<(&'static str, &'static str)>,
    // If で verb, else て.
    pub(super) de: bool,
}

/// The U godan table.
pub(super) static U: &Godan = &Godan {
    a: "わ",
    i: "い",
    u: "う",
    e: "え",
    o: "お",
    te: "って",
    te_stem: "っ",
    past: "った",
    past_conditional: "ったら",
    hypothetical_conversational: Some(("やぁ", "なけやぁ")),
    de: false,
};

/// The TSU godan table.
pub(super) static TSU: &Godan = &Godan {
    a: "た",
    i: "ち",
    u: "つ",
    e: "て",
    o: "と",
    te: "って",
    te_stem: "っ",
    past: "った",
    past_conditional: "ったら",
    hypothetical_conversational: None,
    de: false,
};

/// The RU godan table.
pub(super) static RU: &Godan = &Godan {
    a: "ら",
    i: "り",
    u: "る",
    e: "れ",
    o: "ろ",
    te: "って",
    te_stem: "っ",
    past: "った",
    past_conditional: "ったら",
    hypothetical_conversational: Some(("りゃ", "なけりゃ")),
    de: false,
};
/// The KU godan table.
pub(super) static KU: &Godan = &Godan {
    a: "か",
    i: "き",
    u: "く",
    e: "け",
    o: "こ",
    te: "いて",
    te_stem: "い",
    past: "いた",
    past_conditional: "いたら",
    hypothetical_conversational: Some(("きゃ", "なけきゃ")),
    de: false,
};
/// The GU godan table.
pub(super) static GU: &Godan = &Godan {
    a: "が",
    i: "ぎ",
    u: "ぐ",
    e: "げ",
    o: "ご",
    te: "いで",
    te_stem: "い",
    past: "いだ",
    past_conditional: "いだら",
    hypothetical_conversational: None,
    de: true,
};
/// The MU godan table.
pub(super) static MU: &Godan = &Godan {
    a: "ま",
    i: "み",
    u: "む",
    e: "め",
    o: "も",
    te: "んで",
    te_stem: "ん",
    past: "んだ",
    past_conditional: "んだら",
    hypothetical_conversational: None,
    de: true,
};
/// The BU godan table.
pub(super) static BU: &Godan = &Godan {
    a: "ば",
    i: "び",
    u: "ぶ",
    e: "べ",
    o: "ぼ",
    te: "んで",
    te_stem: "ん",
    past: "んだ",
    past_conditional: "んだら",
    hypothetical_conversational: None,
    de: true,
};
/// The NU godan table.
pub(super) static NU: &Godan = &Godan {
    a: "な",
    i: "に",
    u: "ぬ",
    e: "ね",
    o: "の",
    te: "んで",
    te_stem: "ん",
    past: "んだ",
    past_conditional: "んだら",
    hypothetical_conversational: None,
    de: true,
};
/// The SU godan table.
pub(super) static SU: &Godan = &Godan {
    a: "さ",
    i: "し",
    u: "す",
    e: "せ",
    o: "そ",
    te: "して",
    te_stem: "し",
    past: "した",
    past_conditional: "したら",
    hypothetical_conversational: None,
    de: false,
};
/// The IKU/YUKU godan table.
pub(super) static IKU: &Godan = &Godan {
    a: "か",
    i: "き",
    u: "く",
    e: "け",
    o: "こ",
    te: "って",
    te_stem: "っ",
    past: "った",
    past_conditional: "ったら",
    hypothetical_conversational: Some(("きゃ", "なけきゃ")),
    de: false,
};
