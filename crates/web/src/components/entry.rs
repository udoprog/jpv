use std::collections::BTreeSet;

use lib::database::IndexExtra;
use lib::elements::{Example, KanjiElement, ReadingElement, Sense};
use lib::entities::KanjiInfo;
use lib::{adjective, elements, kana, romaji, verb, Form, Inflection, Inflections};
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
    verb_inflections: Option<Inflections<'static>>,
    adjective_inflections: Option<Inflections<'static>>,
}

#[derive(Properties)]
pub struct Props {
    pub extras: BTreeSet<IndexExtra>,
    pub entry_key: elements::EntryKey,
    pub entry: elements::Entry<'static>,
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
        Self {
            extras_state: ctx
                .props()
                .extras
                .iter()
                .map(|_| ExtraState::default())
                .collect(),
            show_inflection: false,
            verb_inflections: verb::conjugate(&ctx.props().entry),
            adjective_inflections: adjective::conjugate(&ctx.props().entry),
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
        self.verb_inflections = verb::conjugate(&ctx.props().entry);
        self.adjective_inflections = adjective::conjugate(&ctx.props().entry);
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
            let reading = render_seq(entry.reading_elements.iter(), render_reading);

            let reading = (!entry.reading_elements.is_empty())
                .then(|| html!(<div class="block-l entry-reading">{for reading}</div>));

            (reading, None)
        } else {
            let iter = entry.kanji_elements.iter().flat_map(|kanji| {
                entry.reading_elements.iter().flat_map(move |reading| {
                    reading.applies_to(kanji.text).then_some((reading, kanji))
                })
            });

            let combined = render_seq(iter, render_combined);

            let combined = html! {
                html!(<div class="block-l entry-kanji">{for combined}</div>)
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

            let reading = (!entry.reading_elements.is_empty())
                .then(|| html!(<div class="block-l entry-reading">{for reading}</div>));

            (reading, Some(combined))
        };

        let senses = entry.senses.iter().enumerate().map(render_sense);

        let inflection = inflections.and_then(|inflections| {
            if self.show_inflection {
                let iter = inflections.inflections.iter().map(|(inflection, word)| {
                    html! {
                        <li class="inflections-entry block">
                            <div class="inflections-key">{format!("{inflection:?}")}</div>
                            <div class="inflections-value">{ruby(word.furigana())}</div>
                        </li>
                    }
                });

                Some(html! {
                    <ul class="block section inflections">
                        <li class="inflections-entry block">
                            <div class="inflections-key">{"Dictionary"}</div>
                            <div class="inflections-value">{ruby(inflections.dictionary.furigana())}</div>
                        </li>
                        {for iter}
                    </ul>
                })
            } else {
                None
            }
        });

        let button = inflections.map(|_| {
            let onclick = ctx.link().callback(|_: MouseEvent| Msg::ToggleInflection);

            let button = if self.show_inflection {
                html!(<button {onclick}>{"Hide inflections"}</button>)
            } else {
                html!(<button {onclick}>{"Show inflections"}</button>)
            };

            html! {
                <div class="block section">{button}</div>
            }
        });

        let entry_key_style = format!("display: none;");

        html! {
            <div class="block-l entry">
                <div class="block-l section entry-sequence">{entry.sequence}</div>
                <div class="block-l section entry-key" style={entry_key_style}>{format!("{:?}", key)}</div>
                {for extras}
                {for reading}
                {for combined}
                <ul class="block-l section entry-senses">{for senses}</ul>
                {for button}
                {for inflection}
            </div>
        }
    }
}

macro_rules! bullets {
    ($base:ident . $name:ident) => {
        $base.$name.iter().map(|d| {
            html!(<span class={format!(concat!("bullet {name} {name}-{}"), d.ident(), name = stringify!($name))} title={d.help()}>{d.ident()}</span>)
        })
    }
}

fn render_extra(
    ctx: &Context<Entry>,
    index: usize,
    extra: &IndexExtra,
    inflections: Option<&Inflections<'_>>,
    filter: Inflection,
) -> Option<Html> {
    let (extra, inflection) = match extra {
        IndexExtra::VerbInflection(inflection) => (format!("Verb conjugation:"), Some(*inflection)),
        IndexExtra::AdjectiveInflection(inflection) => {
            (format!("Adjective inflection:"), Some(*inflection))
        }
        _ => return None,
    };

    let word = inflection.and_then(|inf| inflections.and_then(|i| i.get(inf ^ filter)));

    let word = word
        .map(|w| ruby(w.furigana()))
        .map(|word| html!(<div class="block extra-word">{word}</div>));

    let inflection = inflection.map(|i| render_inflection(ctx, index, i, filter, inflections));

    Some(html! {
        <div class="block extra">
            <div class="block">{extra}{for inflection}</div>
            {for word}
        </div>
    })
}

fn render_sense((_, s): (usize, &Sense<'_>)) -> Html {
    let info = s
        .info
        .map(|info| html!(<div class="block sense-info">{info}</div>));

    let stags = render_seq(
        s.stagr.iter().chain(s.stagk.iter()),
        |text, not_last| html!(<><span class="sense-stag">{text}</span>{for not_last.then(sep)}</>),
    );

    let stag = if !s.stagk.is_empty() || !s.stagr.is_empty() {
        Some(html! {
            <div class="block sense-stags">{"Applies to: "}{for stags}</div>
        })
    } else {
        None
    };

    let any =
        !s.pos.is_empty() || !s.misc.is_empty() || !s.dialect.is_empty() || !s.field.is_empty();

    let bullets = any.then(|| {
        let pos = bullets!(s.pos);
        let misc = bullets!(s.misc);
        let dialect = bullets!(s.dialect);
        let field = bullets!(s.field);
        html!(<span class="entry-sense-bullets">{for pos}{for misc}{for dialect}{for field}</span>)
    });

    let glossary = render_texts(s.gloss.iter().map(|gloss| gloss.text));
    let glossary = (!s.gloss.is_empty())
        .then(move || html!(<div class="block entry-glossary">{for glossary}{for bullets}</div>));

    let examples = render_seq(s.examples.iter(), render_example);
    let examples = (!s.examples.is_empty())
        .then(move || html!(<div class="block entry-examples">{for examples}</div>));

    html! {
        <li class="block-l entry-sense">
            {for glossary}
            {for info}
            {for stag}
            {for examples}
        </li>
    }
}

fn render_reading(reading: &ReadingElement<'_>, not_last: bool) -> Html {
    let priority = (!reading.priority.is_empty()).then(move || {
        let priority = reading.priority.iter().map(|p| {
            html!(<span class={format!("bullet prio-{}", p.category())}>{p.category()}{p.level()}</span>)
        });

        html!(<span class="priority">{for priority}</span>)
    });

    let info = (!reading.info.is_empty()).then(|| {
        let info = bullets!(reading.info);
        html!(<span class="info">{for info}</span>)
    });

    let bullets = (!reading.priority.is_empty() || !reading.info.is_empty())
        .then(|| html!(<span class="entry-reading-bullets">{for priority}{for info}</span>));

    html! {
        <>
            <span class="text">{reading.text}</span>
            {for bullets}
            {for not_last.then(sep)}
        </>
    }
}

fn render_combined(
    (reading, kanji): (&ReadingElement<'_>, &KanjiElement<'_>),
    not_last: bool,
) -> Html {
    let sep = not_last.then(sep);

    let priority = (!kanji.priority.is_empty()).then(|| {
        let priority = kanji.priority.iter().map(|p| {
            html!(<span class={format!("bullet prio-{}", p.category())}>{p.category()}{p.level()}</span>)
        });

        html!(<span class="priority">{for priority}</span>)
    });

    let info = (!kanji.info.is_empty()).then(|| {
        let info = bullets!(kanji.info);
        html!(<span class="info">{for info}</span>)
    });

    let bullets = (!kanji.priority.is_empty() || !kanji.info.is_empty())
        .then(move || html!(<span class="entry-kanji-bullets">{for priority}{for info}</span>));

    let furigana = kana::Word::new(kanji.text, reading.text).furigana();

    let text = ruby(furigana);

    html! {
        <>
            <span class="text">{text}</span>
            {for bullets}
            {for sep}
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

fn render_example(example: &Example<'_>, _: bool) -> Html {
    let texts = render_texts(example.texts.iter().copied());

    let sent = example
        .sent
        .iter()
        .map(|sent| html!(<span class="block entry-example-sentence">{sent.text}</span>));

    html! {
        <div class="entry-example">{for texts}<span class="sep">{":"}</span>{for sent}</div>
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
fn render_texts<'a, I>(iter: I) -> impl Iterator<Item = Html> + 'a
where
    I: IntoIterator<Item = &'a str>,
    I::IntoIter: 'a + DoubleEndedIterator,
{
    render_seq(iter, |text, not_last| {
        html! {
            <>
                <span class="text">{text}</span>
                {for not_last.then(sep)}
            </>
        }
    })
}

fn sep() -> Html {
    html!(<span class="sep">{","}</span>)
}

fn render_inflection(
    ctx: &Context<Entry>,
    index: usize,
    inflection: Inflection,
    filter: Inflection,
    inflections: Option<&Inflections<'_>>,
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
            this.contains(f).then_some("active"),
            exists.then_some("clickable"),
            "inflection-form"
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
        .then(|| html!(<span class="inflection-reset active" {onclick}>{"Reset"}</span>));

    html!(<>{for form}{for reset}</>)
}
