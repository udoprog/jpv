use std::collections::BTreeSet;
use std::{array, iter};

use lib::database::IndexSource;
use lib::elements::{OwnedExample, OwnedKanjiElement, OwnedReadingElement, OwnedSense};
use lib::entities::KanjiInfo;
use lib::{
    adjective, elements, kana, romaji, verb, Form, Furigana, Inflection, OwnedInflections, Priority,
};
use yew::prelude::*;

pub(crate) enum Msg {
    ToggleInflection,
    ToggleForm(usize, Form),
    ResetForm(usize),
}

#[derive(Default)]
struct ExtraState {
    // Filter inflections to use among the select inflections.
    filter: Inflection,
}

struct Combined {
    kanji: OwnedKanjiElement,
    reading: OwnedReadingElement,
}

impl Combined {
    fn is_common(&self) -> bool {
        !self.is_rare() && !self.is_outdated()
    }

    fn is_rare(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::RareKanji)
    }

    fn is_outdated(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::OutdatedKanji)
    }

    /// Provide furigana iterator for the combined reading.
    fn furigana(&self) -> Furigana<'_, 1, 1> {
        kana::Full::new(&self.kanji.text, &self.reading.text, "").furigana()
    }

    /// Test if this contains the given text.
    fn is_kanji(&self, text: &str) -> bool {
        self.kanji.text == text
    }
}

pub(crate) struct Entry {
    combined: Vec<Combined>,
    readings: Vec<OwnedReadingElement>,
    extras: Vec<ExtraState>,
    show_inflection: bool,
    verb_inflections: Option<OwnedInflections>,
    adjective_inflections: Option<OwnedInflections>,
}

#[derive(Properties)]
pub struct Props {
    pub extras: BTreeSet<IndexSource>,
    pub entry_key: elements::EntryKey,
    pub entry: elements::OwnedEntry,
}

impl PartialEq for Props {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.extras == other.extras
            && self.entry_key == other.entry_key
            && self.entry.sequence == other.entry.sequence
    }
}

impl Component for Entry {
    type Message = Msg;
    type Properties = Props;

    fn create(ctx: &Context<Self>) -> Self {
        let entry = borrowme::borrow(&ctx.props().entry);

        let mut this = Self {
            combined: Vec::new(),
            readings: Vec::new(),
            extras: ctx
                .props()
                .extras
                .iter()
                .map(|_| ExtraState::default())
                .collect(),
            show_inflection: false,
            verb_inflections: verb::conjugate(&entry).map(borrowme::to_owned),
            adjective_inflections: adjective::conjugate(&entry).map(borrowme::to_owned),
        };

        this.refresh_entry(ctx);
        this
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleInflection => {
                self.show_inflection = !self.show_inflection;
            }
            Msg::ToggleForm(index, form) => {
                if let Some(state) = self.extras.get_mut(index) {
                    state.filter.toggle(form);
                }
            }
            Msg::ResetForm(index) => {
                if let Some(state) = self.extras.get_mut(index) {
                    state.filter = Inflection::default();
                }
            }
        }

        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _: &Self::Properties) -> bool {
        let entry = borrowme::borrow(&ctx.props().entry);
        self.verb_inflections = verb::conjugate(&entry).map(borrowme::to_owned);
        self.adjective_inflections = adjective::conjugate(&entry).map(borrowme::to_owned);
        self.extras = ctx
            .props()
            .extras
            .iter()
            .map(|_| ExtraState::default())
            .collect();

        self.refresh_entry(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let extras = &ctx.props().extras;
        let key = &ctx.props().entry_key;
        let entry = &ctx.props().entry;

        let inflections = self
            .verb_inflections
            .as_ref()
            .or(self.adjective_inflections.as_ref());

        let extras =
            extras
                .iter()
                .zip(&self.extras)
                .enumerate()
                .flat_map(|(index, (extra, state))| {
                    render_extra(ctx, index, extra, inflections, state.filter)
                });

        let common = iter(
            seq(
                self.combined.iter().filter(|c| c.is_common()),
                render_combined,
            ),
            |iter| {
                html! {
                    html!(<div class="block block-lg row">{for iter}</div>)
                }
            },
        );

        let special_readings = |what, f: fn(&Combined) -> bool| {
            iter(
                seq(self.combined.iter().filter(|c| f(c)), render_combined),
                |iter| {
                    html! {
                        html!(<div class="block block-lg row"><span>{what}</span>{colon()}{for iter}</div>)
                    }
                },
            )
        };

        let rare = special_readings("Rare kanji", Combined::is_rare);
        let outdated = special_readings("Outdated kanji", Combined::is_outdated);

        let reading = seq(self.readings.iter(), render_reading);
        let reading = iter(
            reading,
            |iter| html!(<div class="block block-lg row entry-readings">{for iter}</div>),
        );

        let senses = iter(
            entry.senses.iter().map(|s| self.render_sense(s)),
            |iter| html!(<ul class="block list-numerical">{for iter}</ul>),
        );

        let show_inflections = inflections.map(|_| {
            let onclick = ctx.link().callback(|_: MouseEvent| Msg::ToggleInflection);

            let button = if self.show_inflection {
                "Hide inflections"
            } else {
                "Show inflections"
            };

            html! {
                <div class="block row">
                    <button class="btn btn-lg" {onclick}>{button}</button>
                </div>
            }
        });

        let inflection = inflections.filter(|_| self.show_inflection).and_then(|inflections| {
            let iter = inflections.inflections.iter().map(|(inflection, word)| {
                html! {
                    <li class="section">
                        <div class="block">{format!("{inflection:?}")}</div>
                        <div class="block text kanji highlight">{ruby(word.furigana())}</div>
                    </li>
                }
            });

            Some(html! {
                <ul class="block list-bulleted">
                    <li class="section">
                        <div class="block">{"Dictionary"}</div>
                        <div class="block text kanji highlight">{ruby(inflections.dictionary.furigana())}</div>
                    </li>
                    {for iter}
                </ul>
            })
        });

        let entry_key_style = format!("display: none;");

        html! {
            <div class="block block-lg entry">
                <div class="block block row entry-sequence">{entry.sequence}</div>
                <div class="block block row entry-key" style={entry_key_style}>{format!("{:?}", key)}</div>
                {for extras}
                {for reading}
                {for common}
                {for senses}
                {for rare}
                {for outdated}
                {for show_inflections}
                {for inflection}
            </div>
        }
    }
}

macro_rules! bullets {
    ($base:ident . $name:ident $(, $($tt:tt)*)?) => {
        $base.$name.iter().map(|d| {
            let class = classes! {
                "bullet",
                stringify!($name),
                format!("{}-{}", stringify!($name), d.ident()),
                $($($tt)*)*
            };

            html!(<span class={class} title={d.help()}>{d.ident()}</span>)
        })
    }
}

impl Entry {
    fn refresh_entry(&mut self, ctx: &Context<Self>) {
        self.combined.clear();
        self.readings.clear();

        let entry = &ctx.props().entry;

        if entry.kanji_elements.is_empty() {
            self.readings.extend(entry.reading_elements.iter().cloned());
        } else {
            self.combined
                .extend(entry.kanji_elements.iter().flat_map(|kanji| {
                    entry.reading_elements.iter().flat_map(move |reading| {
                        reading.applies_to(&kanji.text).then_some(Combined {
                            kanji: kanji.clone(),
                            reading: reading.clone(),
                        })
                    })
                }));

            self.readings.extend(
                entry
                    .reading_elements
                    .iter()
                    .filter(|r| r.no_kanji)
                    .cloned(),
            );

            if !self.combined.iter().any(|c| c.is_common()) {
                self.readings.extend(
                    entry
                        .reading_elements
                        .iter()
                        .filter(|r| !r.no_kanji)
                        .cloned(),
                );
            }
        }
    }

    fn render_sense(&self, s: &OwnedSense) -> Html {
        let info = s
            .info
            .as_ref()
            .map(|info| html!(<div class="block row sense-info">{info}</div>));

        let stags = seq(s.stagr.iter().chain(s.stagk.iter()), |text, not_last| {
            if let Some(c) = self.combined.iter().find(|c| c.is_kanji(text)) {
                html!(<><span class="sense-stag">{ruby(c.furigana())}</span>{for not_last.then(comma)}</>)
            } else {
                html!(<><span class="sense-stag">{text}</span>{for not_last.then(comma)}</>)
            }
        });

        let stag = iter(stags, |stags| {
            html! {
                <div class="block row sense-stags"><span>{"Applies to"}</span>{colon()}{for stags}</div>
            }
        });

        let glossary = texts(s.gloss.iter().map(|gloss| &gloss.text), None);
        let bullets = bullets!(s.pos, "sm")
            .chain(bullets!(s.misc, "sm"))
            .chain(bullets!(s.dialect, "sm"))
            .chain(bullets!(s.field, "sm"));

        let bullets = iter(
            bullets,
            |iter| html!(<>{spacing()}<span class="bullets">{for iter}</span></>),
        );

        let glossary = iter(
            glossary.chain(bullets),
            |iter| html!(<div class="block row entry-glossary">{for iter}</div>),
        );

        let examples = iter(
            s.examples.iter().map(|e| self.render_example(e)),
            |iter| html!(<div class="block entry-examples">{for iter}</div>),
        );

        html! {
            <li class="section entry-sense">
                {for glossary}
                {for info}
                {for stag}
                {for examples}
            </li>
        }
    }

    fn render_example(&self, example: &OwnedExample) -> Html {
        let texts = seq(example.texts.iter(), |text, not_last| {
            if let Some(c) = self.combined.iter().find(|c| c.is_kanji(text)) {
                html!(<><span class="text highlight">{ruby(c.furigana())}</span>{for not_last.then(comma)}</>)
            } else {
                html!(<><span class="text highlight">{text}</span>{for not_last.then(comma)}</>)
            }
        });

        let sent = example
            .sentences
            .iter()
            .map(|sent| html!(<span>{&sent.text}</span>));

        html! {
            <div class="block row entry-example">{for texts}{colon()}{for sent}</div>
        }
    }
}

fn render_extra(
    ctx: &Context<Entry>,
    index: usize,
    extra: &IndexSource,
    inflections: Option<&OwnedInflections>,
    filter: Inflection,
) -> Option<Html> {
    let (extra, inflection, title) = match extra {
        IndexSource::VerbInflection { inflection } => (
            "Conjugation:",
            Some(*inflection),
            "Result based on verb conjugation",
        ),
        IndexSource::AdjectiveInflection { inflection } => (
            "Inflection:",
            Some(*inflection),
            "Result based on adverb inflection",
        ),
        _ => return None,
    };

    let word = inflection.and_then(|inf| inflections.and_then(|i| i.get(inf ^ filter)));

    let word = word.map(|w| ruby(w.furigana())).map(
        |word| html!(<div class="block row"><span class="text kanji highlight">{word}</span></div>),
    );

    let inflection = inflection.map(|i| render_inflection(ctx, index, i, filter, inflections));

    Some(html! {
        <div class="block notice">
            <div class="block row"><span title={title}>{extra}</span>{for inflection}</div>
            {for word}
        </div>
    })
}

fn render_reading(reading: &OwnedReadingElement, not_last: bool) -> Html {
    let priority = reading.priority.iter().map(render_priority);

    let bullets = iter(
        priority.chain(bullets!(reading.info)),
        |iter| html!(<span class="bullets">{for iter}</span>),
    );

    html! {
        <>
            <span class="text kanji highlight">{&reading.text}</span>
            {for bullets}
            {for not_last.then(comma)}
        </>
    }
}

fn render_combined(c @ Combined { kanji, .. }: &Combined, not_last: bool) -> Html {
    let priority = kanji.priority.iter().map(render_priority);

    let bullets = iter(
        priority.chain(bullets!(kanji.info)),
        |iter| html!(<span class="bullets">{for iter}</span>),
    );

    html! {
        <>
            <span class="text kanji highlight">{ruby(c.furigana())}</span>
            {for bullets}
            {for not_last.then(comma)}
        </>
    }
}

fn render_priority(p: &Priority) -> Html {
    html!(<span class={format!("bullet prio-{}", p.category())} title={p.title()}>{p.category()}{p.level()}</span>)
}

fn ruby<const N: usize, const S: usize>(furigana: lib::Furigana<N, S>) -> Html {
    let elements = furigana.iter().map(|group| match group {
        lib::FuriganaGroup::Kanji(kanji, kana) => {
            html!(<ruby>{kanji}<rt>{kana}</rt></ruby>)
        }
        lib::FuriganaGroup::Kana(kana) => {
            html!({ kana })
        }
    });

    let mut romaji = String::new();

    for string in furigana.reading().as_slice() {
        for segment in romaji::analyze(string) {
            romaji.push_str(segment.romanize());
        }
    }

    html!(<span title={romaji}>{for elements}</span>)
}

/// Construct a convenient sequence callback which calls the given `builder`
/// with the item being iterated over, and a `bool` indicating if it is the last
/// in sequence.
pub(crate) fn seq<'a, I, T, B>(iter: I, builder: B) -> impl DoubleEndedIterator<Item = Html> + 'a
where
    I: IntoIterator<Item = T>,
    I::IntoIter: 'a + DoubleEndedIterator,
    B: 'a + Copy + Fn(T, bool) -> Html,
    T: 'a,
{
    let mut it = iter.into_iter();
    let last = it.next_back().map(move |value| builder(value, false));
    it.map(move |value| builder(value, true)).chain(last)
}

/// A simple text sequence renderer.
#[inline]
fn texts<'a, I>(iter: I, extra: Option<&'static str>) -> impl DoubleEndedIterator<Item = Html> + 'a
where
    I: IntoIterator,
    I::Item: 'a + AsRef<str>,
    I::IntoIter: 'a + DoubleEndedIterator,
{
    seq(iter, move |text, not_last| {
        let class = classes!("text", extra);

        html! {
            <>
                <span class={class}>{text.as_ref()}</span>
                {for not_last.then(comma)}
            </>
        }
    })
}

fn comma() -> Html {
    html!(<span class="sep">{","}</span>)
}

fn colon() -> Html {
    html!(<span class="sep">{":"}</span>)
}

/// A simple spacing to insert between elements.
fn spacing() -> Html {
    html!(<span class="sep">{" "}</span>)
}

fn render_inflection(
    ctx: &Context<Entry>,
    index: usize,
    inflection: Inflection,
    filter: Inflection,
    inflections: Option<&OwnedInflections>,
) -> Html {
    let this = filter ^ inflection;

    let form = Inflection::all().iter().flat_map(|f| {
        let mut candidate = this;
        candidate.toggle(f);

        let exists = inflections
            .map(|i| i.contains(candidate))
            .unwrap_or_default();

        if !exists && !this.contains(f) {
            return None;
        }

        let class = classes! {
            "bullet",
            "bullet-inflection",
            this.contains(f).then_some("active"),
            exists.then_some("clickable"),
        };

        let onclick = ctx
            .link()
            .batch_callback(move |_: MouseEvent| exists.then_some(Msg::ToggleForm(index, f)));
        Some(html!(<span {class} {onclick} title={f.title()}>{f.describe()}</span>))
    });

    let onclick = ctx
        .link()
        .callback(move |_: MouseEvent| Msg::ResetForm(index));

    let reset = (!filter.is_empty())
        .then(|| html!(<span class="bullet bullet-destructive active clickable" {onclick}>{"Reset"}</span>));

    html!(<><span class="bullets">{for form}{for reset}</span></>)
}

/// Render the given iterator if it has at least one element. Else returns
/// `None`.
fn iter<I, F, O>(iter: I, render: F) -> Option<O>
where
    I: IntoIterator,
    F: FnOnce(iter::Chain<array::IntoIter<I::Item, 1>, I::IntoIter>) -> O,
{
    let mut iter = iter.into_iter();
    let first = iter.next();
    first.map(move |first| render([first].into_iter().chain(iter)))
}
