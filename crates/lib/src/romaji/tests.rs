use super::analyze;

#[test]
fn segmentations() {
    assert_eq!(
        analyze("ひゃくりょく").collect::<Vec<_>>(),
        ["ひゃ", "く", "りょ", "く",]
    );
}

#[test]
fn romanization() {
    macro_rules! out {
        (nih = $expr:expr, $($tt:tt)*) => {
            $expr
        };

        (w = $expr:expr, $($tt:tt)*) => {
            $expr
        };
    }

    macro_rules! test {
        (
            $((
                $n:expr,
                $hira:tt, $kata:tt,
                $(nih = $nih:expr,)?
                $(w = $w:expr,)*
            ),)*
            $(kana $tt:tt,)*
        ) => {
            $(assert_eq!(
                analyze($hira).map(|segment| segment.romanize()).collect::<Vec<_>>(),
                [out!($(nih = $nih,)* $(w = $w,)*)]
            );)*

            $(assert_eq!(
                analyze($kata).map(|segment| segment.romanize()).collect::<Vec<_>>(),
                [out!($(nih = $nih,)* $(w = $w,)*)]
            );)*
        };
    }

    romaji_table!(test);
}

#[test]
fn hiragana() {
    macro_rules! test {
        (
            $((
                $n:expr,
                $hira:tt, $kata:tt,
                $(nih = $nih:expr,)?
                $(w = $w:expr,)*
            ),)*
            $(kana $tt:tt,)*
        ) => {
            $(assert_eq!(
                analyze($kata).map(|segment| segment.hiragana()).collect::<Vec<_>>(),
                [$hira]
            );)*

            $($(assert_eq!(
                analyze($w).map(|segment| segment.hiragana()).collect::<Vec<_>>(),
                [$hira]
            );)*)*

            $($(assert_eq!(
                analyze($nih).map(|segment| segment.hiragana()).collect::<Vec<_>>(),
                [$hira]
            );)*)*
        };
    }

    romaji_table!(test);
}

#[test]
fn katakana() {
    macro_rules! test {
        (
            $((
                $n:expr,
                $hira:tt, $kata:tt,
                $(nih = $nih:expr,)?
                $(w = $w:expr,)*
            ),)*
            $(kana $tt:tt,)*
        ) => {
            $(assert_eq!(
                analyze($hira).map(|segment| segment.katakana()).collect::<Vec<_>>(),
                [$kata]
            );)*

            $($(assert_eq!(
                analyze($w).map(|segment| segment.katakana()).collect::<Vec<_>>(),
                [$kata]
            );)*)*

            $($(assert_eq!(
                analyze($nih).map(|segment| segment.katakana()).collect::<Vec<_>>(),
                [$kata]
            );)*)*
        };
    }

    romaji_table!(test);
}
