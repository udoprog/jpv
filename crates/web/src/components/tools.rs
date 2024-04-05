use std::array;
use std::iter;

use yew::prelude::*;

macro_rules! bullets {
    ($ctx:expr, $base:ident . $name:ident $(, $($tt:tt)*)?) => {{
        let onclick = $ctx.link().callback(Msg::AddTag);

        $base.$name.iter().map(move |d| {
            let class = classes! {
                "bullet",
                stringify!($name),
                format!("{}-{}", stringify!($name), d.ident()),
                $($($tt)*)*
            };

            let ident = d.ident();
            let onclick = onclick.reform(move |_| ident);
            html!(<a {class} title={d.help()} {onclick}>{d.ident()}</a>)
        })
    }}
}

/// Construct a convenient sequence callback which calls the given `builder`
/// with the item being iterated over, and a `bool` indicating if it is the last
/// in sequence.
pub(super) fn seq<'a, I, T, B>(iter: I, builder: B) -> impl Iterator<Item = Html> + 'a
where
    I: IntoIterator<Item = T>,
    I::IntoIter: 'a,
    B: 'a + Fn(T, bool) -> Html,
{
    let mut it = iter.into_iter().peekable();

    iter::from_fn(move || {
        let value = it.next()?;
        Some(builder(value, it.peek().is_some()))
    })
}

pub(super) fn comma() -> Html {
    html!(<><span class="sep">{","}</span>{spacing()}</>)
}

pub(super) fn colon() -> Html {
    html!(<span class="sep">{":"}</span>)
}

/// A simple spacing to insert between elements.
pub(super) fn spacing() -> Html {
    html!(<span class="sep">{" "}</span>)
}

/// Render the given iterator if it has at least one element. Else returns
/// `None`.
pub(super) fn iter<I, F, O>(iter: I, render: F) -> Option<O>
where
    I: IntoIterator,
    F: FnOnce(iter::Chain<array::IntoIter<I::Item, 1>, I::IntoIter>) -> O,
{
    let mut iter = iter.into_iter();
    let first = iter.next();
    first.map(move |first| render([first].into_iter().chain(iter)))
}

pub(super) fn romaji(furigana: lib::Furigana<'_>) -> String {
    let mut romaji = String::new();

    for string in furigana.reading() {
        for segment in lib::romaji::analyze(string) {
            romaji.push_str(segment.romanize());
        }
    }

    romaji
}

pub(super) fn ruby(furigana: lib::Furigana<'_>) -> Html {
    let elements = furigana.iter().map(|group| match group {
        lib::FuriganaGroup::Kanji(kanji, kana) => {
            html!(<ruby>{kanji}<rt>{kana}</rt></ruby>)
        }
        lib::FuriganaGroup::Kana(kana) => {
            html!({ kana })
        }
    });

    html!(<>{for elements}</>)
}
