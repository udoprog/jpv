use std::collections::BTreeSet;
use std::{array, iter};

use lib::database::IndexExtra;
use lib::elements::{OwnedExample, OwnedKanjiElement, OwnedReadingElement, OwnedSense};
use lib::entities::KanjiInfo;
use lib::{adjective, elements, kana, romaji, verb, Form, Inflection, OwnedInflections};
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

pub(crate) struct Entry {
    extras_state: Vec<ExtraState>,
    show_inflection: bool,
    verb_inflections: Option<OwnedInflections>,
    adjective_inflections: Option<OwnedInflections>,
}

#[derive(Properties)]
pub struct Props {
    pub extras: BTreeSet<IndexExtra>,
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
        let entry = owned::borrow(&ctx.props().entry);

        Self {
            extras_state: ctx
                .props()
                .extras
                .iter()
                .map(|_| ExtraState::default())
                .collect(),
            show_inflection: false,
            verb_inflections: verb::conjugate(&entry).map(owned::to_owned),
            adjective_inflections: adjective::conjugate(&entry).map(owned::to_owned),
        }
    }

    fn update(&mut self, _: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleInflection => {
                self.show_inflection = !self.show_inflection;
            }
            Msg::ToggleForm(index, form) => {
                if let Some(state) = self.extras_state.get_mut(index) {
                    state.filter.toggle(form);
                }
            }
            Msg::ResetForm(index) => {
                if let Some(state) = self.extras_state.get_mut(index) {
                    state.filter = Inflection::default();
                }
            }
        }

        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _: &Self::Properties) -> bool {
        let entry = owned::borrow(&ctx.props().entry);
        self.verb_inflections = verb::conjugate(&entry).map(owned::to_owned);
        self.adjective_inflections = adjective::conjugate(&entry).map(owned::to_owned);
        self.extras_state = ctx
            .props()
            .extras
            .iter()
            .map(|_| ExtraState::default())
            .collect();
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

        let extras = extras.iter().zip(&self.extras_state).enumerate().flat_map(
            |(index, (extra, state))| render_extra(ctx, index, extra, inflections, state.filter),
        );

        let (reading, combined) = if entry.kanji_elements.is_empty() {
            let reading = render_iter(
                render_seq(entry.reading_elements.iter(), render_reading),
                |iter| html!(<div class="block block-lg row entry-reading">{for iter}</div>),
            );
            (reading, None)
        } else {
            let iter = entry.kanji_elements.iter().flat_map(|kanji| {
                entry.reading_elements.iter().flat_map(move |reading| {
                    reading.applies_to(&kanji.text).then_some((reading, kanji))
                })
            });

            let combined = render_seq(iter, render_combined);

            let combined = html! {
                html!(<div class="block block-lg row">{for combined}</div>)
            };

            let all_kanjis_rare = entry
                .kanji_elements
                .iter()
                .all(|k| k.info.contains(KanjiInfo::RareKanji));

            let mut all_readings = entry.reading_elements.iter();
            let mut filtered;

            let reading: &mut dyn DoubleEndedIterator<Item = _> = if !all_kanjis_rare {
                filtered = all_readings.filter(|r| r.no_kanji);
                &mut filtered
            } else {
                // Render regular readings, because *all* kanji readings are rare.
                &mut all_readings
            };

            let reading = render_seq(reading, render_reading);
            let reading = render_iter(
                reading,
                |iter| html!(<div class="block block-lg row">{for iter}</div>),
            );
            (reading, Some(combined))
        };

        let inflection = inflections.and_then(|inflections| {
            if self.show_inflection {
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
            } else {
                None
            }
        });

        let senses = render_iter(
            entry.senses.iter().enumerate().map(render_sense),
            |iter| html!(<ul class="block list-numerical">{for iter}</ul>),
        );

        let button = inflections.map(|_| {
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

        let entry_key_style = format!("display: none;");

        html! {
            <div class="block block-lg entry">
                <div class="block block-lg row entry-sequence">{entry.sequence}</div>
                <div class="block block-lg row entry-key" style={entry_key_style}>{format!("{:?}", key)}</div>
                {for extras}
                {for reading}
                {for combined}
                {for senses}
                {for button}
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

fn render_extra(
    ctx: &Context<Entry>,
    index: usize,
    extra: &IndexExtra,
    inflections: Option<&OwnedInflections>,
    filter: Inflection,
) -> Option<Html> {
    let (extra, inflection, title) = match extra {
        IndexExtra::VerbInflection(inflection) => (
            "Conjugation:",
            Some(*inflection),
            "Result based on verb conjugation",
        ),
        IndexExtra::AdjectiveInflection(inflection) => (
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

fn render_sense((_, s): (usize, &OwnedSense)) -> Html {
    let info = s
        .info
        .as_ref()
        .map(|info| html!(<div class="block row sense-info">{info}</div>));

    let stags = render_seq(
        s.stagr.iter().chain(s.stagk.iter()),
        |text, not_last| html!(<><span class="sense-stag">{text}</span>{for not_last.then(comma)}</>),
    );

    let stag = if !s.stagk.is_empty() || !s.stagr.is_empty() {
        Some(html! {
            <div class="block row sense-stags">{"Applies to: "}{for stags}</div>
        })
    } else {
        None
    };

    let glossary = render_texts(s.gloss.iter().map(|gloss| gloss.text.as_str()), None);
    let bullets = bullets!(s.pos, "sm")
        .chain(bullets!(s.misc, "sm"))
        .chain(bullets!(s.dialect, "sm"))
        .chain(bullets!(s.field, "sm"));

    let bullets = render_iter(
        bullets,
        |iter| html!(<span class="bullets">{for iter}</span>),
    );

    let glossary = render_iter(
        glossary.chain(bullets),
        |iter| html!(<div class="block row entry-glossary">{for iter}</div>),
    );

    let examples = render_iter(
        render_seq(s.examples.iter(), render_example),
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

fn render_reading(reading: &OwnedReadingElement, not_last: bool) -> Html {
    let priority = reading.priority.iter().map(|p| {
        html!(<span class={format!("bullet prio-{}", p.category())}>{p.category()}{p.level()}</span>)
    });

    let bullets = render_iter(
        priority.chain(bullets!(reading.info)),
        |iter| html!(<span class="bullets">{for iter}</span>),
    );

    html! {
        <>
            <span class="text kanji highlight">{reading.text.as_str()}</span>
            {for bullets}
            {for not_last.then(comma)}
        </>
    }
}

fn render_combined(
    (reading, kanji): (&OwnedReadingElement, &OwnedKanjiElement),
    not_last: bool,
) -> Html {
    let priority = kanji.priority.iter().map(|p| {
        html!(<span class={format!("bullet prio-{}", p.category())}>{p.category()}{p.level()}</span>)
    });

    let bullets = render_iter(
        priority.chain(bullets!(kanji.info)),
        |iter| html!(<span class="bullets">{for iter}</span>),
    );

    let furigana = kana::Full::new(kanji.text.as_str(), reading.text.as_str(), "").furigana();
    let text = ruby(furigana);

    html! {
        <>
            <span class="text kanji highlight">{text}</span>
            {for bullets}
            {for not_last.then(comma)}
        </>
    }
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

fn render_example(example: &OwnedExample, _: bool) -> Html {
    let texts = render_texts(example.texts.iter().map(String::as_str), Some("highlight"));

    let sent = example
        .sent
        .iter()
        .map(|sent| html!(<span>{sent.text.as_str()}</span>));

    html! {
        <div class="block row entry-example">{for texts}<span class="sep">{":"}</span>{for sent}</div>
    }
}

fn render_seq<'a, I, T, B>(iter: I, builder: B) -> impl DoubleEndedIterator<Item = Html> + 'a
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

#[inline]
fn render_texts<'a, I>(iter: I, extra: Option<&'static str>) -> impl Iterator<Item = Html> + 'a
where
    I: IntoIterator<Item = &'a str>,
    I::IntoIter: 'a + DoubleEndedIterator,
{
    render_seq(iter, move |text, not_last| {
        let class = classes! {
            "text",
            extra
        };

        html! {
            <>
                <span class={class}>{text}</span>
                {for not_last.then(comma)}
            </>
        }
    })
}

fn comma() -> Html {
    html!(<span class="sep">{","}</span>)
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

fn render_iter<I, F, O>(iter: I, render: F) -> Option<O>
where
    I: IntoIterator,
    F: FnOnce(iter::Chain<array::IntoIter<I::Item, 1>, I::IntoIter>) -> O,
{
    let mut iter = iter.into_iter();
    let first = iter.next();
    first.map(move |first| render([first].into_iter().chain(iter)))
}
