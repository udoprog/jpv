use std::collections::BTreeSet;

use lib::database::Source;
use lib::entities::KanjiInfo;
use lib::jmdict::{
    OwnedExample, OwnedExampleSentence, OwnedKanjiElement, OwnedReadingElement, OwnedSense,
};
use lib::{inflection, jmdict, kana, Form, Furigana, Inflection, OwnedInflections, Priority};
use yew::prelude::*;

use super::{colon, comma, iter, ruby, seq, spacing};

pub(crate) enum Msg {
    ToggleForm(usize, Form),
    ResetForm(usize),
    Change(String, Option<String>),
}

#[derive(Default)]
struct ExtraState {
    // Filter inflections to use among the select inflections.
    filter: Inflection,
}

#[derive(Debug)]
struct Combined {
    kanji: OwnedKanjiElement,
    reading: OwnedReadingElement,
}

impl Combined {
    fn is_common(&self) -> bool {
        !self.is_search_only() && !self.is_irregular() && !self.is_rare() && !self.is_outdated()
    }

    fn is_irregular(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::IrregularKanji)
    }

    fn is_rare(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::RareKanji)
    }

    fn is_outdated(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::OutdatedKanji)
    }

    fn is_search_only(&self) -> bool {
        self.kanji.info.contains(KanjiInfo::SearchOnlyKanji)
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
    states: Vec<ExtraState>,
    inflections: Vec<(inflection::Reading, OwnedInflections)>,
}

#[derive(Properties)]
pub struct Props {
    pub embed: bool,
    pub sources: BTreeSet<Source>,
    pub entry: jmdict::OwnedEntry,
    pub onchange: Callback<(String, Option<String>), ()>,
}

impl PartialEq for Props {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.sources == other.sources && self.entry.sequence == other.entry.sequence
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
            states: ctx
                .props()
                .sources
                .iter()
                .map(|_| ExtraState::default())
                .collect(),
            inflections: inflection::conjugate(&entry)
                .into_iter()
                .map(|(r, i, _)| (r, borrowme::to_owned(i)))
                .collect(),
        };

        this.refresh_entry(ctx);
        this
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::ToggleForm(index, form) => {
                if let Some(state) = self.states.get_mut(index) {
                    state.filter.toggle(form);
                }
            }
            Msg::ResetForm(index) => {
                if let Some(state) = self.states.get_mut(index) {
                    state.filter = Inflection::default();
                }
            }
            Msg::Change(text, english) => {
                ctx.props().onchange.emit((text, english));
            }
        }

        true
    }

    fn changed(&mut self, ctx: &Context<Self>, _: &Self::Properties) -> bool {
        let entry = borrowme::borrow(&ctx.props().entry);

        self.inflections = inflection::conjugate(&entry)
            .into_iter()
            .map(|(r, i, _)| (r, borrowme::to_owned(i)))
            .collect();

        self.states = ctx
            .props()
            .sources
            .iter()
            .map(|_| ExtraState::default())
            .collect();

        self.refresh_entry(ctx);
        true
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let sources = &ctx.props().sources;
        let entry = &ctx.props().entry;

        let inflections =
            sources
                .iter()
                .zip(&self.states)
                .enumerate()
                .flat_map(|(index, (source, state))| {
                    Some((index, state, find_inflection(source, &self.inflections)?))
                });

        let extras =
            inflections
                .clone()
                .take(1)
                .flat_map(|(index, state, (inflection, inflections))| {
                    render_extra(ctx, index, inflection, inflections, state.filter)
                });

        let reading = iter(
            seq(
                self.readings.iter().filter(|r| !r.is_search_only()),
                |e, not_last| render_reading(ctx, e, not_last),
            ),
            |iter| html!(<div class="block row entry-readings">{for iter}</div>),
        );

        let common = iter(
            seq(
                self.combined.iter().filter(|c| c.is_common()),
                |e, not_last| render_combined(ctx, e, not_last),
            ),
            |iter| {
                html! {
                    html!(<div class="block row">{for iter}</div>)
                }
            },
        );

        let other_kana = iter(
            seq(
                self.readings.iter().filter(|c| c.is_search_only()),
                |e, not_last| render_reading(ctx, e, not_last),
            ),
            |iter| {
                html! {
                    html!(<div class="block row"><span>{"Other kana"}</span>{colon()}{spacing()}{for iter}</div>)
                }
            },
        );

        let other_kanji = iter(
            seq(
                self.combined.iter().filter(|c| !c.is_common()),
                |e, not_last| render_combined(ctx, e, not_last),
            ),
            |iter| {
                html! {
                    html!(<div class="block row"><span>{"Other kanji"}</span>{colon()}{spacing()}{for iter}</div>)
                }
            },
        );

        let senses = iter(
            entry.senses.iter().map(|s| self.render_sense(ctx, s)),
            |iter| html!(<ul class="block block-lg list-numerical">{for iter}</ul>),
        );

        let sequence = (!ctx.props().embed).then(|| html! {
            <div class="block block row entry-sequence"><a href={format!("/api/entry/{}", entry.sequence)} target="_api">{format!("#{}", entry.sequence)}</a></div>
        });

        html! {
            <div class="block block-lg entry">
                {sequence}
                {for extras}
                {for reading}
                {for common}
                {for senses}
                {for other_kana}
                {for other_kanji}
            </div>
        }
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
                    .filter(|r| r.applies_to_nothing())
                    .cloned(),
            );

            if !self.combined.iter().any(|c| c.is_common()) {
                self.readings.extend(
                    entry
                        .reading_elements
                        .iter()
                        .filter(|r| !r.applies_to_nothing())
                        .cloned(),
                );
            }
        }
    }

    fn render_sense(&self, ctx: &Context<Self>, s: &OwnedSense) -> Html {
        let info = s
            .info
            .as_ref()
            .map(|info| html!(<div class="block row sense-info">{info}</div>));

        let stags = seq(s.stagr.iter().chain(s.stagk.iter()), |text, not_last| {
            let stag = if let Some(c) = self.combined.iter().find(|c| c.is_kanji(text)) {
                ruby(c.furigana())
            } else {
                html!(<>{text}</>)
            };

            let onclick = ctx.link().callback({
                let text = text.to_owned();
                move |_: MouseEvent| Msg::Change(text.clone(), None)
            });

            html!(<><span class="sense-stag clickable" {onclick}>{stag}{for not_last.then(comma)}</span></>)
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
            s.examples.iter().map(|e| self.render_example(ctx, e)),
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

    fn render_example(&self, ctx: &Context<Self>, example: &OwnedExample) -> Html {
        struct Languages<'a> {
            jpn: &'a str,
            eng: Option<&'a str>,
        }

        fn languages(sentences: &[OwnedExampleSentence]) -> Option<Languages<'_>> {
            let mut jpn = None;
            let mut eng = None;

            for sent in sentences {
                let out = match sent.lang.as_deref() {
                    Some("eng") => &mut eng,
                    Some("jpn") | None => &mut jpn,
                    _ => continue,
                };

                *out = Some(sent.text.as_str());
            }

            Some(Languages { jpn: jpn?, eng })
        }

        let texts = seq(example.texts.iter(), |text, not_last| {
            let onclick = ctx.link().callback({
                let text = text.to_owned();
                move |_: MouseEvent| Msg::Change(text.clone(), None)
            });

            let text = if let Some(c) = self.combined.iter().find(|c| c.is_kanji(text)) {
                ruby(c.furigana())
            } else {
                html!(<>{text}</>)
            };

            html!(<><span class="text highlight clickable" {onclick}>{text}</span>{for not_last.then(comma)}</>)
        });

        let sent = languages(&example.sentences).map(|l: Languages<'_>| {
            let onclick = ctx.link().callback({
                let jpn = l.jpn.to_owned();
                let eng = l.eng.map(ToOwned::to_owned);
                move |_: MouseEvent| Msg::Change(jpn.clone(), eng.clone())
            });

            let eng = l.eng.map(|text| html!(<span>{text}</span>));

            html!(<>{colon()}<span class="clickable" {onclick}>{l.jpn}</span>{for eng}</>)
        });

        html! {
            <div class="block row entry-example">{for texts}{for sent}</div>
        }
    }
}

/// Find the matching inflection based on the source.
fn find_inflection<'a>(
    source: &Source,
    inflections: &'a [(inflection::Reading, OwnedInflections)],
) -> Option<(Inflection, &'a OwnedInflections)> {
    match source {
        Source::Inflection { data } => {
            let Some((_, inflections)) = inflections.iter().find(|(r, _)| *r == data.reading)
            else {
                return None;
            };

            Some((data.inflection, inflections))
        }
        _ => None,
    }
}

fn render_extra(
    ctx: &Context<Entry>,
    index: usize,
    inflection: Inflection,
    inflections: &OwnedInflections,
    filter: Inflection,
) -> Option<Html> {
    let word = inflections.get(inflection ^ filter);

    let word = word.map(|w| ruby(w.furigana())).map(
        |word| html!(<div class="block row"><span class="text kanji highlight">{word}</span></div>),
    );

    let inflection_html = render_inflection(ctx, index, inflection, filter, inflections);
    let tutorials = render_tutorials(inflection, filter);

    Some(html! {
        <div class="block notice">
            <div class="block block-sm title">{"Result based on inflection:"}</div>
            <div class="block block-sm row bullets">{for inflection_html}</div>
            {tutorials}
            {for word}
        </div>
    })
}

fn render_inflection<'a>(
    ctx: &'a Context<Entry>,
    index: usize,
    inflection: Inflection,
    filter: Inflection,
    inflections: &'a OwnedInflections,
) -> impl Iterator<Item = Html> + 'a {
    let this = filter ^ inflection;

    let form = Inflection::all().iter().flat_map(move |f| {
        let mut candidate = this;
        candidate.toggle(f);

        let exists = inflections.contains(candidate);

        if !exists && !this.contains(f) {
            return None;
        }

        let class = classes! {
            "inflection",
            this.contains(f).then_some("active"),
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
        .then(|| html!(<span class="inflection danger" {onclick}>{"Reset"}</span>));

    form.chain(reset)
}

fn render_tutorials(inflection: Inflection, filter: Inflection) -> Html {
    let this = filter ^ inflection;

    let mut tutorials = Inflection::all().iter().flat_map(|f| {
        if this.contains(f) {
            Some((f, f.url()?))
        } else {
            None
        }
    });

    let first = tutorials.next();

    let Some(first) = first else {
        return html!();
    };

    let tutorials = seq(
        [first].into_iter().chain(tutorials),
        |(f, url), not_last| {
            html! {
                <>
                    <a href={url} target="_tutorial" title={format!("Tutorial for {}", f.title())}>{format!("Tutorial for `{}`", f.describe())}</a>
                    {for not_last.then(comma)}
                </>
            }
        },
    );

    html!(<div class="block block-sm tutorials row">{for tutorials}</div>)
}

fn render_reading(ctx: &Context<Entry>, reading: &OwnedReadingElement, not_last: bool) -> Html {
    let priority = reading.priority.iter().map(render_priority);

    let bullets = iter(
        priority.chain(bullets!(reading.info)),
        |iter| html!(<span class="bullets">{for iter}</span>),
    );

    let onclick = ctx.link().callback({
        let text = reading.text.to_owned();
        move |_: MouseEvent| Msg::Change(text.clone(), None)
    });

    html! {
        <>
            <span class="text kanji highlight clickable" {onclick}>{&reading.text}</span>
            {for bullets}
            {for not_last.then(comma)}
        </>
    }
}

fn render_combined(
    ctx: &Context<Entry>,
    c @ Combined { kanji, .. }: &Combined,
    not_last: bool,
) -> Html {
    let priority = kanji.priority.iter().map(render_priority);

    let bullets = iter(
        priority.chain(bullets!(kanji.info)),
        |iter| html!(<span class="bullets">{for iter}</span>),
    );

    let onclick = ctx.link().callback({
        let text = c.kanji.text.to_owned();
        move |_: MouseEvent| Msg::Change(text.clone(), None)
    });

    html! {
        <>
            <span class="text kanji highlight clickable" {onclick}>{ruby(c.furigana())}</span>
            {for bullets}
            {for not_last.then(comma)}
        </>
    }
}

fn render_priority(p: &Priority) -> Html {
    html!(<span class={format!("bullet prio-{}", p.category())} title={p.title()}>{p.category()}{p.level()}</span>)
}

/// A simple text sequence renderer.
#[inline]
fn texts<'a, I>(iter: I, extra: Option<&'static str>) -> impl Iterator<Item = Html> + 'a
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
