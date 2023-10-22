#![allow(unused)]

#[derive(Debug, Clone, Copy)]
pub(super) struct Godan {
    pub(super) a: &'static str,
    pub(super) i: &'static str,
    pub(super) u: &'static str,
    pub(super) e: &'static str,
    pub(super) o: &'static str,
    pub(super) te: &'static str,
    pub(super) te_stem: &'static str,
    pub(super) past: &'static str,
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
            de,
        }
    }
}

/// The U godan table.
pub(super) const U: Godan = Godan::new("わ", "い", "う", "え", "お", "って", "っ", "った", false);
/// The TSU godan table.
pub(super) const TSU: Godan = Godan::new("た", "ち", "つ", "て", "と", "って", "っ", "った", false);
/// The RU godan table.
pub(super) const RU: Godan = Godan::new("ら", "り", "る", "れ", "ろ", "って", "っ", "った", false);
/// The KU godan table.
pub(super) const KU: Godan = Godan::new("か", "き", "く", "け", "こ", "いて", "い", "いた", false);
/// The GU godan table.
pub(super) const GU: Godan = Godan::new("が", "ぎ", "ぐ", "げ", "ご", "いで", "い", "いだ", true);
/// The MU godan table.
pub(super) const MU: Godan = Godan::new("ま", "み", "む", "め", "も", "んで", "ん", "んだ", true);
/// The BU godan table.
pub(super) const BU: Godan = Godan::new("ば", "び", "ぶ", "べ", "ぼ", "んで", "ん", "んだ", true);
/// The NU godan table.
pub(super) const NU: Godan = Godan::new("な", "に", "ぬ", "ね", "の", "んで", "ん", "んだ", true);
/// The SU godan table.
pub(super) const SU: Godan = Godan::new("さ", "し", "す", "せ", "そ", "して", "し", "した", false);
/// The IKU/YUKU godan table.
pub(super) const IKU: Godan = Godan::new("か", "き", "く", "け", "こ", "って", "っ", "った", false);
